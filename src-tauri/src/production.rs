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
