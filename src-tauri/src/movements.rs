use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

const MOVEMENT_TYPES: &[&str] = &["internal_transfer", "external_transfer", "transport"];
const MOVABLE_STATUSES: &[&str] = &["active", "weak", "recovering"];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyMovement {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub movement_type: String,
    pub moved_at: String,
    pub from_meliponary_id: String,
    pub from_meliponary_name: String,
    pub to_meliponary_id: Option<String>,
    pub to_meliponary_name: Option<String>,
    pub from_box_id: Option<String>,
    pub from_box_code: Option<String>,
    pub to_box_id: Option<String>,
    pub to_box_code: Option<String>,
    pub destination: Option<String>,
    pub document_reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovement {
    pub colony_id: String,
    pub movement_type: String,
    pub moved_at: Option<String>,
    pub to_meliponary_id: Option<String>,
    pub to_box_id: Option<String>,
    pub destination: Option<String>,
    pub document_reference: Option<String>,
    pub notes: Option<String>,
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

fn movement_type(value: &str) -> Result<String, AppError> {
    let value = required(value, "Tipo da movimentação")?;
    if MOVEMENT_TYPES.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "Tipo de movimentação inválido.".to_owned(),
        ))
    }
}

async fn get(pool: &SqlitePool, id: &str) -> Result<ColonyMovement, AppError> {
    Ok(sqlx::query_as::<_, ColonyMovement>(
        "SELECT
            m.id,
            m.colony_id,
            c.code AS colony_code,
            m.movement_type,
            m.moved_at,
            m.from_meliponary_id,
            fm.name AS from_meliponary_name,
            m.to_meliponary_id,
            tm.name AS to_meliponary_name,
            m.from_box_id,
            fb.code AS from_box_code,
            m.to_box_id,
            tb.code AS to_box_code,
            m.destination,
            m.document_reference,
            m.notes,
            m.created_at
         FROM colony_movements m
         JOIN colonies c ON c.id = m.colony_id
         JOIN meliponaries fm ON fm.id = m.from_meliponary_id
         LEFT JOIN meliponaries tm ON tm.id = m.to_meliponary_id
         LEFT JOIN boxes fb ON fb.id = m.from_box_id
         LEFT JOIN boxes tb ON tb.id = m.to_box_id
         WHERE m.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

async fn historical_box(
    pool: &SqlitePool,
    colony_id: &str,
    moved_at: &str,
) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT box_id
         FROM colony_box_occupancies
         WHERE colony_id = ?
           AND started_at <= ?
           AND (ended_at IS NULL OR ended_at >= ?)
         ORDER BY started_at DESC
         LIMIT 1",
    )
    .bind(colony_id)
    .bind(moved_at)
    .bind(moved_at)
    .fetch_optional(pool)
    .await?)
}

pub async fn create(pool: &SqlitePool, input: CreateMovement) -> Result<ColonyMovement, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let movement_type = movement_type(&input.movement_type)?;
    let to_meliponary_id = optional(&input.to_meliponary_id);
    let to_box_id = optional(&input.to_box_id);
    let destination = optional(&input.destination);
    let document_reference = optional(&input.document_reference);
    let notes = optional(&input.notes);

    let moved_at = match optional(&input.moved_at) {
        Some(value) => value,
        None => {
            sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
                .fetch_one(pool)
                .await?
        }
    };

    if movement_type == "transport" {
        let colony: Option<(String, String)> =
            sqlx::query_as("SELECT meliponary_id, status FROM colonies WHERE id = ?")
                .bind(&colony_id)
                .fetch_optional(pool)
                .await?;
        let (from_meliponary_id, status) =
            colony.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;

        if !MOVABLE_STATUSES.contains(&status.as_str()) {
            return Err(AppError::Validation(
                "Esta colônia não está disponível para movimentação.".to_owned(),
            ));
        }
        if to_meliponary_id.is_some() || to_box_id.is_some() {
            return Err(AppError::Validation(
                "Transporte temporário não altera meliponário nem caixa de destino.".to_owned(),
            ));
        }
        let destination = destination
            .ok_or_else(|| AppError::Validation("Informe o destino do transporte.".to_owned()))?;
        let from_box_id = historical_box(pool, &colony_id, &moved_at).await?;
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO colony_movements (
                id, colony_id, movement_type, moved_at, from_meliponary_id,
                from_box_id, destination, document_reference, notes
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&colony_id)
        .bind(&movement_type)
        .bind(&moved_at)
        .bind(&from_meliponary_id)
        .bind(from_box_id)
        .bind(destination)
        .bind(document_reference)
        .bind(notes)
        .execute(pool)
        .await?;

        return get(pool, &id).await;
    }

    let mut tx = pool.begin().await?;

    let colony: Option<(String, String, String)> =
        sqlx::query_as("SELECT meliponary_id, status, code FROM colonies WHERE id = ?")
            .bind(&colony_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (from_meliponary_id, status, colony_code) =
        colony.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;

    if !MOVABLE_STATUSES.contains(&status.as_str()) {
        return Err(AppError::Validation(
            "Esta colônia não está disponível para transferência.".to_owned(),
        ));
    }

    let active_occupancy: Option<(String, String, String)> = sqlx::query_as(
        "SELECT id, box_id, started_at
         FROM colony_box_occupancies
         WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&colony_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some((_, _, started_at)) = &active_occupancy {
        if moved_at < *started_at {
            return Err(AppError::Validation(
                "A data da transferência não pode ser anterior ao início da ocupação atual."
                    .to_owned(),
            ));
        }
    }

    let from_box_id = active_occupancy
        .as_ref()
        .map(|(_, box_id, _)| box_id.clone());
    let id = Uuid::new_v4().to_string();

    match movement_type.as_str() {
        "internal_transfer" => {
            let target_meliponary_id = to_meliponary_id.ok_or_else(|| {
                AppError::Validation("Informe o meliponário de destino.".to_owned())
            })?;

            if destination.is_some() {
                return Err(AppError::Validation(
                    "Transferência interna usa um meliponário cadastrado como destino.".to_owned(),
                ));
            }
            if target_meliponary_id == from_meliponary_id {
                return Err(AppError::Validation(
                    "O meliponário de destino precisa ser diferente do atual.".to_owned(),
                ));
            }

            let target_exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meliponaries WHERE id = ?)")
                    .bind(&target_meliponary_id)
                    .fetch_one(&mut *tx)
                    .await?;
            if !target_exists {
                return Err(AppError::NotFound(
                    "Meliponário de destino não encontrado.".to_owned(),
                ));
            }

            let code_conflict: bool = sqlx::query_scalar(
                "SELECT EXISTS(
                    SELECT 1 FROM colonies
                    WHERE meliponary_id = ? AND code = ? AND id <> ?
                 )",
            )
            .bind(&target_meliponary_id)
            .bind(&colony_code)
            .bind(&colony_id)
            .fetch_one(&mut *tx)
            .await?;
            if code_conflict {
                return Err(AppError::Validation(
                    "Já existe uma colônia com este código no meliponário de destino.".to_owned(),
                ));
            }

            if let Some(target_box_id) = &to_box_id {
                let target_box: Option<(String, String)> =
                    sqlx::query_as("SELECT meliponary_id, status FROM boxes WHERE id = ?")
                        .bind(target_box_id)
                        .fetch_optional(&mut *tx)
                        .await?;
                let (box_meliponary_id, box_status) = target_box.ok_or_else(|| {
                    AppError::NotFound("Caixa de destino não encontrada.".to_owned())
                })?;

                if box_meliponary_id != target_meliponary_id {
                    return Err(AppError::Validation(
                        "A caixa de destino precisa pertencer ao meliponário de destino."
                            .to_owned(),
                    ));
                }
                if box_status != "active" {
                    return Err(AppError::Validation(
                        "A caixa de destino precisa estar ativa.".to_owned(),
                    ));
                }

                let occupied: bool = sqlx::query_scalar(
                    "SELECT EXISTS(
                        SELECT 1 FROM colony_box_occupancies
                        WHERE box_id = ? AND ended_at IS NULL
                     )",
                )
                .bind(target_box_id)
                .fetch_one(&mut *tx)
                .await?;
                if occupied {
                    return Err(AppError::Validation(
                        "A caixa de destino já está ocupada.".to_owned(),
                    ));
                }
            }

            if active_occupancy.is_some() {
                sqlx::query(
                    "UPDATE colony_box_occupancies
                     SET ended_at = ?
                     WHERE colony_id = ? AND ended_at IS NULL",
                )
                .bind(&moved_at)
                .bind(&colony_id)
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "UPDATE colonies
                 SET meliponary_id = ?, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(&target_meliponary_id)
            .bind(&colony_id)
            .execute(&mut *tx)
            .await?;

            if let Some(target_box_id) = &to_box_id {
                let occupancy_id = Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO colony_box_occupancies (
                        id, colony_id, box_id, started_at, reason, notes
                     ) VALUES (?, ?, ?, ?, 'Transferência entre meliponários', ?)",
                )
                .bind(occupancy_id)
                .bind(&colony_id)
                .bind(target_box_id)
                .bind(&moved_at)
                .bind(notes.clone())
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "INSERT INTO colony_movements (
                    id, colony_id, movement_type, moved_at,
                    from_meliponary_id, to_meliponary_id,
                    from_box_id, to_box_id, document_reference, notes
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&colony_id)
            .bind(&movement_type)
            .bind(&moved_at)
            .bind(&from_meliponary_id)
            .bind(&target_meliponary_id)
            .bind(from_box_id)
            .bind(to_box_id)
            .bind(document_reference)
            .bind(notes)
            .execute(&mut *tx)
            .await?;
        }
        "external_transfer" => {
            if to_meliponary_id.is_some() || to_box_id.is_some() {
                return Err(AppError::Validation(
                    "Transferência externa usa um destino textual, não um meliponário ou caixa cadastrados."
                        .to_owned(),
                ));
            }
            let destination = destination.ok_or_else(|| {
                AppError::Validation("Informe o destino da transferência.".to_owned())
            })?;

            if active_occupancy.is_some() {
                sqlx::query(
                    "UPDATE colony_box_occupancies
                     SET ended_at = ?
                     WHERE colony_id = ? AND ended_at IS NULL",
                )
                .bind(&moved_at)
                .bind(&colony_id)
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "UPDATE colonies
                 SET status = 'transferred', updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(&colony_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO colony_movements (
                    id, colony_id, movement_type, moved_at,
                    from_meliponary_id, from_box_id,
                    destination, document_reference, notes
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&colony_id)
            .bind(&movement_type)
            .bind(&moved_at)
            .bind(&from_meliponary_id)
            .bind(from_box_id)
            .bind(destination)
            .bind(document_reference)
            .bind(notes)
            .execute(&mut *tx)
            .await?;
        }
        _ => unreachable!(),
    }

    tx.commit().await?;
    get(pool, &id).await
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<ColonyMovement>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
        .bind(&colony_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, ColonyMovement>(
        "SELECT
            m.id,
            m.colony_id,
            c.code AS colony_code,
            m.movement_type,
            m.moved_at,
            m.from_meliponary_id,
            fm.name AS from_meliponary_name,
            m.to_meliponary_id,
            tm.name AS to_meliponary_name,
            m.from_box_id,
            fb.code AS from_box_code,
            m.to_box_id,
            tb.code AS to_box_code,
            m.destination,
            m.document_reference,
            m.notes,
            m.created_at
         FROM colony_movements m
         JOIN colonies c ON c.id = m.colony_id
         JOIN meliponaries fm ON fm.id = m.from_meliponary_id
         LEFT JOIN meliponaries tm ON tm.id = m.to_meliponary_id
         LEFT JOIN boxes fb ON fb.id = m.from_box_id
         LEFT JOIN boxes tb ON tb.id = m.to_box_id
         WHERE m.colony_id = ?
         ORDER BY m.moved_at DESC, m.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM colony_movements")
        .fetch_one(pool)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        history, repository,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    struct Seed {
        source_meliponary_id: String,
        target_meliponary_id: String,
        species_id: String,
        source_box_id: String,
        target_box_id: String,
        colony_id: String,
    }

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        pool
    }

    async fn seed(pool: &SqlitePool) -> Seed {
        let source = repository::create_meliponary(
            pool,
            CreateMeliponary {
                name: "Meliponário A".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let target = repository::create_meliponary(
            pool,
            CreateMeliponary {
                name: "Meliponário B".into(),
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
        let source_box = repository::create_box(
            pool,
            CreateHiveBox {
                meliponary_id: source.id.clone(),
                code: "CX-A1".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let target_box = repository::create_box(
            pool,
            CreateHiveBox {
                meliponary_id: target.id.clone(),
                code: "CX-B1".into(),
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
                meliponary_id: source.id.clone(),
                species_id: species.id.clone(),
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
                box_id: source_box.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        Seed {
            source_meliponary_id: source.id,
            target_meliponary_id: target.id,
            species_id: species.id,
            source_box_id: source_box.id,
            target_box_id: target_box.id,
            colony_id: colony.id,
        }
    }

    #[tokio::test]
    async fn internal_transfer_moves_colony_and_preserves_box_history() {
        let pool = test_pool().await;
        let seed = seed(&pool).await;

        let movement = create(
            &pool,
            CreateMovement {
                colony_id: seed.colony_id.clone(),
                movement_type: "internal_transfer".into(),
                moved_at: Some("2026-02-01 10:00:00".into()),
                to_meliponary_id: Some(seed.target_meliponary_id.clone()),
                to_box_id: Some(seed.target_box_id.clone()),
                destination: None,
                document_reference: Some("REF-001".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            movement.from_box_id.as_deref(),
            Some(seed.source_box_id.as_str())
        );
        assert_eq!(
            movement.to_box_id.as_deref(),
            Some(seed.target_box_id.as_str())
        );

        let colony_state: (String, String) =
            sqlx::query_as("SELECT meliponary_id, status FROM colonies WHERE id = ?")
                .bind(&seed.colony_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(colony_state.0, seed.target_meliponary_id);
        assert_eq!(colony_state.1, "active");

        let active_box: String = sqlx::query_scalar(
            "SELECT box_id FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
        )
        .bind(&seed.colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(active_box, seed.target_box_id);

        let history_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM colony_box_occupancies WHERE colony_id = ?")
                .bind(&seed.colony_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(history_count, 2);
    }

    #[tokio::test]
    async fn occupied_target_box_rolls_back_internal_transfer() {
        let pool = test_pool().await;
        let seed = seed(&pool).await;

        let occupant = repository::create_colony(
            &pool,
            CreateColony {
                meliponary_id: seed.target_meliponary_id.clone(),
                species_id: seed.species_id.clone(),
                code: "JAT-900".into(),
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
            &pool,
            PlaceColony {
                colony_id: occupant.id,
                box_id: seed.target_box_id.clone(),
                started_at: Some("2026-01-10 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let result = create(
            &pool,
            CreateMovement {
                colony_id: seed.colony_id.clone(),
                movement_type: "internal_transfer".into(),
                moved_at: Some("2026-02-01 10:00:00".into()),
                to_meliponary_id: Some(seed.target_meliponary_id),
                to_box_id: Some(seed.target_box_id),
                destination: None,
                document_reference: None,
                notes: None,
            },
        )
        .await;
        assert!(matches!(result, Err(AppError::Validation(_))));

        let current_meliponary: String =
            sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id = ?")
                .bind(&seed.colony_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(current_meliponary, seed.source_meliponary_id);

        let current_box: String = sqlx::query_scalar(
            "SELECT box_id FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
        )
        .bind(&seed.colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(current_box, seed.source_box_id);

        let movement_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM colony_movements")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(movement_count, 0);
    }

    #[tokio::test]
    async fn external_transfer_closes_occupancy_and_marks_colony_transferred() {
        let pool = test_pool().await;
        let seed = seed(&pool).await;

        let movement = create(
            &pool,
            CreateMovement {
                colony_id: seed.colony_id.clone(),
                movement_type: "external_transfer".into(),
                moved_at: Some("2026-03-01 10:00:00".into()),
                to_meliponary_id: None,
                to_box_id: None,
                destination: Some("Meliponário parceiro".into()),
                document_reference: Some("DOC-42".into()),
                notes: Some("Transferência definitiva".into()),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            movement.destination.as_deref(),
            Some("Meliponário parceiro")
        );

        let status: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id = ?")
            .bind(&seed.colony_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "transferred");

        let active_occupancies: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
        )
        .bind(&seed.colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(active_occupancies, 0);
    }

    #[tokio::test]
    async fn transport_records_history_without_changing_colony_state() {
        let pool = test_pool().await;
        let seed = seed(&pool).await;

        let movement = create(
            &pool,
            CreateMovement {
                colony_id: seed.colony_id.clone(),
                movement_type: "transport".into(),
                moved_at: Some("2026-01-20 08:00:00".into()),
                to_meliponary_id: None,
                to_box_id: None,
                destination: Some("Exposição municipal".into()),
                document_reference: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            movement.from_box_id.as_deref(),
            Some(seed.source_box_id.as_str())
        );

        let colony_state: (String, String) =
            sqlx::query_as("SELECT meliponary_id, status FROM colonies WHERE id = ?")
                .bind(&seed.colony_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(colony_state.0, seed.source_meliponary_id);
        assert_eq!(colony_state.1, "active");

        let current_box: String = sqlx::query_scalar(
            "SELECT box_id FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
        )
        .bind(&seed.colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(current_box, seed.source_box_id);
    }

    #[tokio::test]
    async fn movement_appears_in_colony_timeline() {
        let pool = test_pool().await;
        let seed = seed(&pool).await;

        create(
            &pool,
            CreateMovement {
                colony_id: seed.colony_id.clone(),
                movement_type: "transport".into(),
                moved_at: Some("2026-01-20 08:00:00".into()),
                to_meliponary_id: None,
                to_box_id: None,
                destination: Some("Feira técnica".into()),
                document_reference: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let timeline = history::timeline_by_colony(&pool, &seed.colony_id)
            .await
            .unwrap();
        assert!(timeline.iter().any(|entry| entry.source_type == "movement"));
    }
}
