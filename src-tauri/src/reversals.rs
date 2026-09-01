use crate::{agenda, audit, operational, repository::AppError};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseRecord {
    pub id: String,
    pub reason: String,
}
fn req(v: &str, f: &str) -> Result<String, AppError> {
    let v = v.trim();
    if v.is_empty() {
        Err(AppError::Validation(format!("{f} é obrigatório.")))
    } else {
        Ok(v.to_owned())
    }
}
async fn now(tx: &mut Transaction<'_, Sqlite>) -> Result<String, AppError> {
    Ok(sqlx::query_scalar("SELECT datetime('now','localtime')")
        .fetch_one(&mut **tx)
        .await?)
}
async fn later_facts(
    p: &SqlitePool,
    c: &str,
    after: &str,
    except_lifecycle: Option<&str>,
    except_movement: Option<&str>,
) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM inspections WHERE colony_id=? AND inspected_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM feedings WHERE colony_id=? AND fed_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM production_records WHERE colony_id=? AND harvested_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM colony_events WHERE colony_id=? AND occurred_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM colony_divisions WHERE (parent_colony_id=? OR daughter_colony_id=?) AND performed_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM colony_movements WHERE colony_id=? AND moved_at>? AND voided_at IS NULL AND reversed_at IS NULL AND (? IS NULL OR id<>?)) OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE colony_id=? AND occurred_at>? AND reversed_at IS NULL AND (? IS NULL OR id<>?))")
.bind(c).bind(after).bind(c).bind(after).bind(c).bind(after).bind(c).bind(after).bind(c).bind(c).bind(after).bind(c).bind(after).bind(except_movement).bind(except_movement).bind(c).bind(after).bind(except_lifecycle).bind(except_lifecycle).fetch_one(p).await?)
}
async fn box_free(tx: &mut Transaction<'_, Sqlite>, b: &str) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM boxes WHERE id=? AND status='active') AND NOT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE box_id=? AND ended_at IS NULL)").bind(b).bind(b).fetch_one(&mut**tx).await?)
}
async fn restore_box(
    tx: &mut Transaction<'_, Sqlite>,
    c: &str,
    b: Option<&str>,
    at: &str,
    reason: &str,
) -> Result<(), AppError> {
    let Some(b) = b else { return Ok(()) };
    if !box_free(tx, b).await? {
        return Err(AppError::Validation(
            "A caixa anterior não está ativa e livre; a reversão automática foi bloqueada.".into(),
        ));
    }
    let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL)").bind(c).fetch_one(&mut**tx).await?;
    if active {
        return Err(AppError::Validation(
            "A colônia já possui uma ocupação ativa; a reversão não pode criar duas ocupações."
                .into(),
        ));
    }
    sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at,reason,notes)VALUES(?,?,?,?,?,?)").bind(Uuid::new_v4().to_string()).bind(c).bind(b).bind(at).bind("Retificação: restauração de ocupação").bind(Some(reason)).execute(&mut**tx).await?;
    Ok(())
}

mod lifecycle;
mod movements;

pub use lifecycle::reverse_lifecycle;
pub use movements::reverse_movement;

#[cfg(test)]
mod tests;
