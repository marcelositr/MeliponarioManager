use crate::{
    agenda::{self, AgendaSummary, CreateTask, DuplicateTask, RescheduleTask, ScheduledTask, TaskQuery, TaskReason},
    repository::AppError,
};
use sqlx::SqlitePool;
use tauri::State;

fn message(error: AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn create_task(
    pool: State<'_, SqlitePool>,
    input: CreateTask,
) -> Result<ScheduledTask, String> {
    agenda::create_manual(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_tasks(
    pool: State<'_, SqlitePool>,
    query: TaskQuery,
) -> Result<Vec<ScheduledTask>, String> {
    agenda::list(&pool, query).await.map_err(message)
}

#[tauri::command]
pub async fn get_task(
    pool: State<'_, SqlitePool>,
    task_id: String,
) -> Result<ScheduledTask, String> {
    agenda::get(&pool, &task_id).await.map_err(message)
}

#[tauri::command]
pub async fn get_agenda_summary(
    pool: State<'_, SqlitePool>,
    meliponary_id: Option<String>,
) -> Result<AgendaSummary, String> {
    agenda::summary(&pool, meliponary_id.as_deref())
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn reschedule_task(
    pool: State<'_, SqlitePool>,
    input: RescheduleTask,
) -> Result<ScheduledTask, String> {
    agenda::reschedule(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn cancel_task(
    pool: State<'_, SqlitePool>,
    input: TaskReason,
) -> Result<ScheduledTask, String> {
    agenda::cancel(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn skip_task(
    pool: State<'_, SqlitePool>,
    input: TaskReason,
) -> Result<ScheduledTask, String> {
    agenda::skip(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn complete_generic_task(
    pool: State<'_, SqlitePool>,
    task_id: String,
) -> Result<ScheduledTask, String> {
    agenda::complete_generic(&pool, &task_id).await.map_err(message)
}

#[tauri::command]
pub async fn duplicate_task(
    pool: State<'_, SqlitePool>,
    input: DuplicateTask,
) -> Result<ScheduledTask, String> {
    agenda::duplicate(&pool, input).await.map_err(message)
}
