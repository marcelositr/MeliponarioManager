use crate::{
    audit,
    domain::{
        Colony, ColonyBoxOccupancy, CoreSummary, CreateColony, CreateHiveBox, CreateMeliponary,
        CreateSpecies, HiveBox, Meliponary, PlaceColony, Species,
    },
    time,
};
use serde_json::json;
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("Não foi possível acessar os dados locais.")]
    Database(#[from] sqlx::Error),
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

mod entities;
mod occupancy;

pub use entities::{
    core_summary, create_box, create_colony, create_meliponary, create_species, list_boxes,
    list_colonies, list_meliponaries, list_species,
};
pub use occupancy::place_colony;

#[cfg(test)]
mod tests;
