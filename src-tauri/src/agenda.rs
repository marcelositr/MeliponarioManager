use crate::{audit, operational, repository::AppError, time};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

const TASK_TYPES: &[&str] = &["inspection", "feeding", "maintenance", "generic"];
const PRIORITIES: &[&str] = &["normal", "attention", "critical"];
const VIEWS: &[&str] = &[
    "all",
    "pending",
    "overdue",
    "today",
    "upcoming",
    "completed",
    "cancelled",
    "rescheduled",
    "skipped",
];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: String,
    pub meliponary_id: String,
    pub meliponary_name: String,
    pub colony_id: Option<String>,
    pub colony_code: Option<String>,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub task_type: String,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_for: String,
    pub status: String,
    pub priority: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub completed_at: Option<String>,
    pub completed_by_type: Option<String>,
    pub completed_by_id: Option<String>,
    pub cancelled_at: Option<String>,
    pub cancellation_reason: Option<String>,
    pub skipped_at: Option<String>,
    pub skip_reason: Option<String>,
    pub rescheduled_from_id: Option<String>,
    pub reschedule_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub overdue: bool,
    pub today: bool,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgendaSummary {
    pub overdue: i64,
    pub today: i64,
    pub next_seven_days: i64,
    pub future: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTask {
    pub meliponary_id: String,
    pub colony_id: Option<String>,
    pub box_id: Option<String>,
    pub task_type: String,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_for: String,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskQuery {
    pub view: Option<String>,
    pub meliponary_id: Option<String>,
    pub colony_id: Option<String>,
    pub box_id: Option<String>,
    pub task_type: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RescheduleTask {
    pub id: String,
    pub scheduled_for: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReason {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateTask {
    pub id: String,
    pub scheduled_for: String,
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

fn valid_task_type(value: &str) -> Result<(), AppError> {
    if TASK_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation("Tipo de tarefa inválido.".to_owned()))
    }
}

fn valid_priority(value: &str) -> Result<(), AppError> {
    if PRIORITIES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation("Prioridade inválida.".to_owned()))
    }
}

async fn now_tx(tx: &mut Transaction<'_, Sqlite>) -> Result<String, AppError> {
    Ok(
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')")
            .fetch_one(&mut **tx)
            .await?,
    )
}

async fn task_snapshot_tx(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<Value, AppError> {
    let raw: Option<String> = sqlx::query_scalar(
        "SELECT json_object(
            'id', id, 'meliponary_id', meliponary_id, 'colony_id', colony_id,
            'box_id', box_id, 'task_type', task_type, 'title', title,
            'description', description, 'scheduled_for', scheduled_for,
            'status', status, 'priority', priority, 'source_type', source_type,
            'source_id', source_id, 'completed_at', completed_at,
            'completed_by_type', completed_by_type, 'completed_by_id', completed_by_id,
            'cancelled_at', cancelled_at, 'cancellation_reason', cancellation_reason,
            'skipped_at', skipped_at, 'skip_reason', skip_reason,
            'rescheduled_from_id', rescheduled_from_id, 'reschedule_reason', reschedule_reason
         ) FROM scheduled_tasks WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    let raw = raw.ok_or_else(|| AppError::NotFound("Tarefa não encontrada.".to_owned()))?;
    serde_json::from_str(&raw).map_err(|error| {
        AppError::Validation(format!(
            "Não foi possível preparar a auditoria da Agenda: {error}"
        ))
    })
}

mod derived;
mod manual;
mod queries;

pub use derived::{
    mark_completed_by_fact_tx, reconcile_all, reconcile_feeding, reconcile_inspection,
    reconcile_maintenance,
};
pub use manual::{cancel, complete_generic, create_manual, duplicate, reschedule, skip};
pub use queries::{get, list, summary};

#[cfg(test)]
use queries::summary_at;

#[cfg(test)]
mod tests;
