use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Feeding {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub fed_at: String,
    pub food_type: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub response_notes: Option<String>,
    pub notes: Option<String>,
    pub next_feeding_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeeding {
    pub colony_id: String,
    pub fed_at: Option<String>,
    pub food_type: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub response_notes: Option<String>,
    pub notes: Option<String>,
    pub next_feeding_at: Option<String>,
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

fn validate_quantity(
    quantity: Option<f64>,
    unit: &Option<String>,
) -> Result<Option<String>, AppError> {
    let unit = optional(unit);

    match (quantity, unit) {
        (None, None) => Ok(None),
        (Some(value), Some(unit)) if value > 0.0 && value.is_finite() => Ok(Some(unit)),
        (Some(_), None) => Err(AppError::Validation(
            "Informe a unidade quando registrar uma quantidade.".to_owned(),
        )),
        (None, Some(_)) => Err(AppError::Validation(
            "Informe a quantidade quando registrar uma unidade.".to_owned(),
        )),
        (Some(_), Some(_)) => Err(AppError::Validation(
            "A quantidade precisa ser maior que zero.".to_owned(),
        )),
    }
}

async fn colony_exists(pool: &SqlitePool, colony_id: &str) -> Result<bool, AppError> {
    Ok(
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
            .bind(colony_id)
            .fetch_one(pool)
            .await?,
    )
}

async fn box_at(
    pool: &SqlitePool,
    colony_id: &str,
    fed_at: &str,
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
    .bind(fed_at)
    .bind(fed_at)
    .fetch_optional(pool)
    .await?)
}

async fn get(pool: &SqlitePool, id: &str) -> Result<Feeding, AppError> {
    Ok(sqlx::query_as::<_, Feeding>(
        "SELECT f.id, f.colony_id, c.code AS colony_code, f.box_id, b.code AS box_code,
                f.fed_at, f.food_type, f.quantity, f.unit, f.response_notes, f.notes,
                f.next_feeding_at, f.created_at
         FROM feedings f
         JOIN colonies c ON c.id = f.colony_id
         LEFT JOIN boxes b ON b.id = f.box_id
         WHERE f.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn create(pool: &SqlitePool, input: CreateFeeding) -> Result<Feeding, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let food_type = required(&input.food_type, "Tipo de alimentação")?;
    let unit = validate_quantity(input.quantity, &input.unit)?;

    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    let fed_at = match optional(&input.fed_at) {
        Some(value) => value,
        None => {
            sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
                .fetch_one(pool)
                .await?
        }
    };

    let box_id = box_at(pool, &colony_id, &fed_at).await?;
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO feedings (
            id, colony_id, box_id, fed_at, food_type, quantity, unit,
            response_notes, notes, next_feeding_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&colony_id)
    .bind(box_id)
    .bind(fed_at)
    .bind(food_type)
    .bind(input.quantity)
    .bind(unit)
    .bind(optional(&input.response_notes))
    .bind(optional(&input.notes))
    .bind(optional(&input.next_feeding_at))
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn list_by_colony(pool: &SqlitePool, colony_id: &str) -> Result<Vec<Feeding>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;

    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, Feeding>(
        "SELECT f.id, f.colony_id, c.code AS colony_code, f.box_id, b.code AS box_code,
                f.fed_at, f.food_type, f.quantity, f.unit, f.response_notes, f.notes,
                f.next_feeding_at, f.created_at
         FROM feedings f
         JOIN colonies c ON c.id = f.colony_id
         LEFT JOIN boxes b ON b.id = f.box_id
         WHERE f.colony_id = ?
         ORDER BY f.fed_at DESC, f.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM feedings")
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
                material: None,
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
                box_id: box_one.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: None,
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
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        (colony.id, box_one.id, box_two.id)
    }

    #[tokio::test]
    async fn retrospective_feeding_keeps_box_from_that_date() {
        let pool = test_pool().await;
        let (colony_id, box_one_id, _) = seed(&pool).await;

        let feeding = create(
            &pool,
            CreateFeeding {
                colony_id,
                fed_at: Some("2026-01-15 12:00:00".into()),
                food_type: "Xarope".into(),
                quantity: Some(50.0),
                unit: Some("ml".into()),
                response_notes: None,
                notes: None,
                next_feeding_at: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(feeding.box_id.as_deref(), Some(box_one_id.as_str()));
        assert_eq!(feeding.box_code.as_deref(), Some("CX-001"));
    }

    #[tokio::test]
    async fn quantity_requires_unit() {
        let pool = test_pool().await;
        let (colony_id, _, _) = seed(&pool).await;

        let result = create(
            &pool,
            CreateFeeding {
                colony_id,
                fed_at: None,
                food_type: "Xarope".into(),
                quantity: Some(50.0),
                unit: None,
                response_notes: None,
                notes: None,
                next_feeding_at: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn feeding_appears_in_colony_timeline() {
        let pool = test_pool().await;
        let (colony_id, _, _) = seed(&pool).await;

        create(
            &pool,
            CreateFeeding {
                colony_id: colony_id.clone(),
                fed_at: Some("2026-02-10 12:00:00".into()),
                food_type: "Xarope 1:1".into(),
                quantity: Some(40.0),
                unit: Some("ml".into()),
                response_notes: Some("Boa aceitação".into()),
                notes: None,
                next_feeding_at: Some("2026-02-17 12:00:00".into()),
            },
        )
        .await
        .unwrap();

        let timeline = history::timeline_by_colony(&pool, &colony_id)
            .await
            .unwrap();
        assert!(timeline.iter().any(|entry| entry.source_type == "feeding"));
    }
}
