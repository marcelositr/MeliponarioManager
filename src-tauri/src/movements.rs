use crate::{agenda, repository::AppError};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

const MOVEMENT_TYPES: &[&str] = &["internal_transfer", "external_transfer", "transport"];
const MOVABLE_STATUSES: &[&str] = &["active", "weak", "recovering"];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyMovement {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub movement_type: String,
    pub moved_at: String,
    pub from_meliponary_id: String,
    pub from_meliponary_name: String,
    pub to_meliponary_id: Option<String>,
    pub to_meliponary_name: Option<String>,
    pub from_box_id: Option<String>,
    pub from_box_code: Option<String>,
    pub to_box_id: Option<String>,
    pub to_box_code: Option<String>,
    pub destination: Option<String>,
    pub document_reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovement {
    pub colony_id: String,
    pub movement_type: String,
    pub moved_at: Option<String>,
    pub to_meliponary_id: Option<String>,
    pub to_box_id: Option<String>,
    pub destination: Option<String>,
    pub document_reference: Option<String>,
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

fn movement_type(value: &str) -> Result<String, AppError> {
    let value = required(value, "Tipo da movimentação")?;
    if MOVEMENT_TYPES.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "Tipo de movimentação inválido.".to_owned(),
        ))
    }
}

mod creation;
mod queries;

pub use creation::create;
pub use queries::{count, list_by_colony};

#[cfg(test)]
mod tests;
