use crate::{
    agenda, audit,
    domain::{Colony, HiveBox, Meliponary, Species},
    repository::AppError,
};
use serde::Deserialize;
use sqlx::{Sqlite, SqlitePool, Transaction};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditMeliponary {
    pub id: String,
    pub name: String,
    pub responsible_name: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSpecies {
    pub id: String,
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub genus: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditBox {
    pub id: String,
    pub code: String,
    pub model: Option<String>,
    pub material: Option<String>,
    pub location_note: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditColony {
    pub id: String,
    pub code: String,
    pub origin_notes: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityAction {
    pub id: String,
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

async fn local_now(tx: &mut Transaction<'_, Sqlite>) -> Result<String, AppError> {
    Ok(sqlx::query_scalar("SELECT datetime('now', 'localtime')")
        .fetch_one(&mut **tx)
        .await?)
}

async fn get_meliponary(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<Meliponary, AppError> {
    sqlx::query_as::<_, Meliponary>(
        "SELECT id, name, responsible_name, location, notes, archived_at, archive_reason, created_at
         FROM meliponaries WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Meliponário não encontrado.".to_owned()))
}

async fn get_species(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<Species, AppError> {
    sqlx::query_as::<_, Species>(
        "SELECT id, common_name, scientific_name, genus, notes, archived_at, archive_reason, created_at
         FROM species WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Espécie não encontrada.".to_owned()))
}

async fn get_box(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<HiveBox, AppError> {
    sqlx::query_as::<_, HiveBox>(
        "SELECT b.id, b.meliponary_id, b.code, b.model, b.material, b.location_note,
                b.status, b.notes, c.code AS current_colony_code, b.created_at
         FROM boxes b
         LEFT JOIN colony_box_occupancies o ON o.box_id = b.id AND o.ended_at IS NULL
         LEFT JOIN colonies c ON c.id = o.colony_id
         WHERE b.id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))
}

async fn get_colony(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<Colony, AppError> {
    sqlx::query_as::<_, Colony>(
        "SELECT c.id, c.meliponary_id, c.species_id, c.code, c.origin_type,
                c.origin_notes, c.installed_at, c.status, c.mother_colony_id, c.notes,
                b.code AS current_box_code, c.created_at
         FROM colonies c
         LEFT JOIN colony_box_occupancies o ON o.colony_id = c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id = o.box_id
         WHERE c.id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))
}

mod boxes;
mod colonies;
mod meliponaries;
mod species;

pub use boxes::{delete_box, edit_box};
pub use colonies::{delete_colony, edit_colony};
pub use meliponaries::{
    archive_meliponary, delete_meliponary, edit_meliponary, reactivate_meliponary,
};
pub use species::{archive_species, delete_species, edit_species, reactivate_species};

#[cfg(test)]
mod tests;
