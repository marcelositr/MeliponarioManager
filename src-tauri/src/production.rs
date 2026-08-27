use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

const PRODUCT_TYPES: &[&str] = &["honey", "pollen", "propolis", "wax", "cerumen", "other"];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductionRecord {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub harvested_at: String,
    pub product_type: String,
    pub quantity: f64,
    pub unit: String,
    pub purpose: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductionRecord {
    pub colony_id: String,
    pub harvested_at: Option<String>,
    pub product_type: String,
    pub quantity: f64,
    pub unit: String,
    pub purpose: Option<String>,
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

fn product_type(value: &str) -> Result<String, AppError> {
    let value = required(value, "Tipo de produto")?;
    if PRODUCT_TYPES.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "Tipo de produto inválido. Use honey, pollen, propolis, wax, cerumen ou other."
                .to_owned(),
        ))
    }
}

fn quantity(value: f64) -> Result<f64, AppError> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "A quantidade produzida precisa ser maior que zero.".to_owned(),
        ))
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
    harvested_at: &str,
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
    .bind(harvested_at)
    .bind(harvested_at)
    .fetch_optional(pool)
    .await?)
}

async fn get(pool: &SqlitePool, id: &str) -> Result<ProductionRecord, AppError> {
    Ok(sqlx::query_as::<_, ProductionRecord>(
        "SELECT p.id, p.colony_id, c.code AS colony_code, p.box_id, b.code AS box_code,
                p.harvested_at, p.product_type, p.quantity, p.unit, p.purpose, p.notes,
                p.created_at
         FROM production_records p
         JOIN colonies c ON c.id = p.colony_id
         LEFT JOIN boxes b ON b.id = p.box_id
         WHERE p.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn create(
    pool: &SqlitePool,
    input: CreateProductionRecord,
) -> Result<ProductionRecord, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let product_type = product_type(&input.product_type)?;
    let quantity = quantity(input.quantity)?;
    let unit = required(&input.unit, "Unidade")?;

    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    let harvested_at = match optional(&input.harvested_at) {
        Some(value) => value,
        None => {
            sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
                .fetch_one(pool)
                .await?
        }
    };

    let box_id = box_at(pool, &colony_id, &harvested_at).await?;
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO production_records (
            id, colony_id, box_id, harvested_at, product_type, quantity, unit, purpose, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&colony_id)
    .bind(box_id)
    .bind(harvested_at)
    .bind(product_type)
    .bind(quantity)
    .bind(unit)
    .bind(optional(&input.purpose))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<ProductionRecord>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;

    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, ProductionRecord>(
        "SELECT p.id, p.colony_id, c.code AS colony_code, p.box_id, b.code AS box_code,
                p.harvested_at, p.product_type, p.quantity, p.unit, p.purpose, p.notes,
                p.created_at
         FROM production_records p
         JOIN colonies c ON c.id = p.colony_id
         LEFT JOIN boxes b ON b.id = p.box_id
         WHERE p.colony_id = ?
         ORDER BY p.harvested_at DESC, p.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM production_records")
            .fetch_one(pool)
            .await?,
    )
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
                box_id: box_two.id,
                started_at: Some("2026-02-01 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        (colony.id, box_one.id)
    }

    #[tokio::test]
    async fn retrospective_harvest_keeps_box_from_that_date() {
        let pool = test_pool().await;
        let (colony_id, box_one_id) = seed(&pool).await;

        let record = create(
            &pool,
            CreateProductionRecord {
                colony_id,
                harvested_at: Some("2026-01-15 12:00:00".into()),
                product_type: "honey".into(),
                quantity: 120.0,
                unit: "ml".into(),
                purpose: Some("Consumo".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(record.box_id.as_deref(), Some(box_one_id.as_str()));
        assert_eq!(record.box_code.as_deref(), Some("CX-001"));
    }

    #[tokio::test]
    async fn rejects_invalid_product_type() {
        let pool = test_pool().await;
        let (colony_id, _) = seed(&pool).await;

        let result = create(
            &pool,
            CreateProductionRecord {
                colony_id,
                harvested_at: None,
                product_type: "royal_jelly".into(),
                quantity: 10.0,
                unit: "g".into(),
                purpose: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_non_positive_quantity() {
        let pool = test_pool().await;
        let (colony_id, _) = seed(&pool).await;

        let result = create(
            &pool,
            CreateProductionRecord {
                colony_id,
                harvested_at: None,
                product_type: "pollen".into(),
                quantity: 0.0,
                unit: "g".into(),
                purpose: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn production_appears_in_colony_timeline() {
        let pool = test_pool().await;
        let (colony_id, _) = seed(&pool).await;

        create(
            &pool,
            CreateProductionRecord {
                colony_id: colony_id.clone(),
                harvested_at: Some("2026-02-10 12:00:00".into()),
                product_type: "honey".into(),
                quantity: 80.0,
                unit: "ml".into(),
                purpose: None,
                notes: Some("Colheita leve".into()),
            },
        )
        .await
        .unwrap();

        let timeline = history::timeline_by_colony(&pool, &colony_id)
            .await
            .unwrap();
        assert!(timeline
            .iter()
            .any(|entry| entry.source_type == "production"));
    }
}
