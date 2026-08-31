use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
const TYPES: &[&str] = &["honey", "pollen", "propolis", "wax", "cerumen", "other"];
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
    pub corrected_at: Option<String>,
    pub voided_at: Option<String>,
    pub void_reason: Option<String>,
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
fn req(v: &str, f: &str) -> Result<String, AppError> {
    let v = v.trim();
    if v.is_empty() {
        Err(AppError::Validation(format!("{f} é obrigatório.")))
    } else {
        Ok(v.to_owned())
    }
}
fn opt(v: &Option<String>) -> Option<String> {
    v.as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}
const SELECT_BY_ID: &str = "SELECT p.id,p.colony_id,c.code AS colony_code,p.box_id,b.code AS box_code,p.harvested_at,p.product_type,p.quantity,p.unit,p.purpose,p.notes,p.corrected_at,p.voided_at,p.void_reason,p.created_at FROM production_records p JOIN colonies c ON c.id=p.colony_id LEFT JOIN boxes b ON b.id=p.box_id WHERE p.id=?";
const SELECT_BY_COLONY: &str = "SELECT p.id,p.colony_id,c.code AS colony_code,p.box_id,b.code AS box_code,p.harvested_at,p.product_type,p.quantity,p.unit,p.purpose,p.notes,p.corrected_at,p.voided_at,p.void_reason,p.created_at FROM production_records p JOIN colonies c ON c.id=p.colony_id LEFT JOIN boxes b ON b.id=p.box_id WHERE p.colony_id=? ORDER BY p.harvested_at DESC,p.created_at DESC";
async fn get(p: &SqlitePool, id: &str) -> Result<ProductionRecord, AppError> {
    Ok(sqlx::query_as::<_, ProductionRecord>(SELECT_BY_ID)
        .bind(id)
        .fetch_one(p)
        .await?)
}
pub async fn create(
    p: &SqlitePool,
    i: CreateProductionRecord,
) -> Result<ProductionRecord, AppError> {
    let c = req(&i.colony_id, "Colônia")?;
    let t = req(&i.product_type, "Tipo de produto")?;
    if !TYPES.contains(&t.as_str()) {
        return Err(AppError::Validation("Tipo de produto inválido.".to_owned()));
    }
    if !i.quantity.is_finite() || i.quantity <= 0.0 {
        return Err(AppError::Validation(
            "A quantidade produzida precisa ser maior que zero.".to_owned(),
        ));
    }
    let u = req(&i.unit, "Unidade")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id=?)")
        .bind(&c)
        .fetch_one(p)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }
    let at = match opt(&i.harvested_at) {
        Some(v) => v,
        None => {
            sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
                .fetch_one(p)
                .await?
        }
    };
    let b:Option<String>=sqlx::query_scalar("SELECT box_id FROM colony_box_occupancies WHERE colony_id=? AND started_at<=? AND (ended_at IS NULL OR ended_at>=?) ORDER BY started_at DESC LIMIT 1").bind(&c).bind(&at).bind(&at).fetch_optional(p).await?;
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO production_records(id,colony_id,box_id,harvested_at,product_type,quantity,unit,purpose,notes) VALUES(?,?,?,?,?,?,?,?,?)").bind(&id).bind(&c).bind(b).bind(at).bind(t).bind(i.quantity).bind(u).bind(opt(&i.purpose)).bind(opt(&i.notes)).execute(p).await?;
    get(p, &id).await
}
pub async fn list_by_colony(p: &SqlitePool, c: &str) -> Result<Vec<ProductionRecord>, AppError> {
    let c = req(c, "Colônia")?;
    Ok(sqlx::query_as::<_, ProductionRecord>(SELECT_BY_COLONY)
        .bind(c)
        .fetch_all(p)
        .await?)
}
pub async fn count(p: &SqlitePool) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM production_records WHERE voided_at IS NULL")
            .fetch_one(p)
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

    async fn seed_with_box_history(pool: &SqlitePool) -> (String, String) {
        let meliponary = repository::create_meliponary(
            pool,
            CreateMeliponary {
                name: "Principal".into(),
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
        let first_box = repository::create_box(
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
        let second_box = repository::create_box(
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
                box_id: first_box.id.clone(),
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
                box_id: second_box.id,
                started_at: Some("2026-02-01 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        (colony.id, first_box.id)
    }

    fn production_input(
        colony_id: String,
        harvested_at: Option<&str>,
        product_type: &str,
        quantity: f64,
    ) -> CreateProductionRecord {
        CreateProductionRecord {
            colony_id,
            harvested_at: harvested_at.map(str::to_owned),
            product_type: product_type.to_owned(),
            quantity,
            unit: "ml".into(),
            purpose: None,
            notes: None,
        }
    }

    #[tokio::test]
    async fn retrospective_harvest_keeps_box_from_that_date() {
        let pool = test_pool().await;
        let (colony_id, first_box_id) = seed_with_box_history(&pool).await;

        let record = create(
            &pool,
            production_input(colony_id, Some("2026-01-15 12:00:00"), "honey", 120.0),
        )
        .await
        .unwrap();

        assert_eq!(record.box_id.as_deref(), Some(first_box_id.as_str()));
        assert_eq!(record.box_code.as_deref(), Some("CX-001"));
    }

    #[tokio::test]
    async fn rejects_invalid_product_type() {
        let pool = test_pool().await;
        let (colony_id, _) = seed_with_box_history(&pool).await;

        let result = create(
            &pool,
            production_input(colony_id, None, "royal_jelly", 10.0),
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_non_positive_quantity() {
        for quantity in [0.0, -1.0] {
            let pool = test_pool().await;
            let (colony_id, _) = seed_with_box_history(&pool).await;

            let result = create(&pool, production_input(colony_id, None, "pollen", quantity)).await;

            assert!(matches!(result, Err(AppError::Validation(_))));
        }
    }

    #[tokio::test]
    async fn production_appears_in_colony_timeline() {
        let pool = test_pool().await;
        let (colony_id, _) = seed_with_box_history(&pool).await;

        create(
            &pool,
            production_input(
                colony_id.clone(),
                Some("2026-02-10 12:00:00"),
                "honey",
                80.0,
            ),
        )
        .await
        .unwrap();

        let entries = timeline::by_colony(&pool, &colony_id).await.unwrap();
        assert!(entries
            .iter()
            .any(|entry| entry.source_type == "production"));
    }
}
