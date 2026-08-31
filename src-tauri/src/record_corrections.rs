use crate::{audit, operational, repository::AppError, time};
use serde::Deserialize;
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

const STRENGTHS: &[&str] = &["strong", "medium", "weak", "unknown"];
const PRODUCT_TYPES: &[&str] = &["honey", "pollen", "propolis", "wax", "cerumen", "other"];
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
const EVENT_TYPES: &[&str] = &[
    "swarming",
    "abandonment",
    "queen_loss",
    "attack",
    "pest",
    "recovery",
    "maintenance",
    "observation",
    "other",
];
const SEVERITIES: &[&str] = &["info", "attention", "critical"];
const DOCUMENT_TYPES: &[&str] = &[
    "gta",
    "authorization",
    "invoice",
    "receipt",
    "declaration",
    "protocol",
    "certificate",
    "other",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoidRecord {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectInspection {
    pub id: String,
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
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectFeeding {
    pub id: String,
    pub fed_at: String,
    pub food_type: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub response_notes: Option<String>,
    pub notes: Option<String>,
    pub next_feeding_at: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectProduction {
    pub id: String,
    pub harvested_at: String,
    pub product_type: String,
    pub quantity: f64,
    pub unit: String,
    pub purpose: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectMaintenance {
    pub id: String,
    pub box_id: String,
    pub maintained_at: String,
    pub maintenance_type: String,
    pub description: Option<String>,
    pub performed_by: Option<String>,
    pub cost: Option<f64>,
    pub next_maintenance_at: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectEvent {
    pub id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub title: Option<String>,
    pub details: Option<String>,
    pub severity: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectMovementDetails {
    pub id: String,
    pub destination: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMovementDocument {
    pub id: String,
    pub document_type: String,
    pub reference_number: String,
    pub source_system: Option<String>,
    pub issuer: Option<String>,
    pub issued_at: Option<String>,
    pub valid_until: Option<String>,
    pub file_path: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectDivision {
    pub id: String,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoidDivision {
    pub id: String,
    pub reason: String,
    pub daughter_disposition: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectOccupancy {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub occupancy_reason: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
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

async fn now_tx(tx: &mut Transaction<'_, Sqlite>) -> Result<String, AppError> {
    Ok(
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')")
            .fetch_one(&mut **tx)
            .await?,
    )
}

async fn snapshot_tx(
    tx: &mut Transaction<'_, Sqlite>,
    sql: &'static str,
    id: &str,
    not_found: &str,
) -> Result<Value, AppError> {
    let raw: Option<String> = sqlx::query_scalar(sql)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;
    let raw = raw.ok_or_else(|| AppError::NotFound(not_found.to_owned()))?;
    serde_json::from_str(&raw).map_err(|error| {
        AppError::Validation(format!("Não foi possível preparar a auditoria: {error}"))
    })
}

async fn historical_box_for_colony(
    pool: &SqlitePool,
    colony_id: &str,
    occurred_at: &str,
) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT box_id FROM colony_box_occupancies
         WHERE colony_id = ? AND started_at <= ?
           AND (ended_at IS NULL OR ended_at >= ?)
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(colony_id)
    .bind(occurred_at)
    .bind(occurred_at)
    .fetch_optional(pool)
    .await?)
}

async fn colony_in_box_at(
    pool: &SqlitePool,
    box_id: &str,
    occurred_at: &str,
) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT colony_id FROM colony_box_occupancies
         WHERE box_id = ? AND started_at <= ?
           AND (ended_at IS NULL OR ended_at >= ?)
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(box_id)
    .bind(occurred_at)
    .bind(occurred_at)
    .fetch_optional(pool)
    .await?)
}

mod facts;
mod history;
mod movement_documents;

pub use facts::{
    correct_event, correct_feeding, correct_inspection, correct_maintenance, correct_production,
    void_event, void_feeding, void_inspection, void_maintenance, void_production,
};
pub use history::{correct_division, correct_occupancy, void_division};
pub use movement_documents::{
    correct_movement_details, update_movement_document, void_movement_document, void_transport,
};

async fn void_fact(
    pool: &SqlitePool,
    input: VoidRecord,
    entity_type: &str,
    table: &str,
    snapshot_sql: &'static str,
) -> Result<(), AppError> {
    let id = required(&input.id, "Registro")?;
    let reason = required(&input.reason, "Motivo da anulação")?;
    let mut tx = pool.begin().await?;
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Registro não encontrado.").await?;
    let (current_void_sql, void_sql) = match table {
        "inspections" => (
            "SELECT voided_at FROM inspections WHERE id=?",
            "UPDATE inspections SET voided_at=?, void_reason=? WHERE id=?",
        ),
        "feedings" => (
            "SELECT voided_at FROM feedings WHERE id=?",
            "UPDATE feedings SET voided_at=?, void_reason=? WHERE id=?",
        ),
        "production_records" => (
            "SELECT voided_at FROM production_records WHERE id=?",
            "UPDATE production_records SET voided_at=?, void_reason=? WHERE id=?",
        ),
        "box_maintenance_records" => (
            "SELECT voided_at FROM box_maintenance_records WHERE id=?",
            "UPDATE box_maintenance_records SET voided_at=?, void_reason=? WHERE id=?",
        ),
        "colony_events" => (
            "SELECT voided_at FROM colony_events WHERE id=?",
            "UPDATE colony_events SET voided_at=?, void_reason=? WHERE id=?",
        ),
        "colony_movements" => (
            "SELECT voided_at FROM colony_movements WHERE id=?",
            "UPDATE colony_movements SET voided_at=?, void_reason=? WHERE id=?",
        ),
        "movement_documents" => (
            "SELECT voided_at FROM movement_documents WHERE id=?",
            "UPDATE movement_documents SET voided_at=?, void_reason=? WHERE id=?",
        ),
        _ => {
            return Err(AppError::Validation(
                "Tipo de registro inválido para anulação.".to_owned(),
            ))
        }
    };
    let current_void: Option<String> = sqlx::query_scalar(current_void_sql)
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;
    if current_void.is_some() {
        return Err(AppError::Validation(
            "O registro já está anulado.".to_owned(),
        ));
    }
    let voided_at = now_tx(&mut tx).await?;
    sqlx::query(void_sql)
        .bind(voided_at)
        .bind(&reason)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Registro não encontrado.").await?;
    audit::record_tx(
        &mut tx,
        entity_type,
        &id,
        "void",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests;
