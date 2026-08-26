use crate::{history::TimelineEntry, repository::AppError};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

const ACTIVE_STATUSES: &[&str] = &["active", "weak", "recovering"];
const ACTIONS: &[&str] = &["loss", "deactivate", "reactivate"];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyLifecycleRecord {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub action: String,
    pub occurred_at: String,
    pub previous_status: String,
    pub new_status: String,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeColonyLifecycle {
    pub colony_id: String,
    pub action: String,
    pub occurred_at: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, FromRow)]
struct EntrySnapshot {
    id: String,
    occurred_at: String,
    origin_type: String,
    origin_notes: Option<String>,
}

fn required(value: &str, field: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} é obrigatório.")));
    }
    Ok(value.to_owned())
}

fn optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_action(value: &str) -> Result<String, AppError> {
    let value = required(value, "Ação do ciclo de vida")?;
    if ACTIONS.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "Ação inválida. Use loss, deactivate ou reactivate.".to_owned(),
        ))
    }
}

async fn get(pool: &SqlitePool, id: &str) -> Result<ColonyLifecycleRecord, AppError> {
    Ok(sqlx::query_as::<_, ColonyLifecycleRecord>(
        "SELECT
            r.id,
            r.colony_id,
            c.code AS colony_code,
            r.box_id,
            b.code AS box_code,
            r.action,
            r.occurred_at,
            r.previous_status,
            r.new_status,
            r.reason,
            r.notes,
            r.created_at
         FROM colony_lifecycle_records r
         JOIN colonies c ON c.id = r.colony_id
         LEFT JOIN boxes b ON b.id = r.box_id
         WHERE r.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn change(
    pool: &SqlitePool,
    input: ChangeColonyLifecycle,
) -> Result<ColonyLifecycleRecord, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let action = validate_action(&input.action)?;
    let reason = optional(&input.reason);
    let notes = optional(&input.notes);

    let occurred_at = match optional(&input.occurred_at) {
        Some(value) => value,
        None => sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
            .fetch_one(pool)
            .await?,
    };

    let mut tx = pool.begin().await?;

    let colony: Option<(String, String)> =
        sqlx::query_as("SELECT code, status FROM colonies WHERE id = ?")
            .bind(&colony_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (_, current_status) =
        colony.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;

    let last_transition: Option<String> = sqlx::query_scalar(
        "SELECT MAX(occurred_at) FROM colony_lifecycle_records WHERE colony_id = ?",
    )
    .bind(&colony_id)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(last_transition) = last_transition {
        if occurred_at < last_transition {
            return Err(AppError::Validation(
                "A data não pode ser anterior à última transição do ciclo de vida.".to_owned(),
            ));
        }
    }

    let active_occupancy: Option<(String, String, String)> = sqlx::query_as(
        "SELECT id, box_id, started_at
         FROM colony_box_occupancies
         WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&colony_id)
    .fetch_optional(&mut *tx)
    .await?;

    let new_status = match action.as_str() {
        "loss" => {
            if !ACTIVE_STATUSES.contains(&current_status.as_str()) {
                return Err(AppError::Validation(
                    "Somente uma colônia ativa, fraca ou em recuperação pode ser registrada como perdida."
                        .to_owned(),
                ));
            }
            "lost"
        }
        "deactivate" => {
            if !ACTIVE_STATUSES.contains(&current_status.as_str()) {
                return Err(AppError::Validation(
                    "Somente uma colônia ativa, fraca ou em recuperação pode ser inativada."
                        .to_owned(),
                ));
            }
            "inactive"
        }
        "reactivate" => {
            if current_status != "inactive" {
                return Err(AppError::Validation(
                    "Somente uma colônia inativa pode ser reativada.".to_owned(),
                ));
            }
            if active_occupancy.is_some() {
                return Err(AppError::Validation(
                    "Uma colônia inativa não deve possuir ocupação de caixa ativa.".to_owned(),
                ));
            }
            "active"
        }
        _ => unreachable!(),
    };

    let box_id = active_occupancy
        .as_ref()
        .map(|(_, box_id, _)| box_id.clone());

    if action != "reactivate" {
        if let Some((occupancy_id, _, started_at)) = &active_occupancy {
            if occurred_at < *started_at {
                return Err(AppError::Validation(
                    "A data da baixa não pode ser anterior ao início da ocupação atual."
                        .to_owned(),
                ));
            }

            sqlx::query(
                "UPDATE colony_box_occupancies
                 SET ended_at = ?
                 WHERE id = ? AND ended_at IS NULL",
            )
            .bind(&occurred_at)
            .bind(occupancy_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    sqlx::query(
        "UPDATE colonies
         SET status = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(new_status)
    .bind(&colony_id)
    .execute(&mut *tx)
    .await?;

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO colony_lifecycle_records (
            id, colony_id, box_id, action, occurred_at,
            previous_status, new_status, reason, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&colony_id)
    .bind(box_id)
    .bind(&action)
    .bind(&occurred_at)
    .bind(&current_status)
    .bind(new_status)
    .bind(reason)
    .bind(notes)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    get(pool, &id).await
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<ColonyLifecycleRecord>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
        .bind(&colony_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, ColonyLifecycleRecord>(
        "SELECT
            r.id,
            r.colony_id,
            c.code AS colony_code,
            r.box_id,
            b.code AS box_code,
            r.action,
            r.occurred_at,
            r.previous_status,
            r.new_status,
            r.reason,
            r.notes,
            r.created_at
         FROM colony_lifecycle_records r
         JOIN colonies c ON c.id = r.colony_id
         LEFT JOIN boxes b ON b.id = r.box_id
         WHERE r.colony_id = ?
         ORDER BY r.occurred_at DESC, r.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn timeline_entries(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<TimelineEntry>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    let entry: Option<EntrySnapshot> = sqlx::query_as(
        "SELECT
            id,
            COALESCE(installed_at, created_at) AS occurred_at,
            origin_type,
            origin_notes
         FROM colonies
         WHERE id = ?",
    )
    .bind(&colony_id)
    .fetch_optional(pool)
    .await?;
    let entry = entry.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;

    let origin = match entry.origin_type.as_str() {
        "acquisition" => "Aquisição",
        "multiplication" => "Multiplicação",
        "transfer" => "Transferência",
        "rescue" => "Resgate",
        "authorized_capture" => "Captura autorizada",
        "historical" => "Registro histórico",
        _ => "Outra origem",
    };
    let details = match entry.origin_notes {
        Some(notes) => Some(format!("Origem: {origin}. {notes}")),
        None => Some(format!("Origem: {origin}")),
    };

    let mut entries = vec![TimelineEntry {
        source_type: "colony_entry".to_owned(),
        source_id: entry.id,
        occurred_at: entry.occurred_at,
        title: "Entrada no plantel".to_owned(),
        details,
        box_code: None,
        severity: "info".to_owned(),
    }];

    entries.extend(
        sqlx::query_as::<_, TimelineEntry>(
            "SELECT
                'lifecycle' AS source_type,
                r.id AS source_id,
                r.occurred_at,
                CASE r.action
                    WHEN 'loss' THEN 'Baixa por perda'
                    WHEN 'deactivate' THEN 'Colônia inativada'
                    ELSE 'Colônia reativada'
                END AS title,
                CASE
                    WHEN r.reason IS NOT NULL AND r.notes IS NOT NULL
                        THEN r.reason || ' · ' || r.notes
                    ELSE COALESCE(r.reason, r.notes)
                END AS details,
                b.code AS box_code,
                CASE WHEN r.action = 'loss' THEN 'critical' ELSE 'info' END AS severity
             FROM colony_lifecycle_records r
             LEFT JOIN boxes b ON b.id = r.box_id
             WHERE r.colony_id = ?",
        )
        .bind(&colony_id)
        .fetch_all(pool)
        .await?,
    );

    Ok(entries)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM colony_lifecycle_records")
        .fetch_one(pool)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        repository, timeline,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        pool
    }

    async fn seed(pool: &SqlitePool) -> (String, String) {
        let meliponary = repository::create_meliponary(
            pool,
            CreateMeliponary {
                name: "Meliponário principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let species = repository::create_species(
            pool,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let hive_box = repository::create_box(
            pool,
            CreateHiveBox {
                meliponary_id: meliponary.id.clone(),
                code: "CX-001".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let colony = repository::create_colony(
            pool,
            CreateColony {
                meliponary_id: meliponary.id,
                species_id: species.id,
                code: "JAT-001".into(),
                origin_type: Some("acquisition".into()),
                origin_notes: Some("Entrada inicial".into()),
                installed_at: Some("2026-01-01 09:00:00".into()),
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::place_colony(
            pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: hive_box.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();
        (colony.id, hive_box.id)
    }

    #[tokio::test]
    async fn loss_closes_active_occupancy_and_updates_status() {
        let pool = test_pool().await;
        let (colony_id, box_id) = seed(&pool).await;

        let record = change(
            &pool,
            ChangeColonyLifecycle {
                colony_id: colony_id.clone(),
                action: "loss".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: Some("Colônia perdida".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(record.box_id.as_deref(), Some(box_id.as_str()));
        assert_eq!(record.previous_status, "active");
        assert_eq!(record.new_status, "lost");

        let status: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id = ?")
            .bind(&colony_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let ended_at: String = sqlx::query_scalar(
            "SELECT ended_at FROM colony_box_occupancies WHERE colony_id = ?",
        )
        .bind(&colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(status, "lost");
        assert_eq!(ended_at, "2026-02-01 10:00:00");
    }

    #[tokio::test]
    async fn inactive_colony_can_be_reactivated_without_reopening_old_box() {
        let pool = test_pool().await;
        let (colony_id, _) = seed(&pool).await;

        change(
            &pool,
            ChangeColonyLifecycle {
                colony_id: colony_id.clone(),
                action: "deactivate".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let reactivated = change(
            &pool,
            ChangeColonyLifecycle {
                colony_id: colony_id.clone(),
                action: "reactivate".into(),
                occurred_at: Some("2026-02-10 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(reactivated.previous_status, "inactive");
        assert_eq!(reactivated.new_status, "active");
        assert!(reactivated.box_id.is_none());

        let active_boxes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
        )
        .bind(&colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(active_boxes, 0);
    }

    #[tokio::test]
    async fn lost_colony_cannot_be_placed_back_in_a_box() {
        let pool = test_pool().await;
        let (colony_id, box_id) = seed(&pool).await;

        change(
            &pool,
            ChangeColonyLifecycle {
                colony_id: colony_id.clone(),
                action: "loss".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let result = repository::place_colony(
            &pool,
            PlaceColony {
                colony_id,
                box_id,
                started_at: Some("2026-02-02 10:00:00".into()),
                reason: Some("Tentativa inválida".into()),
                notes: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn timeline_contains_entry_and_lifecycle_transition() {
        let pool = test_pool().await;
        let (colony_id, _) = seed(&pool).await;

        change(
            &pool,
            ChangeColonyLifecycle {
                colony_id: colony_id.clone(),
                action: "loss".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: Some("Perda confirmada".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        let entries = timeline::by_colony(&pool, &colony_id).await.unwrap();
        assert!(entries.iter().any(|entry| entry.source_type == "colony_entry"));
        assert!(entries.iter().any(|entry| entry.source_type == "lifecycle"));
    }
}
