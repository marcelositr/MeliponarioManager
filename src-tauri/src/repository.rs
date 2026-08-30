use crate::{
    audit,
    domain::{
        Colony, ColonyBoxOccupancy, CoreSummary, CreateColony, CreateHiveBox, CreateMeliponary,
        CreateSpecies, HiveBox, Meliponary, PlaceColony, Species,
    },
    time,
};
use serde_json::json;
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("Não foi possível acessar os dados locais.")]
    Database(#[from] sqlx::Error),
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

pub async fn core_summary(pool: &SqlitePool) -> Result<CoreSummary, AppError> {
    Ok(sqlx::query_as::<_, CoreSummary>(
        "SELECT
            (SELECT COUNT(*) FROM meliponaries) AS meliponaries,
            (SELECT COUNT(*) FROM species) AS species,
            (SELECT COUNT(*) FROM colonies) AS colonies,
            (SELECT COUNT(*) FROM boxes) AS boxes",
    )
    .fetch_one(pool)
    .await?)
}

pub async fn create_meliponary(
    pool: &SqlitePool,
    input: CreateMeliponary,
) -> Result<Meliponary, AppError> {
    let id = Uuid::new_v4().to_string();
    let name = required(&input.name, "Nome do meliponário")?;

    sqlx::query(
        "INSERT INTO meliponaries (id, name, responsible_name, location, notes)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(optional(&input.responsible_name))
    .bind(optional(&input.location))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    Ok(sqlx::query_as::<_, Meliponary>(
        "SELECT id, name, responsible_name, location, notes, archived_at, archive_reason, created_at
         FROM meliponaries WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_meliponaries(pool: &SqlitePool) -> Result<Vec<Meliponary>, AppError> {
    Ok(sqlx::query_as::<_, Meliponary>(
        "SELECT id, name, responsible_name, location, notes, archived_at, archive_reason, created_at
         FROM meliponaries
         ORDER BY archived_at IS NOT NULL, name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_species(pool: &SqlitePool, input: CreateSpecies) -> Result<Species, AppError> {
    let id = Uuid::new_v4().to_string();
    let common_name = required(&input.common_name, "Nome popular")?;

    sqlx::query(
        "INSERT INTO species (id, common_name, scientific_name, genus, notes)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(common_name)
    .bind(optional(&input.scientific_name))
    .bind(optional(&input.genus))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    Ok(sqlx::query_as::<_, Species>(
        "SELECT id, common_name, scientific_name, genus, notes, archived_at, archive_reason, created_at
         FROM species WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_species(pool: &SqlitePool) -> Result<Vec<Species>, AppError> {
    Ok(sqlx::query_as::<_, Species>(
        "SELECT id, common_name, scientific_name, genus, notes, archived_at, archive_reason, created_at
         FROM species
         ORDER BY archived_at IS NOT NULL, common_name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_box(pool: &SqlitePool, input: CreateHiveBox) -> Result<HiveBox, AppError> {
    let id = Uuid::new_v4().to_string();
    let meliponary_id = required(&input.meliponary_id, "Meliponário")?;
    let code = required(&input.code, "Identificação da caixa")?;

    let meliponary_archived: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM meliponaries WHERE id = ?")
            .bind(&meliponary_id)
            .fetch_optional(pool)
            .await?;
    match meliponary_archived {
        None => return Err(AppError::NotFound("Meliponário não encontrado.".to_owned())),
        Some(Some(_)) => {
            return Err(AppError::Validation(
                "Um meliponário arquivado não pode receber novas caixas.".to_owned(),
            ))
        }
        Some(None) => {}
    }

    sqlx::query(
        "INSERT INTO boxes (id, meliponary_id, code, model, material, location_note, notes)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(meliponary_id)
    .bind(code)
    .bind(optional(&input.model))
    .bind(optional(&input.material))
    .bind(optional(&input.location_note))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    get_box(pool, &id).await
}

async fn get_box(pool: &SqlitePool, id: &str) -> Result<HiveBox, AppError> {
    Ok(sqlx::query_as::<_, HiveBox>(
        "SELECT b.id, b.meliponary_id, b.code, b.model, b.material, b.location_note,
                b.status, b.notes, c.code AS current_colony_code, b.created_at
         FROM boxes b
         LEFT JOIN colony_box_occupancies o ON o.box_id = b.id AND o.ended_at IS NULL
         LEFT JOIN colonies c ON c.id = o.colony_id
         WHERE b.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_boxes(pool: &SqlitePool) -> Result<Vec<HiveBox>, AppError> {
    Ok(sqlx::query_as::<_, HiveBox>(
        "SELECT b.id, b.meliponary_id, b.code, b.model, b.material, b.location_note,
                b.status, b.notes, c.code AS current_colony_code, b.created_at
         FROM boxes b
         LEFT JOIN colony_box_occupancies o ON o.box_id = b.id AND o.ended_at IS NULL
         LEFT JOIN colonies c ON c.id = o.colony_id
         ORDER BY b.code COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_colony(pool: &SqlitePool, input: CreateColony) -> Result<Colony, AppError> {
    let id = Uuid::new_v4().to_string();
    let meliponary_id = required(&input.meliponary_id, "Meliponário")?;
    let species_id = required(&input.species_id, "Espécie")?;
    let code = required(&input.code, "Identificação da colônia")?;
    let origin_type = optional(&input.origin_type).unwrap_or_else(|| "historical".to_owned());

    let meliponary_archived: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM meliponaries WHERE id = ?")
            .bind(&meliponary_id)
            .fetch_optional(pool)
            .await?;
    match meliponary_archived {
        None => return Err(AppError::NotFound("Meliponário não encontrado.".to_owned())),
        Some(Some(_)) => {
            return Err(AppError::Validation(
                "Um meliponário arquivado não pode receber novas colônias.".to_owned(),
            ))
        }
        Some(None) => {}
    }

    let species_archived: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM species WHERE id = ?")
            .bind(&species_id)
            .fetch_optional(pool)
            .await?;
    match species_archived {
        None => return Err(AppError::NotFound("Espécie não encontrada.".to_owned())),
        Some(Some(_)) => {
            return Err(AppError::Validation(
                "Uma espécie arquivada não pode ser usada em uma nova colônia.".to_owned(),
            ))
        }
        Some(None) => {}
    }

    sqlx::query(
        "INSERT INTO colonies (
            id, meliponary_id, species_id, code, origin_type, origin_notes,
            installed_at, mother_colony_id, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(meliponary_id)
    .bind(species_id)
    .bind(code)
    .bind(origin_type)
    .bind(optional(&input.origin_notes))
    .bind(optional(&input.installed_at))
    .bind(optional(&input.mother_colony_id))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    get_colony(pool, &id).await
}

async fn get_colony(pool: &SqlitePool, id: &str) -> Result<Colony, AppError> {
    Ok(sqlx::query_as::<_, Colony>(
        "SELECT c.id, c.meliponary_id, c.species_id, c.code, c.origin_type,
                c.origin_notes, c.installed_at, c.status, c.mother_colony_id, c.notes,
                b.code AS current_box_code, c.created_at
         FROM colonies c
         LEFT JOIN colony_box_occupancies o ON o.colony_id = c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id = o.box_id
         WHERE c.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_colonies(pool: &SqlitePool) -> Result<Vec<Colony>, AppError> {
    Ok(sqlx::query_as::<_, Colony>(
        "SELECT c.id, c.meliponary_id, c.species_id, c.code, c.origin_type,
                c.origin_notes, c.installed_at, c.status, c.mother_colony_id, c.notes,
                b.code AS current_box_code, c.created_at
         FROM colonies c
         LEFT JOIN colony_box_occupancies o ON o.colony_id = c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id = o.box_id
         ORDER BY c.code COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn place_colony(
    pool: &SqlitePool,
    input: PlaceColony,
) -> Result<ColonyBoxOccupancy, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let box_id = required(&input.box_id, "Caixa")?;
    let started_at = match optional(&input.started_at) {
        Some(value) => time::normalize(&value, false)?,
        None => time::local_now(pool).await?,
    };
    let mut tx = pool.begin().await?;

    let colony_context: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT c.meliponary_id, m.archived_at
         FROM colonies c
         JOIN meliponaries m ON m.id = c.meliponary_id
         WHERE c.id = ?",
    )
    .bind(&colony_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (colony_meliponary, colony_meliponary_archived) =
        colony_context.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;
    if colony_meliponary_archived.is_some() {
        return Err(AppError::Validation(
            "O meliponário da colônia está arquivado e não aceita nova ocupação.".to_owned(),
        ));
    }

    let target_box: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT b.meliponary_id, b.status, m.archived_at
         FROM boxes b
         JOIN meliponaries m ON m.id = b.meliponary_id
         WHERE b.id = ?",
    )
    .bind(&box_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (box_meliponary, box_status, box_meliponary_archived) =
        target_box.ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))?;

    if box_meliponary_archived.is_some() {
        return Err(AppError::Validation(
            "O meliponário da caixa está arquivado.".to_owned(),
        ));
    }
    if box_status != "active" {
        return Err(AppError::Validation(
            "Somente uma caixa ativa pode receber uma nova ocupação.".to_owned(),
        ));
    }
    if colony_meliponary != box_meliponary {
        return Err(AppError::Validation(
            "A colônia e a caixa precisam pertencer ao mesmo meliponário.".to_owned(),
        ));
    }

    let current_occupancy: Option<(String, String, String)> = sqlx::query_as(
        "SELECT id, box_id, started_at FROM colony_box_occupancies
         WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&colony_id)
    .fetch_optional(&mut *tx)
    .await?;

    if current_occupancy
        .as_ref()
        .is_some_and(|(_, current_box, _)| current_box == &box_id)
    {
        return Err(AppError::Validation(
            "A colônia já está registrada nesta caixa.".to_owned(),
        ));
    }

    let target_occupant: Option<String> = sqlx::query_scalar(
        "SELECT colony_id FROM colony_box_occupancies
         WHERE box_id = ? AND ended_at IS NULL",
    )
    .bind(&box_id)
    .fetch_optional(&mut *tx)
    .await?;
    if target_occupant.is_some() {
        return Err(AppError::Validation(
            "A caixa já está ocupada por outra colônia.".to_owned(),
        ));
    }

    let before = current_occupancy
        .as_ref()
        .map(|(id, current_box, current_started_at)| {
            json!({
                "occupancy_id": id,
                "colony_id": colony_id,
                "box_id": current_box,
                "started_at": current_started_at,
                "ended_at": null
            })
        });

    if let Some((occupancy_id, _, current_started_at)) = &current_occupancy {
        if started_at < *current_started_at {
            return Err(AppError::Validation(
                "A data da troca não pode ser anterior ao início da ocupação atual.".to_owned(),
            ));
        }
        sqlx::query(
            "UPDATE colony_box_occupancies
             SET ended_at = ?
             WHERE id = ? AND ended_at IS NULL",
        )
        .bind(&started_at)
        .bind(occupancy_id)
        .execute(&mut *tx)
        .await?;
    }

    let occupancy_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO colony_box_occupancies
            (id, colony_id, box_id, started_at, reason, notes)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&occupancy_id)
    .bind(&colony_id)
    .bind(&box_id)
    .bind(&started_at)
    .bind(optional(&input.reason))
    .bind(optional(&input.notes))
    .execute(&mut *tx)
    .await?;

    let audit_reason =
        optional(&input.reason).unwrap_or_else(|| "Alteração de ocupação de caixa".to_owned());
    audit::record_tx(
        &mut tx,
        "box_occupancy",
        &occupancy_id,
        "place",
        &audit_reason,
        before,
        Some(json!({
            "occupancy_id": occupancy_id,
            "colony_id": colony_id,
            "box_id": box_id,
            "started_at": started_at,
            "ended_at": null
        })),
    )
    .await?;

    tx.commit().await?;

    Ok(sqlx::query_as::<_, ColonyBoxOccupancy>(
        "SELECT id, colony_id, box_id, started_at, ended_at, reason, notes, corrected_at
         FROM colony_box_occupancies WHERE id = ?",
    )
    .bind(occupancy_id)
    .fetch_one(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn database_error_display_is_safe_for_ipc() {
        let error = AppError::Database(sqlx::Error::RowNotFound);
        assert_eq!(error.to_string(), "Não foi possível acessar os dados locais.");
    }

    #[test]
    fn domain_error_display_preserves_user_facing_messages() {
        assert_eq!(
            AppError::Validation("Valor inválido.".to_owned()).to_string(),
            "Valor inválido."
        );
        assert_eq!(
            AppError::NotFound("Registro não encontrado.".to_owned()).to_string(),
            "Registro não encontrado."
        );
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

    async fn seed(pool: &SqlitePool) -> (Meliponary, Species, HiveBox, HiveBox, Colony) {
        let meliponary = create_meliponary(
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

        let species = create_species(
            pool,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: Some("Tetragonisca angustula".into()),
                genus: Some("Tetragonisca".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        let box_one = create_box(
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

        let box_two = create_box(
            pool,
            CreateHiveBox {
                meliponary_id: meliponary.id.clone(),
                code: "CX-002".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let colony = create_colony(
            pool,
            CreateColony {
                meliponary_id: meliponary.id.clone(),
                species_id: species.id.clone(),
                code: "JAT-001".into(),
                origin_type: Some("historical".into()),
                origin_notes: None,
                installed_at: None,
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        (meliponary, species, box_one, box_two, colony)
    }

    #[tokio::test]
    async fn moving_colony_preserves_box_history() {
        let pool = test_pool().await;
        let (_, _, box_one, box_two, colony) = seed(&pool).await;

        place_colony(
            &pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: box_one.id.clone(),
                started_at: Some("2026-01-01 10:00:00".into()),
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        place_colony(
            &pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: box_two.id.clone(),
                started_at: Some("2026-02-01 10:00:00".into()),
                reason: Some("Troca de caixa".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        let history_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM colony_box_occupancies WHERE colony_id = ?")
                .bind(&colony.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(history_count, 2);

        let colonies = list_colonies(&pool).await.unwrap();
        assert_eq!(colonies[0].current_box_code.as_deref(), Some("CX-002"));
    }

    #[tokio::test]
    async fn colony_cannot_be_placed_in_box_from_another_meliponary() {
        let pool = test_pool().await;
        let (_, _, _, _, colony) = seed(&pool).await;

        let other = create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Outro meliponário".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let other_box = create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: other.id,
                code: "CX-100".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let result = place_colony(
            &pool,
            PlaceColony {
                colony_id: colony.id,
                box_id: other_box.id,
                started_at: None,
                reason: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn nonactive_box_is_rejected_by_repository() {
        for status in ["maintenance", "retired"] {
            let pool = test_pool().await;
            let (_, _, box_one, _, colony) = seed(&pool).await;
            sqlx::query("UPDATE boxes SET status = ? WHERE id = ?")
                .bind(status)
                .bind(&box_one.id)
                .execute(&pool)
                .await
                .unwrap();

            let result = place_colony(
                &pool,
                PlaceColony {
                    colony_id: colony.id,
                    box_id: box_one.id,
                    started_at: Some("2026-01-01 10:00:00".into()),
                    reason: None,
                    notes: None,
                },
            )
            .await;

            assert!(matches!(result, Err(AppError::Validation(_))));
        }
    }

    #[tokio::test]
    async fn archived_master_data_cannot_receive_new_operational_children() {
        let pool = test_pool().await;
        let mel = create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Arquivado".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let species = create_species(
            &pool,
            CreateSpecies {
                common_name: "Mandaçaia".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        sqlx::query(
            "UPDATE meliponaries SET archived_at = datetime('now','localtime') WHERE id = ?",
        )
        .bind(&mel.id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("UPDATE species SET archived_at = datetime('now','localtime') WHERE id = ?")
            .bind(&species.id)
            .execute(&pool)
            .await
            .unwrap();

        assert!(create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: mel.id.clone(),
                code: "CX-X".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .is_err());

        let active_mel = create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Ativo".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        assert!(create_colony(
            &pool,
            CreateColony {
                meliponary_id: active_mel.id,
                species_id: species.id,
                code: "MAN-001".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: None,
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .is_err());
    }
}
