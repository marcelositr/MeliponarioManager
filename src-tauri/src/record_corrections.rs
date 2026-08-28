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

pub async fn correct_inspection(
    pool: &SqlitePool,
    input: CorrectInspection,
) -> Result<(), AppError> {
    let id = required(&input.id, "Inspeção")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let strength = required(&input.strength, "Força")?;
    if !STRENGTHS.contains(&strength.as_str()) {
        return Err(AppError::Validation(
            "Força da colônia inválida.".to_owned(),
        ));
    }
    let inspected_at = time::normalize(&input.inspected_at, false)?;
    let next = time::normalize_optional(&input.next_inspection_at, false)?;
    time::ensure_not_before(
        &next,
        &inspected_at,
        "A próxima inspeção não pode ser anterior à inspeção registrada.",
    )?;

    let colony_id: Option<String> =
        sqlx::query_scalar("SELECT colony_id FROM inspections WHERE id = ? AND voided_at IS NULL")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let colony_id = colony_id
        .ok_or_else(|| AppError::Validation("Inspeção não encontrada ou já anulada.".to_owned()))?;
    operational::ensure_colony_available_at(pool, &colony_id, &inspected_at).await?;
    let box_id = historical_box_for_colony(pool, &colony_id, &inspected_at).await?;

    let mut tx = pool.begin().await?;
    let before = snapshot_tx(&mut tx,
        "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'inspected_at',inspected_at,'strength',strength,'queen_present',queen_present,'laying_status',laying_status,'food_reserves',food_reserves,'brood_status',brood_status,'pests_notes',pests_notes,'observations',observations,'actions_taken',actions_taken,'next_inspection_at',next_inspection_at,'voided_at',voided_at) FROM inspections WHERE id = ?",
        &id, "Inspeção não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query(
        "UPDATE inspections SET box_id=?, inspected_at=?, strength=?, queen_present=?,
            laying_status=?, food_reserves=?, brood_status=?, pests_notes=?, observations=?,
            actions_taken=?, next_inspection_at=?, corrected_at=? WHERE id=? AND voided_at IS NULL",
    )
    .bind(box_id)
    .bind(&inspected_at)
    .bind(strength)
    .bind(input.queen_present)
    .bind(optional(&input.laying_status))
    .bind(optional(&input.food_reserves))
    .bind(optional(&input.brood_status))
    .bind(optional(&input.pests_notes))
    .bind(optional(&input.observations))
    .bind(optional(&input.actions_taken))
    .bind(next)
    .bind(corrected_at)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = snapshot_tx(&mut tx,
        "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'inspected_at',inspected_at,'strength',strength,'queen_present',queen_present,'laying_status',laying_status,'food_reserves',food_reserves,'brood_status',brood_status,'pests_notes',pests_notes,'observations',observations,'actions_taken',actions_taken,'next_inspection_at',next_inspection_at,'voided_at',voided_at) FROM inspections WHERE id = ?",
        &id, "Inspeção não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "inspection",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_inspection(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "inspection", "inspections",
        "SELECT json_object('id',id,'colony_id',colony_id,'inspected_at',inspected_at,'strength',strength,'next_inspection_at',next_inspection_at,'voided_at',voided_at,'void_reason',void_reason) FROM inspections WHERE id = ?")
        .await
}

pub async fn correct_feeding(pool: &SqlitePool, input: CorrectFeeding) -> Result<(), AppError> {
    let id = required(&input.id, "Alimentação")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let food_type = required(&input.food_type, "Tipo de alimento")?;
    let fed_at = time::normalize(&input.fed_at, false)?;
    let next = time::normalize_optional(&input.next_feeding_at, false)?;
    time::ensure_not_before(
        &next,
        &fed_at,
        "A próxima alimentação não pode ser anterior à alimentação registrada.",
    )?;
    let unit = optional(&input.unit);
    match (input.quantity, unit.as_ref()) {
        (None, None) => {}
        (Some(value), Some(_)) if value.is_finite() && value > 0.0 => {}
        (Some(_), None) => {
            return Err(AppError::Validation(
                "Informe a unidade quando houver quantidade.".to_owned(),
            ))
        }
        (None, Some(_)) => {
            return Err(AppError::Validation(
                "Informe a quantidade quando houver unidade.".to_owned(),
            ))
        }
        _ => {
            return Err(AppError::Validation(
                "A quantidade precisa ser maior que zero.".to_owned(),
            ))
        }
    }
    let colony_id: Option<String> =
        sqlx::query_scalar("SELECT colony_id FROM feedings WHERE id=? AND voided_at IS NULL")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let colony_id = colony_id.ok_or_else(|| {
        AppError::Validation("Alimentação não encontrada ou já anulada.".to_owned())
    })?;
    operational::ensure_colony_available_at(pool, &colony_id, &fed_at).await?;
    let box_id = historical_box_for_colony(pool, &colony_id, &fed_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'fed_at',fed_at,'food_type',food_type,'quantity',quantity,'unit',unit,'response_notes',response_notes,'notes',notes,'next_feeding_at',next_feeding_at,'voided_at',voided_at) FROM feedings WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Alimentação não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE feedings SET box_id=?, fed_at=?, food_type=?, quantity=?, unit=?, response_notes=?, notes=?, next_feeding_at=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(box_id).bind(&fed_at).bind(food_type).bind(input.quantity).bind(unit)
        .bind(optional(&input.response_notes)).bind(optional(&input.notes)).bind(next)
        .bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Alimentação não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "feeding",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_feeding(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "feeding", "feedings",
        "SELECT json_object('id',id,'colony_id',colony_id,'fed_at',fed_at,'food_type',food_type,'next_feeding_at',next_feeding_at,'voided_at',voided_at,'void_reason',void_reason) FROM feedings WHERE id=?")
        .await
}

pub async fn correct_production(
    pool: &SqlitePool,
    input: CorrectProduction,
) -> Result<(), AppError> {
    let id = required(&input.id, "Produção")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let product_type = required(&input.product_type, "Produto")?;
    if !PRODUCT_TYPES.contains(&product_type.as_str()) {
        return Err(AppError::Validation("Tipo de produto inválido.".to_owned()));
    }
    if !input.quantity.is_finite() || input.quantity <= 0.0 {
        return Err(AppError::Validation(
            "A quantidade precisa ser maior que zero.".to_owned(),
        ));
    }
    let unit = required(&input.unit, "Unidade")?;
    let harvested_at = time::normalize(&input.harvested_at, false)?;
    let colony_id: Option<String> = sqlx::query_scalar(
        "SELECT colony_id FROM production_records WHERE id=? AND voided_at IS NULL",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let colony_id = colony_id
        .ok_or_else(|| AppError::Validation("Produção não encontrada ou já anulada.".to_owned()))?;
    operational::ensure_colony_available_at(pool, &colony_id, &harvested_at).await?;
    let box_id = historical_box_for_colony(pool, &colony_id, &harvested_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'harvested_at',harvested_at,'product_type',product_type,'quantity',quantity,'unit',unit,'purpose',purpose,'notes',notes,'voided_at',voided_at) FROM production_records WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Produção não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE production_records SET box_id=?, harvested_at=?, product_type=?, quantity=?, unit=?, purpose=?, notes=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(box_id).bind(&harvested_at).bind(product_type).bind(input.quantity).bind(unit)
        .bind(optional(&input.purpose)).bind(optional(&input.notes)).bind(corrected_at).bind(&id)
        .execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Produção não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "production",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_production(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "production", "production_records",
        "SELECT json_object('id',id,'colony_id',colony_id,'harvested_at',harvested_at,'product_type',product_type,'quantity',quantity,'unit',unit,'voided_at',voided_at,'void_reason',void_reason) FROM production_records WHERE id=?")
        .await
}

pub async fn correct_maintenance(
    pool: &SqlitePool,
    input: CorrectMaintenance,
) -> Result<(), AppError> {
    let id = required(&input.id, "Manutenção")?;
    let box_id = required(&input.box_id, "Caixa")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let maintenance_type = required(&input.maintenance_type, "Tipo de manutenção")?;
    if !MAINTENANCE_TYPES.contains(&maintenance_type.as_str()) {
        return Err(AppError::Validation(
            "Tipo de manutenção inválido.".to_owned(),
        ));
    }
    if input
        .cost
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(AppError::Validation(
            "O custo precisa ser válido e não negativo.".to_owned(),
        ));
    }
    let maintained_at = time::normalize(&input.maintained_at, false)?;
    let next = time::normalize_optional(&input.next_maintenance_at, false)?;
    time::ensure_not_before(
        &next,
        &maintained_at,
        "A próxima manutenção não pode ser anterior à manutenção registrada.",
    )?;
    let box_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM boxes WHERE id=?)")
        .bind(&box_id)
        .fetch_one(pool)
        .await?;
    if !box_exists {
        return Err(AppError::NotFound("Caixa não encontrada.".to_owned()));
    }
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM box_maintenance_records WHERE id=? AND voided_at IS NULL)",
    )
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Err(AppError::Validation(
            "Manutenção não encontrada ou já anulada.".to_owned(),
        ));
    }
    let colony_id = colony_in_box_at(pool, &box_id, &maintained_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'box_id',box_id,'colony_id',colony_id,'maintained_at',maintained_at,'maintenance_type',maintenance_type,'description',description,'performed_by',performed_by,'cost',cost,'next_maintenance_at',next_maintenance_at,'voided_at',voided_at) FROM box_maintenance_records WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Manutenção não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE box_maintenance_records SET box_id=?, colony_id=?, maintained_at=?, maintenance_type=?, description=?, performed_by=?, cost=?, next_maintenance_at=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(&box_id).bind(colony_id).bind(&maintained_at).bind(maintenance_type)
        .bind(optional(&input.description)).bind(optional(&input.performed_by)).bind(input.cost)
        .bind(next).bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Manutenção não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "box_maintenance",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_maintenance(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "box_maintenance", "box_maintenance_records",
        "SELECT json_object('id',id,'box_id',box_id,'colony_id',colony_id,'maintained_at',maintained_at,'maintenance_type',maintenance_type,'next_maintenance_at',next_maintenance_at,'voided_at',voided_at,'void_reason',void_reason) FROM box_maintenance_records WHERE id=?")
        .await
}

pub async fn correct_event(pool: &SqlitePool, input: CorrectEvent) -> Result<(), AppError> {
    let id = required(&input.id, "Evento")?;
    let event_type = required(&input.event_type, "Tipo do evento")?;
    let severity = required(&input.severity, "Importância")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    if !EVENT_TYPES.contains(&event_type.as_str()) {
        return Err(AppError::Validation("Tipo de evento inválido.".to_owned()));
    }
    if !SEVERITIES.contains(&severity.as_str()) {
        return Err(AppError::Validation("Nível do evento inválido.".to_owned()));
    }
    let occurred_at = time::normalize(&input.occurred_at, false)?;
    let now = time::local_now(pool).await?;
    if occurred_at > now {
        return Err(AppError::Validation("Eventos manuais representam fatos ocorridos e não podem ser registrados no futuro. Use a Agenda em etapa própria.".to_owned()));
    }
    let colony_id: Option<String> =
        sqlx::query_scalar("SELECT colony_id FROM colony_events WHERE id=? AND voided_at IS NULL")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let colony_id = colony_id
        .ok_or_else(|| AppError::Validation("Evento não encontrado ou já anulado.".to_owned()))?;
    let box_id = historical_box_for_colony(pool, &colony_id, &occurred_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'event_type',event_type,'occurred_at',occurred_at,'title',title,'details',details,'severity',severity,'voided_at',voided_at) FROM colony_events WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Evento não encontrado.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_events SET box_id=?, event_type=?, occurred_at=?, title=?, details=?, severity=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(box_id).bind(event_type).bind(&occurred_at).bind(optional(&input.title))
        .bind(optional(&input.details)).bind(severity).bind(corrected_at).bind(&id)
        .execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Evento não encontrado.").await?;
    audit::record_tx(
        &mut tx,
        "colony_event",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_event(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "colony_event", "colony_events",
        "SELECT json_object('id',id,'colony_id',colony_id,'event_type',event_type,'occurred_at',occurred_at,'title',title,'severity',severity,'voided_at',voided_at,'void_reason',void_reason) FROM colony_events WHERE id=?")
        .await
}

pub async fn correct_movement_details(
    pool: &SqlitePool,
    input: CorrectMovementDetails,
) -> Result<(), AppError> {
    let id = required(&input.id, "Movimentação")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let movement: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT movement_type, voided_at, reversed_at FROM colony_movements WHERE id=?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let (movement_type, voided_at, reversed_at) =
        movement.ok_or_else(|| AppError::NotFound("Movimentação não encontrada.".to_owned()))?;
    if voided_at.is_some() || reversed_at.is_some() {
        return Err(AppError::Validation(
            "Movimentação anulada ou revertida não pode ser editada.".to_owned(),
        ));
    }
    let destination = optional(&input.destination);
    if movement_type != "internal_transfer" && destination.is_none() {
        return Err(AppError::Validation(
            "Informe o destino da movimentação.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'movement_type',movement_type,'moved_at',moved_at,'from_meliponary_id',from_meliponary_id,'to_meliponary_id',to_meliponary_id,'from_box_id',from_box_id,'to_box_id',to_box_id,'destination',destination,'notes',notes,'voided_at',voided_at,'reversed_at',reversed_at) FROM colony_movements WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Movimentação não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_movements SET destination=?, notes=?, corrected_at=? WHERE id=?")
        .bind(if movement_type == "internal_transfer" {
            None::<String>
        } else {
            destination
        })
        .bind(optional(&input.notes))
        .bind(corrected_at)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Movimentação não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "movement",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_transport(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    let id = required(&input.id, "Movimentação")?;
    let kind: Option<String> =
        sqlx::query_scalar("SELECT movement_type FROM colony_movements WHERE id=?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    match kind.as_deref() {
        Some("transport") => void_fact(pool, VoidRecord { id, reason: input.reason }, "movement", "colony_movements",
            "SELECT json_object('id',id,'movement_type',movement_type,'moved_at',moved_at,'destination',destination,'notes',notes,'voided_at',voided_at,'void_reason',void_reason,'reversed_at',reversed_at) FROM colony_movements WHERE id=?").await,
        Some(_) => Err(AppError::Validation("Transferências têm consequências e precisam ser revertidas pelo fluxo específico.".to_owned())),
        None => Err(AppError::NotFound("Movimentação não encontrada.".to_owned())),
    }
}

pub async fn update_movement_document(
    pool: &SqlitePool,
    input: UpdateMovementDocument,
) -> Result<(), AppError> {
    let id = required(&input.id, "Documento")?;
    let document_type = required(&input.document_type, "Tipo")?;
    let reference = required(&input.reference_number, "Referência")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    if !DOCUMENT_TYPES.contains(&document_type.as_str()) {
        return Err(AppError::Validation(
            "Tipo de documento inválido.".to_owned(),
        ));
    }
    let issued_at = time::normalize_optional(&input.issued_at, false)?;
    let valid_until = time::normalize_optional(&input.valid_until, false)?;
    if let (Some(issued), Some(valid)) = (&issued_at, &valid_until) {
        if valid < issued {
            return Err(AppError::Validation(
                "A validade não pode ser anterior à emissão.".to_owned(),
            ));
        }
    }
    let movement_id: Option<String> = sqlx::query_scalar(
        "SELECT movement_id FROM movement_documents WHERE id=? AND voided_at IS NULL",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let movement_id = movement_id.ok_or_else(|| {
        AppError::Validation("Documento não encontrado ou invalidado.".to_owned())
    })?;
    let duplicate: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM movement_documents WHERE id<>? AND movement_id=? AND document_type=? AND reference_number=? AND voided_at IS NULL)")
        .bind(&id).bind(&movement_id).bind(&document_type).bind(&reference).fetch_one(pool).await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe um documento válido com esta referência na movimentação.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'movement_id',movement_id,'document_type',document_type,'reference_number',reference_number,'source_system',source_system,'issuer',issuer,'issued_at',issued_at,'valid_until',valid_until,'file_path',file_path,'notes',notes,'voided_at',voided_at) FROM movement_documents WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Documento não encontrado.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE movement_documents SET document_type=?, reference_number=?, source_system=?, issuer=?, issued_at=?, valid_until=?, file_path=?, notes=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(document_type).bind(reference).bind(optional(&input.source_system)).bind(optional(&input.issuer))
        .bind(issued_at).bind(valid_until).bind(optional(&input.file_path)).bind(optional(&input.notes))
        .bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Documento não encontrado.").await?;
    audit::record_tx(
        &mut tx,
        "movement_document",
        &id,
        "edit",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_movement_document(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "movement_document", "movement_documents",
        "SELECT json_object('id',id,'movement_id',movement_id,'document_type',document_type,'reference_number',reference_number,'voided_at',voided_at,'void_reason',void_reason) FROM movement_documents WHERE id=?")
        .await
}

pub async fn correct_division(pool: &SqlitePool, input: CorrectDivision) -> Result<(), AppError> {
    let id = required(&input.id, "Divisão")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'parent_colony_id',parent_colony_id,'daughter_colony_id',daughter_colony_id,'source_box_id',source_box_id,'performed_at',performed_at,'result',result,'notes',notes,'voided_at',voided_at) FROM colony_divisions WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    let already_void: Option<String> =
        sqlx::query_scalar("SELECT voided_at FROM colony_divisions WHERE id=?")
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;
    if already_void.is_some() {
        return Err(AppError::Validation(
            "Divisão anulada não pode ser corrigida.".to_owned(),
        ));
    }
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_divisions SET notes=?, corrected_at=? WHERE id=?")
        .bind(optional(&input.notes))
        .bind(corrected_at)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "division",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_division(pool: &SqlitePool, input: VoidDivision) -> Result<(), AppError> {
    let id = required(&input.id, "Divisão")?;
    let reason = required(&input.reason, "Motivo da anulação")?;
    let row: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("SELECT daughter_colony_id, voided_at FROM colony_divisions WHERE id=?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let (daughter_id, voided_at) =
        row.ok_or_else(|| AppError::NotFound("Divisão não encontrada.".to_owned()))?;
    if voided_at.is_some() {
        return Err(AppError::Validation(
            "A divisão já está anulada.".to_owned(),
        ));
    }

    let disposition = optional(&input.daughter_disposition);
    if let Some(daughter_id) = &daughter_id {
        let consequences: bool = sqlx::query_scalar(
            "SELECT
                EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM inspections WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM feedings WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM production_records WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_events WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_movements WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM box_maintenance_records WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_divisions WHERE id<>? AND (parent_colony_id=? OR daughter_colony_id=?))
                OR EXISTS(SELECT 1 FROM colonies WHERE mother_colony_id=?)",
        )
        .bind(daughter_id).bind(daughter_id).bind(daughter_id).bind(daughter_id)
        .bind(daughter_id).bind(daughter_id).bind(daughter_id).bind(daughter_id)
        .bind(&id).bind(daughter_id).bind(daughter_id).bind(daughter_id)
        .fetch_one(pool).await?;
        if consequences {
            return Err(AppError::Validation(
                "A divisão criou uma filha que já possui consequências históricas. A anulação automática foi bloqueada para não reescrever o passado."
                    .to_owned(),
            ));
        }
        if !matches!(disposition.as_deref(), Some("keep") | Some("deactivate")) {
            return Err(AppError::Validation(
                "Informe explicitamente o destino da filha: keep para preservá-la ativa ou deactivate para inativá-la."
                    .to_owned(),
            ));
        }
    }

    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'parent_colony_id',parent_colony_id,'daughter_colony_id',daughter_colony_id,'performed_at',performed_at,'result',result,'notes',notes,'voided_at',voided_at,'void_reason',void_reason) FROM colony_divisions WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    let voided_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_divisions SET voided_at=?, void_reason=? WHERE id=?")
        .bind(&voided_at)
        .bind(&reason)
        .bind(&id)
        .execute(&mut *tx)
        .await?;

    if let (Some(daughter_id), Some("deactivate")) = (&daughter_id, disposition.as_deref()) {
        let previous: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
            .bind(daughter_id)
            .fetch_one(&mut *tx)
            .await?;
        if matches!(previous.as_str(), "active" | "weak" | "recovering") {
            sqlx::query(
                "UPDATE colonies SET status='inactive', updated_at=CURRENT_TIMESTAMP WHERE id=?",
            )
            .bind(daughter_id)
            .execute(&mut *tx)
            .await?;
            let lifecycle_id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO colony_lifecycle_records (id,colony_id,action,occurred_at,previous_status,new_status,reason,notes) VALUES (?,?,'deactivate',?,?,'inactive',?,'Gerado explicitamente durante a anulação de uma divisão sem consequências posteriores.')")
                .bind(lifecycle_id).bind(daughter_id).bind(&voided_at).bind(previous)
                .bind(format!("Anulação da divisão {id}: {reason}"))
                .execute(&mut *tx).await?;
        }
    }

    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "division",
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

pub async fn correct_occupancy(pool: &SqlitePool, input: CorrectOccupancy) -> Result<(), AppError> {
    let id = required(&input.id, "Ocupação")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let started_at = time::normalize(&input.started_at, false)?;
    let ended_at = time::normalize_optional(&input.ended_at, false)?;
    if ended_at
        .as_deref()
        .is_some_and(|end| end < started_at.as_str())
    {
        return Err(AppError::Validation(
            "O fim da ocupação não pode ser anterior ao início.".to_owned(),
        ));
    }
    let row: Option<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT colony_id, box_id, started_at, ended_at FROM colony_box_occupancies WHERE id=?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let (colony_id, box_id, old_start, old_end) =
        row.ok_or_else(|| AppError::NotFound("Ocupação não encontrada.".to_owned()))?;
    if old_end.is_none() != ended_at.is_none() {
        return Err(AppError::Validation(
            "A correção histórica não pode abrir ou encerrar uma ocupação. Use o fluxo operacional correspondente."
                .to_owned(),
        ));
    }

    let state_at: Option<String> = sqlx::query_scalar(
        "SELECT new_status FROM box_state_records WHERE box_id=? AND occurred_at<=?
         ORDER BY occurred_at DESC, created_at DESC, id DESC LIMIT 1",
    )
    .bind(&box_id)
    .bind(&started_at)
    .fetch_optional(pool)
    .await?;
    if state_at.as_deref().unwrap_or("active") != "active" {
        return Err(AppError::Validation(
            "A caixa não estava ativa no novo início da ocupação.".to_owned(),
        ));
    }

    let new_end_for_overlap = ended_at.as_deref().unwrap_or("9999-12-31 23:59:59");
    let colony_overlap: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM colony_box_occupancies
            WHERE id<>? AND colony_id=? AND started_at<=?
              AND COALESCE(ended_at,'9999-12-31 23:59:59')>=?)",
    )
    .bind(&id)
    .bind(&colony_id)
    .bind(new_end_for_overlap)
    .bind(&started_at)
    .fetch_one(pool)
    .await?;
    if colony_overlap {
        return Err(AppError::Validation(
            "A correção faria a colônia ocupar duas caixas no mesmo intervalo.".to_owned(),
        ));
    }
    let box_overlap: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM colony_box_occupancies
            WHERE id<>? AND box_id=? AND started_at<=?
              AND COALESCE(ended_at,'9999-12-31 23:59:59')>=?)",
    )
    .bind(&id)
    .bind(&box_id)
    .bind(new_end_for_overlap)
    .bind(&started_at)
    .fetch_one(pool)
    .await?;
    if box_overlap {
        return Err(AppError::Validation(
            "A correção faria a caixa abrigar duas colônias no mesmo intervalo.".to_owned(),
        ));
    }

    let fact_outside: bool = sqlx::query_scalar(
        "WITH facts(ts) AS (
            SELECT inspected_at FROM inspections WHERE colony_id=? AND box_id=?
            UNION ALL SELECT fed_at FROM feedings WHERE colony_id=? AND box_id=?
            UNION ALL SELECT harvested_at FROM production_records WHERE colony_id=? AND box_id=?
            UNION ALL SELECT occurred_at FROM colony_events WHERE colony_id=? AND box_id=?
            UNION ALL SELECT occurred_at FROM colony_lifecycle_records WHERE colony_id=? AND box_id=?
            UNION ALL SELECT maintained_at FROM box_maintenance_records WHERE colony_id=? AND box_id=?
            UNION ALL SELECT moved_at FROM colony_movements WHERE colony_id=? AND (from_box_id=? OR to_box_id=?)
         )
         SELECT EXISTS(
            SELECT 1 FROM facts
            WHERE ts>=? AND (? IS NULL OR ts<=?)
              AND (ts<? OR (? IS NOT NULL AND ts>?))
         )",
    )
    .bind(&colony_id).bind(&box_id).bind(&colony_id).bind(&box_id)
    .bind(&colony_id).bind(&box_id).bind(&colony_id).bind(&box_id)
    .bind(&colony_id).bind(&box_id).bind(&colony_id).bind(&box_id)
    .bind(&colony_id).bind(&box_id).bind(&box_id)
    .bind(&old_start).bind(&old_end).bind(&old_end)
    .bind(&started_at).bind(&ended_at).bind(&ended_at)
    .fetch_one(pool).await?;
    if fact_outside {
        return Err(AppError::Validation(
            "A correção removeria do intervalo fatos que já usam esta ocupação como contexto histórico. Corrija primeiro os fatos dependentes."
                .to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'started_at',started_at,'ended_at',ended_at,'reason',reason,'notes',notes,'corrected_at',corrected_at) FROM colony_box_occupancies WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Ocupação não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_box_occupancies SET started_at=?, ended_at=?, reason=?, notes=?, corrected_at=? WHERE id=?")
        .bind(&started_at).bind(&ended_at).bind(optional(&input.occupancy_reason)).bind(optional(&input.notes))
        .bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Ocupação não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "box_occupancy",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn void_fact(
    pool: &SqlitePool,
    input: VoidRecord,
    entity_type: &str,
    table: &str,
    snapshot_sql: &str,
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
mod tests {
    use super::*;
    use crate::{
        alerts,
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        feeding::{self, CreateFeeding},
        inspections::{self, CreateInspection},
        maintenance::{self, CreateBoxMaintenance},
        production::{self, CreateProductionRecord},
        repository,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    async fn seeded() -> (SqlitePool, String, String) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        let mel = repository::create_meliponary(
            &pool,
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
            &pool,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let box1 = repository::create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: mel.id.clone(),
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
            &pool,
            CreateColony {
                meliponary_id: mel.id,
                species_id: species.id,
                code: "JAT-001".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: Some("2026-01-01 09:00:00".into()),
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::place_colony(
            &pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: box1.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        (pool, colony.id, box1.id)
    }

    #[tokio::test]
    async fn inspection_correction_and_void_preserve_latest_valid_semantics() {
        let (pool, colony_id, _) = seeded().await;
        let old = inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-01-10 10:00:00".into()),
                strength: Some("weak".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2026-01-20 10:00:00".into()),
            },
        )
        .await
        .unwrap();
        let latest = inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-02-10 10:00:00".into()),
                strength: Some("strong".into()),
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
        .await
        .unwrap();
        correct_inspection(
            &pool,
            CorrectInspection {
                id: old.id.clone(),
                inspected_at: "2026-01-11 10:00:00".into(),
                strength: "medium".into(),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2026-01-21 10:00:00".into()),
                reason: "Data corrigida".into(),
            },
        )
        .await
        .unwrap();
        void_inspection(
            &pool,
            VoidRecord {
                id: latest.id,
                reason: "Lançamento duplicado".into(),
            },
        )
        .await
        .unwrap();
        assert!(!alerts::list(&pool)
            .await
            .unwrap()
            .iter()
            .any(|a| a.alert_type == "weak_colony"));
        let audit_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_records WHERE entity_type='inspection'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(audit_count, 2);
    }

    #[tokio::test]
    async fn invalid_inspection_correction_date_is_rejected() {
        let (pool, colony_id, _) = seeded().await;
        let item = inspections::create(
            &pool,
            CreateInspection {
                colony_id,
                inspected_at: Some("2026-02-10 10:00:00".into()),
                strength: Some("strong".into()),
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
        .await
        .unwrap();
        assert!(correct_inspection(
            &pool,
            CorrectInspection {
                id: item.id,
                inspected_at: "2025-01-01 10:00:00".into(),
                strength: "strong".into(),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: None,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn voided_feeding_does_not_generate_due_alert() {
        let (pool, colony_id, _) = seeded().await;
        let item = feeding::create(
            &pool,
            CreateFeeding {
                colony_id,
                fed_at: Some("2026-01-10 10:00:00".into()),
                food_type: "Xarope".into(),
                quantity: Some(10.0),
                unit: Some("ml".into()),
                response_notes: None,
                notes: None,
                next_feeding_at: Some("2026-01-11 10:00:00".into()),
            },
        )
        .await
        .unwrap();
        void_feeding(
            &pool,
            VoidRecord {
                id: item.id,
                reason: "Erro de lançamento".into(),
            },
        )
        .await
        .unwrap();
        assert!(!alerts::list(&pool)
            .await
            .unwrap()
            .iter()
            .any(|a| a.alert_type == "feeding_due"));
    }

    #[tokio::test]
    async fn voided_production_is_excluded_from_valid_count() {
        let (pool, colony_id, _) = seeded().await;
        let item = production::create(
            &pool,
            CreateProductionRecord {
                colony_id,
                harvested_at: Some("2026-02-01 10:00:00".into()),
                product_type: "honey".into(),
                quantity: 20.0,
                unit: "ml".into(),
                purpose: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        void_production(
            &pool,
            VoidRecord {
                id: item.id,
                reason: "Pesagem incorreta".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(production::count(&pool).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn maintenance_correction_revalidates_next_date() {
        let (pool, _, box_id) = seeded().await;
        let item = maintenance::create(
            &pool,
            CreateBoxMaintenance {
                box_id: box_id.clone(),
                maintained_at: Some("2026-02-01 10:00:00".into()),
                maintenance_type: "repair".into(),
                description: None,
                performed_by: None,
                cost: None,
                next_maintenance_at: None,
            },
        )
        .await
        .unwrap();
        assert!(correct_maintenance(
            &pool,
            CorrectMaintenance {
                id: item.id,
                box_id,
                maintained_at: "2026-02-01 10:00:00".into(),
                maintenance_type: "repair".into(),
                description: None,
                performed_by: None,
                cost: None,
                next_maintenance_at: Some("2026-01-01 10:00:00".into()),
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn occupancy_overlap_is_rejected() {
        let (pool, colony_id, first_box) = seeded().await;
        let mel_id: String = sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id=?")
            .bind(&colony_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let second = repository::create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: mel_id,
                code: "CX-002".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::place_colony(
            &pool,
            PlaceColony {
                colony_id: colony_id.clone(),
                box_id: second.id,
                started_at: Some("2026-03-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let first_id: String = sqlx::query_scalar(
            "SELECT id FROM colony_box_occupancies WHERE colony_id=? AND box_id=?",
        )
        .bind(&colony_id)
        .bind(first_box)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(correct_occupancy(
            &pool,
            CorrectOccupancy {
                id: first_id,
                started_at: "2026-01-01 09:00:00".into(),
                ended_at: Some("2026-04-01 10:00:00".into()),
                occupancy_reason: None,
                notes: None,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
    }
}
