use crate::{audit, repository::AppError, time};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, SqlitePool};
use tauri::State;
use uuid::Uuid;

const OTHER_OPEN_TRANSPORT_MESSAGE: &str = "Esta colônia já possui outro transporte temporário aberto. Conclua ou anule o transporte atual antes de reabrir este.";

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TransportReturn {
    pub id: String,
    pub movement_id: String,
    pub returned_at: String,
    pub notes: Option<String>,
    pub reversed_at: Option<String>,
    pub reversal_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteTransport {
    pub movement_id: String,
    pub returned_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReopenTransport {
    pub movement_id: String,
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

fn message(error: AppError) -> String {
    error.to_string()
}

fn reopen_write_error(error: sqlx::Error) -> AppError {
    if error
        .to_string()
        .contains("A colônia já possui outro transporte temporário aberto.")
    {
        AppError::Validation(OTHER_OPEN_TRANSPORT_MESSAGE.to_owned())
    } else {
        AppError::Database(error)
    }
}

async fn get_active_return(
    pool: &SqlitePool,
    movement_id: &str,
) -> Result<Option<TransportReturn>, AppError> {
    Ok(sqlx::query_as::<_, TransportReturn>(
        "SELECT id, movement_id, returned_at, notes,
                reversed_at, reversal_reason, created_at
         FROM transport_returns
         WHERE movement_id = ? AND reversed_at IS NULL
         ORDER BY returned_at DESC, created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(movement_id)
    .fetch_optional(pool)
    .await?)
}

mod commands;
mod lifecycle;
mod queries;

pub use commands::{complete_transport, list_transport_returns, reopen_transport};
pub use lifecycle::{complete, has_open_transport_for_colony, reopen};
pub use queries::list_by_colony;

#[cfg(test)]
mod tests;
