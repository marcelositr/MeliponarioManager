use crate::{
    history::{self, TimelineEntry},
    lifecycle, maintenance,
    repository::AppError,
};
use sqlx::SqlitePool;
pub async fn by_colony(p: &SqlitePool, colony_id: &str) -> Result<Vec<TimelineEntry>, AppError> {
    let mut e = history::timeline_by_colony(p, colony_id).await?;
    e.extend(maintenance::timeline_entries_by_colony(p, colony_id).await?);
    e.extend(lifecycle::timeline_entries(p, colony_id).await?);
    e.extend(sqlx::query_as::<_,TimelineEntry>("SELECT 'division' source_type,d.id source_id,d.performed_at occurred_at,'Divisão de colônia' title,d.notes details,b.code box_code,'info' severity FROM colony_divisions d LEFT JOIN boxes b ON b.id=d.source_box_id WHERE d.parent_colony_id=? OR d.daughter_colony_id=?").bind(colony_id).bind(colony_id).fetch_all(p).await?);
    for item in &mut e {
        decorate(p, item).await?;
    }
    e.sort_by(|a, b| {
        b.occurred_at
            .cmp(&a.occurred_at)
            .then_with(|| b.source_id.cmp(&a.source_id))
    });
    Ok(e)
}
async fn decorate(p: &SqlitePool, e: &mut TimelineEntry) -> Result<(), AppError> {
    match e.source_type.as_str() {
        "inspection" => {
            let s: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
                "SELECT corrected_at,voided_at,void_reason FROM inspections WHERE id=?",
            )
            .bind(&e.source_id)
            .fetch_optional(p)
            .await?;
            if let Some((c, v, r)) = s {
                if v.is_some() {
                    e.title = "Registro anulado · Inspeção".into();
                    e.details = r;
                    e.severity = "attention".into();
                } else if c.is_some() {
                    e.title = "Inspeção corrigida".into();
                }
            }
        }
        "feeding" => {
            decorate_voidable(
                p,
                e,
                "SELECT corrected_at,voided_at,void_reason FROM feedings WHERE id=?",
                "Alimentação corrigida",
                "Registro anulado · Alimentação",
            )
            .await?
        }
        "production" => {
            decorate_voidable(
                p,
                e,
                "SELECT corrected_at,voided_at,void_reason FROM production_records WHERE id=?",
                "Produção corrigida",
                "Registro anulado · Produção",
            )
            .await?
        }
        "event" => {
            decorate_voidable(
                p,
                e,
                "SELECT corrected_at,voided_at,void_reason FROM colony_events WHERE id=?",
                "Evento corrigido",
                "Registro anulado · Evento",
            )
            .await?
        }
        "box_maintenance" => {
            decorate_voidable(
                p,
                e,
                "SELECT corrected_at,voided_at,void_reason FROM box_maintenance_records WHERE id=?",
                "Manutenção corrigida",
                "Registro anulado · Manutenção",
            )
            .await?
        }
        "division" => {
            decorate_voidable(
                p,
                e,
                "SELECT corrected_at,voided_at,void_reason FROM colony_divisions WHERE id=?",
                "Divisão corrigida",
                "Registro anulado · Divisão",
            )
            .await?
        }
        "movement" => {
            let s:Option<(Option<String>,Option<String>,Option<String>,Option<String>,Option<String>)>=sqlx::query_as("SELECT corrected_at,voided_at,void_reason,reversed_at,reversal_reason FROM colony_movements WHERE id=?").bind(&e.source_id).fetch_optional(p).await?;
            if let Some((c, v, vr, r, rr)) = s {
                if r.is_some() {
                    e.title = "Movimentação revertida".into();
                    e.details = rr;
                    e.severity = "attention".into();
                } else if v.is_some() {
                    e.title = "Registro anulado · Movimentação".into();
                    e.details = vr;
                    e.severity = "attention".into();
                } else if c.is_some() {
                    e.title = "Movimentação corrigida".into();
                }
            }
        }
        "lifecycle" => {
            let s: Option<(Option<String>, Option<String>)> = sqlx::query_as(
                "SELECT reversed_at,reversal_reason FROM colony_lifecycle_records WHERE id=?",
            )
            .bind(&e.source_id)
            .fetch_optional(p)
            .await?;
            if let Some((r, reason)) = s {
                if r.is_some() {
                    e.title = "Ciclo de vida revertido".into();
                    e.details = reason;
                    e.severity = "attention".into();
                }
            }
        }
        "box_occupancy" => {
            let c: Option<String> =
                sqlx::query_scalar("SELECT corrected_at FROM colony_box_occupancies WHERE id=?")
                    .bind(&e.source_id)
                    .fetch_optional(p)
                    .await?
                    .flatten();
            if c.is_some() {
                e.title = "Ocupação de caixa corrigida".into();
            }
        }
        _ => {}
    }
    Ok(())
}
async fn decorate_voidable(
    p: &SqlitePool,
    e: &mut TimelineEntry,
    sql: &'static str,
    corrected: &str,
    voided: &str,
) -> Result<(), AppError> {
    let s: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(sql)
        .bind(&e.source_id)
        .fetch_optional(p)
        .await?;
    if let Some((c, v, r)) = s {
        if v.is_some() {
            e.title = voided.to_owned();
            e.details = r;
            e.severity = "attention".into();
        } else if c.is_some() {
            e.title = corrected.to_owned();
        }
    }
    Ok(())
}
