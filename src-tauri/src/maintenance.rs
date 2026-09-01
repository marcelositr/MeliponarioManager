use crate::{history::TimelineEntry, repository::AppError};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

const MAINTENANCE_TYPES: &[&str] = &[
    "cleaning",
    "repair",
    "painting",
    "waterproofing",
    "roof",
    "entrance",
    "internal_structure",
    "inspection",
    "other",
];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BoxMaintenance {
    pub id: String,
    pub box_id: String,
    pub box_code: String,
    pub colony_id: Option<String>,
    pub colony_code: Option<String>,
    pub maintained_at: String,
    pub maintenance_type: String,
    pub description: Option<String>,
    pub performed_by: Option<String>,
    pub cost: Option<f64>,
    pub next_maintenance_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoxMaintenance {
    pub box_id: String,
    pub maintained_at: Option<String>,
    pub maintenance_type: String,
    pub description: Option<String>,
    pub performed_by: Option<String>,
    pub cost: Option<f64>,
    pub next_maintenance_at: Option<String>,
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

fn validate_cost(cost: Option<f64>) -> Result<Option<f64>, AppError> {
    match cost {
        Some(value) if !value.is_finite() || value < 0.0 => Err(AppError::Validation(
            "O custo da manutenção precisa ser um valor válido e não negativo.".to_owned(),
        )),
        _ => Ok(cost),
    }
}

async fn box_exists(pool: &SqlitePool, box_id: &str) -> Result<bool, AppError> {
    Ok(
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM boxes WHERE id = ?)")
            .bind(box_id)
            .fetch_one(pool)
            .await?,
    )
}

async fn get(pool: &SqlitePool, id: &str) -> Result<BoxMaintenance, AppError> {
    Ok(sqlx::query_as::<_, BoxMaintenance>(
        "SELECT m.id, m.box_id, b.code AS box_code,
                m.colony_id, c.code AS colony_code,
                m.maintained_at, m.maintenance_type, m.description,
                m.performed_by, m.cost, m.next_maintenance_at, m.created_at
         FROM box_maintenance_records m
         JOIN boxes b ON b.id = m.box_id
         LEFT JOIN colonies c ON c.id = m.colony_id
         WHERE m.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

async fn get_tx(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<BoxMaintenance, AppError> {
    Ok(sqlx::query_as::<_, BoxMaintenance>(
        "SELECT m.id, m.box_id, b.code AS box_code,
                m.colony_id, c.code AS colony_code,
                m.maintained_at, m.maintenance_type, m.description,
                m.performed_by, m.cost, m.next_maintenance_at, m.created_at
         FROM box_maintenance_records m
         JOIN boxes b ON b.id = m.box_id
         LEFT JOIN colonies c ON c.id = m.colony_id
         WHERE m.id = ?",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await?)
}

pub(crate) async fn create_tx(
    tx: &mut Transaction<'_, Sqlite>,
    input: CreateBoxMaintenance,
) -> Result<BoxMaintenance, AppError> {
    let box_id = required(&input.box_id, "Caixa")?;
    let maintenance_type = required(&input.maintenance_type, "Tipo de manutenção")?;

    if !MAINTENANCE_TYPES.contains(&maintenance_type.as_str()) {
        return Err(AppError::Validation(
            "Tipo de manutenção inválido.".to_owned(),
        ));
    }

    let cost = validate_cost(input.cost)?;

    let box_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM boxes WHERE id = ?)")
        .bind(&box_id)
        .fetch_one(&mut **tx)
        .await?;
    if !box_exists {
        return Err(AppError::NotFound("Caixa não encontrada.".to_owned()));
    }

    let maintained_at = match optional(&input.maintained_at) {
        Some(value) => value,
        None => {
            sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
                .fetch_one(&mut **tx)
                .await?
        }
    };

    let next_maintenance_at = optional(&input.next_maintenance_at);
    if next_maintenance_at
        .as_deref()
        .is_some_and(|next| next < maintained_at.as_str())
    {
        return Err(AppError::Validation(
            "A próxima manutenção não pode ser anterior à manutenção registrada.".to_owned(),
        ));
    }

    let colony_id: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT colony_id
         FROM colony_box_occupancies
         WHERE box_id = ?
           AND started_at <= ?
           AND (ended_at IS NULL OR ended_at >= ?)
         ORDER BY started_at DESC
         LIMIT 1",
    )
    .bind(&box_id)
    .bind(&maintained_at)
    .bind(&maintained_at)
    .fetch_optional(&mut **tx)
    .await?;
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO box_maintenance_records (
            id, box_id, colony_id, maintained_at, maintenance_type,
            description, performed_by, cost, next_maintenance_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&box_id)
    .bind(colony_id)
    .bind(maintained_at)
    .bind(maintenance_type)
    .bind(optional(&input.description))
    .bind(optional(&input.performed_by))
    .bind(cost)
    .bind(next_maintenance_at)
    .execute(&mut **tx)
    .await?;

    get_tx(tx, &id).await
}

pub async fn create(
    pool: &SqlitePool,
    input: CreateBoxMaintenance,
) -> Result<BoxMaintenance, AppError> {
    let mut tx = pool.begin().await?;
    let record = create_tx(&mut tx, input).await?;
    tx.commit().await?;
    Ok(record)
}

pub async fn list_by_box(pool: &SqlitePool, box_id: &str) -> Result<Vec<BoxMaintenance>, AppError> {
    let box_id = required(box_id, "Caixa")?;
    if !box_exists(pool, &box_id).await? {
        return Err(AppError::NotFound("Caixa não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, BoxMaintenance>(
        "SELECT m.id, m.box_id, b.code AS box_code,
                m.colony_id, c.code AS colony_code,
                m.maintained_at, m.maintenance_type, m.description,
                m.performed_by, m.cost, m.next_maintenance_at, m.created_at
         FROM box_maintenance_records m
         JOIN boxes b ON b.id = m.box_id
         LEFT JOIN colonies c ON c.id = m.colony_id
         WHERE m.box_id = ?
         ORDER BY m.maintained_at DESC, m.created_at DESC",
    )
    .bind(box_id)
    .fetch_all(pool)
    .await?)
}

pub async fn timeline_entries_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<TimelineEntry>, AppError> {
    Ok(sqlx::query_as::<_, TimelineEntry>(
        "SELECT
            'box_maintenance' AS source_type,
            m.id AS source_id,
            m.maintained_at AS occurred_at,
            CASE m.maintenance_type
                WHEN 'cleaning' THEN 'Manutenção da caixa: limpeza'
                WHEN 'repair' THEN 'Manutenção da caixa: reparo'
                WHEN 'painting' THEN 'Manutenção da caixa: pintura'
                WHEN 'waterproofing' THEN 'Manutenção da caixa: impermeabilização'
                WHEN 'roof' THEN 'Manutenção da caixa: cobertura'
                WHEN 'entrance' THEN 'Manutenção da caixa: entrada'
                WHEN 'internal_structure' THEN 'Manutenção da caixa: estrutura interna'
                WHEN 'inspection' THEN 'Revisão da caixa'
                ELSE 'Manutenção da caixa'
            END AS title,
            m.description AS details,
            b.code AS box_code,
            'info' AS severity
         FROM box_maintenance_records m
         JOIN boxes b ON b.id = m.box_id
         WHERE m.colony_id = ?",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM box_maintenance_records")
            .fetch_one(pool)
            .await?,
    )
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

    async fn seed(pool: &SqlitePool) -> (String, String, String) {
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

        let box_one = repository::create_box(
            pool,
            CreateHiveBox {
                meliponary_id: meliponary.id.clone(),
                code: "CX-001".into(),
                model: None,
                material: Some("Madeira".into()),
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let box_two = repository::create_box(
            pool,
            CreateHiveBox {
                meliponary_id: meliponary.id.clone(),
                code: "CX-002".into(),
                model: None,
                material: Some("Madeira".into()),
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
                origin_type: None,
                origin_notes: None,
                installed_at: None,
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
                box_id: box_one.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        repository::place_colony(
            pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: box_two.id.clone(),
                started_at: Some("2026-02-01 09:00:00".into()),
                reason: Some("Troca de caixa".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        (colony.id, box_one.id, box_two.id)
    }

    #[tokio::test]
    async fn retrospective_maintenance_keeps_colony_context_from_that_date() {
        let pool = test_pool().await;
        let (colony_id, box_one_id, _) = seed(&pool).await;

        let maintenance = create(
            &pool,
            CreateBoxMaintenance {
                box_id: box_one_id,
                maintained_at: Some("2026-01-15 10:00:00".into()),
                maintenance_type: "repair".into(),
                description: Some("Ajuste da tampa".into()),
                performed_by: None,
                cost: Some(12.5),
                next_maintenance_at: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(maintenance.colony_id.as_deref(), Some(colony_id.as_str()));
        assert_eq!(maintenance.colony_code.as_deref(), Some("JAT-001"));
    }

    #[tokio::test]
    async fn maintenance_on_empty_box_does_not_invent_colony_context() {
        let pool = test_pool().await;
        let (_, box_one_id, _) = seed(&pool).await;

        let maintenance = create(
            &pool,
            CreateBoxMaintenance {
                box_id: box_one_id,
                maintained_at: Some("2026-03-01 10:00:00".into()),
                maintenance_type: "cleaning".into(),
                description: None,
                performed_by: None,
                cost: None,
                next_maintenance_at: None,
            },
        )
        .await
        .unwrap();

        assert!(maintenance.colony_id.is_none());
        assert!(maintenance.colony_code.is_none());
    }

    #[tokio::test]
    async fn maintenance_rejects_invalid_type_and_negative_cost() {
        let pool = test_pool().await;
        let (_, box_one_id, _) = seed(&pool).await;

        let invalid_type = create(
            &pool,
            CreateBoxMaintenance {
                box_id: box_one_id.clone(),
                maintained_at: None,
                maintenance_type: "teleport".into(),
                description: None,
                performed_by: None,
                cost: None,
                next_maintenance_at: None,
            },
        )
        .await;
        assert!(matches!(invalid_type, Err(AppError::Validation(_))));

        let invalid_cost = create(
            &pool,
            CreateBoxMaintenance {
                box_id: box_one_id,
                maintained_at: None,
                maintenance_type: "repair".into(),
                description: None,
                performed_by: None,
                cost: Some(-1.0),
                next_maintenance_at: None,
            },
        )
        .await;
        assert!(matches!(invalid_cost, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn occupied_box_maintenance_appears_in_colony_timeline() {
        let pool = test_pool().await;
        let (colony_id, box_one_id, _) = seed(&pool).await;

        create(
            &pool,
            CreateBoxMaintenance {
                box_id: box_one_id,
                maintained_at: Some("2026-01-20 10:00:00".into()),
                maintenance_type: "painting".into(),
                description: Some("Proteção externa".into()),
                performed_by: Some("Marcelo".into()),
                cost: None,
                next_maintenance_at: Some("2026-07-20 10:00:00".into()),
            },
        )
        .await
        .unwrap();

        let entries = timeline::by_colony(&pool, &colony_id).await.unwrap();
        assert!(entries
            .iter()
            .any(|entry| entry.source_type == "box_maintenance"));
    }
}
