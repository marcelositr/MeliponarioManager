use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub inspected_at: String,
    pub strength: String,
    pub queen_present: Option<bool>,
    pub laying_status: Option<String>,
    pub food_reserves: Option<String>,
    pub brood_status: Option<String>,
    pub pests_notes: Option<String>,
    pub observations: Option<String>,
    pub actions_taken: Option<String>,
    pub next_inspection_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInspection {
    pub colony_id: String,
    pub inspected_at: Option<String>,
    pub strength: Option<String>,
    pub queen_present: Option<bool>,
    pub laying_status: Option<String>,
    pub food_reserves: Option<String>,
    pub brood_status: Option<String>,
    pub pests_notes: Option<String>,
    pub observations: Option<String>,
    pub actions_taken: Option<String>,
    pub next_inspection_at: Option<String>,
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

fn strength(value: &Option<String>) -> Result<String, AppError> {
    let value = optional(value).unwrap_or_else(|| "unknown".to_owned());
    match value.as_str() {
        "strong" | "medium" | "weak" | "unknown" => Ok(value),
        _ => Err(AppError::Validation(
            "Força da colônia inválida. Use strong, medium, weak ou unknown.".to_owned(),
        )),
    }
}

async fn get(pool: &SqlitePool, id: &str) -> Result<Inspection, AppError> {
    Ok(sqlx::query_as::<_, Inspection>(
        "SELECT i.id, i.colony_id, c.code AS colony_code, i.box_id, b.code AS box_code,
                i.inspected_at, i.strength, i.queen_present, i.laying_status,
                i.food_reserves, i.brood_status, i.pests_notes, i.observations,
                i.actions_taken, i.next_inspection_at, i.created_at
         FROM inspections i
         JOIN colonies c ON c.id = i.colony_id
         LEFT JOIN boxes b ON b.id = i.box_id
         WHERE i.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn create(pool: &SqlitePool, input: CreateInspection) -> Result<Inspection, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let inspection_strength = strength(&input.strength)?;

    let colony_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)",
    )
    .bind(&colony_id)
    .fetch_one(pool)
    .await?;

    if !colony_exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    let box_id: Option<String> = sqlx::query_scalar(
        "SELECT box_id
         FROM colony_box_occupancies
         WHERE colony_id = ? AND ended_at IS NULL
         LIMIT 1",
    )
    .bind(&colony_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    let inspected_at = match optional(&input.inspected_at) {
        Some(value) => value,
        None => sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
            .fetch_one(pool)
            .await?,
    };

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO inspections (
            id, colony_id, box_id, inspected_at, strength, queen_present,
            laying_status, food_reserves, brood_status, pests_notes, observations,
            actions_taken, next_inspection_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&colony_id)
    .bind(box_id)
    .bind(inspected_at)
    .bind(inspection_strength)
    .bind(input.queen_present)
    .bind(optional(&input.laying_status))
    .bind(optional(&input.food_reserves))
    .bind(optional(&input.brood_status))
    .bind(optional(&input.pests_notes))
    .bind(optional(&input.observations))
    .bind(optional(&input.actions_taken))
    .bind(optional(&input.next_inspection_at))
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<Inspection>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;

    Ok(sqlx::query_as::<_, Inspection>(
        "SELECT i.id, i.colony_id, c.code AS colony_code, i.box_id, b.code AS box_code,
                i.inspected_at, i.strength, i.queen_present, i.laying_status,
                i.food_reserves, i.brood_status, i.pests_notes, i.observations,
                i.actions_taken, i.next_inspection_at, i.created_at
         FROM inspections i
         JOIN colonies c ON c.id = i.colony_id
         LEFT JOIN boxes b ON b.id = i.box_id
         WHERE i.colony_id = ?
         ORDER BY i.inspected_at DESC, i.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM inspections")
        .fetch_one(pool)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony}, repository};
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

    async fn seeded_colony(pool: &SqlitePool) -> (String, String) {
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
                box_id: hive_box.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        (colony.id, hive_box.id)
    }

    #[tokio::test]
    async fn inspection_keeps_box_context_from_that_moment() {
        let pool = test_pool().await;
        let (colony_id, box_id) = seeded_colony(&pool).await;

        let inspection = create(
            &pool,
            CreateInspection {
                colony_id,
                inspected_at: Some("2026-02-01 10:00:00".into()),
                strength: Some("strong".into()),
                queen_present: Some(true),
                laying_status: Some("Postura normal".into()),
                food_reserves: Some("Boas reservas".into()),
                brood_status: Some("Crias regulares".into()),
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(inspection.box_id.as_deref(), Some(box_id.as_str()));
        assert_eq!(inspection.box_code.as_deref(), Some("CX-001"));
        assert_eq!(inspection.strength, "strong");
    }

    #[tokio::test]
    async fn inspection_rejects_unknown_colony() {
        let pool = test_pool().await;

        let result = create(
            &pool,
            CreateInspection {
                colony_id: Uuid::new_v4().to_string(),
                inspected_at: None,
                strength: None,
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn inspection_rejects_invalid_strength() {
        let pool = test_pool().await;
        let (colony_id, _) = seeded_colony(&pool).await;

        let result = create(
            &pool,
            CreateInspection {
                colony_id,
                inspected_at: None,
                strength: Some("gigantic".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
