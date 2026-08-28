use crate::{
    audit::{self, AuditRecord},
    domain::{Colony, HiveBox, Meliponary, Species},
    master_data::{self, EditBox, EditColony, EditMeliponary, EditSpecies, EntityAction},
    record_corrections::{
        self, CorrectDivision, CorrectEvent, CorrectFeeding, CorrectInspection, CorrectMaintenance,
        CorrectMovementDetails, CorrectOccupancy, CorrectProduction, UpdateMovementDocument,
        VoidDivision, VoidRecord,
    },
    repository,
    reversals::{self, ReverseRecord},
};
use sqlx::SqlitePool;
use tauri::State;

fn message(error: repository::AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn edit_meliponary(
    pool: State<'_, SqlitePool>,
    input: EditMeliponary,
) -> Result<Meliponary, String> {
    master_data::edit_meliponary(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn archive_meliponary(
    pool: State<'_, SqlitePool>,
    input: EntityAction,
) -> Result<Meliponary, String> {
    master_data::archive_meliponary(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn reactivate_meliponary(
    pool: State<'_, SqlitePool>,
    input: EntityAction,
) -> Result<Meliponary, String> {
    master_data::reactivate_meliponary(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn delete_meliponary(
    pool: State<'_, SqlitePool>,
    input: EntityAction,
) -> Result<(), String> {
    master_data::delete_meliponary(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn edit_species(
    pool: State<'_, SqlitePool>,
    input: EditSpecies,
) -> Result<Species, String> {
    master_data::edit_species(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn archive_species(
    pool: State<'_, SqlitePool>,
    input: EntityAction,
) -> Result<Species, String> {
    master_data::archive_species(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn reactivate_species(
    pool: State<'_, SqlitePool>,
    input: EntityAction,
) -> Result<Species, String> {
    master_data::reactivate_species(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn delete_species(
    pool: State<'_, SqlitePool>,
    input: EntityAction,
) -> Result<(), String> {
    master_data::delete_species(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn edit_box(pool: State<'_, SqlitePool>, input: EditBox) -> Result<HiveBox, String> {
    master_data::edit_box(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn delete_box(pool: State<'_, SqlitePool>, input: EntityAction) -> Result<(), String> {
    master_data::delete_box(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn edit_colony(pool: State<'_, SqlitePool>, input: EditColony) -> Result<Colony, String> {
    master_data::edit_colony(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn delete_colony(pool: State<'_, SqlitePool>, input: EntityAction) -> Result<(), String> {
    master_data::delete_colony(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn list_audit_records(
    pool: State<'_, SqlitePool>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<AuditRecord>, String> {
    audit::list_by_entity(&pool, &entity_type, &entity_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_inspection(
    pool: State<'_, SqlitePool>,
    input: CorrectInspection,
) -> Result<(), String> {
    record_corrections::correct_inspection(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_inspection(pool: State<'_, SqlitePool>, input: VoidRecord) -> Result<(), String> {
    record_corrections::void_inspection(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_feeding(
    pool: State<'_, SqlitePool>,
    input: CorrectFeeding,
) -> Result<(), String> {
    record_corrections::correct_feeding(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_feeding(pool: State<'_, SqlitePool>, input: VoidRecord) -> Result<(), String> {
    record_corrections::void_feeding(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_production_record(
    pool: State<'_, SqlitePool>,
    input: CorrectProduction,
) -> Result<(), String> {
    record_corrections::correct_production(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_production_record(
    pool: State<'_, SqlitePool>,
    input: VoidRecord,
) -> Result<(), String> {
    record_corrections::void_production(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_box_maintenance(
    pool: State<'_, SqlitePool>,
    input: CorrectMaintenance,
) -> Result<(), String> {
    record_corrections::correct_maintenance(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_box_maintenance(
    pool: State<'_, SqlitePool>,
    input: VoidRecord,
) -> Result<(), String> {
    record_corrections::void_maintenance(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_colony_event(
    pool: State<'_, SqlitePool>,
    input: CorrectEvent,
) -> Result<(), String> {
    record_corrections::correct_event(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_colony_event(
    pool: State<'_, SqlitePool>,
    input: VoidRecord,
) -> Result<(), String> {
    record_corrections::void_event(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_movement_details(
    pool: State<'_, SqlitePool>,
    input: CorrectMovementDetails,
) -> Result<(), String> {
    record_corrections::correct_movement_details(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_transport(pool: State<'_, SqlitePool>, input: VoidRecord) -> Result<(), String> {
    record_corrections::void_transport(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn update_movement_document(
    pool: State<'_, SqlitePool>,
    input: UpdateMovementDocument,
) -> Result<(), String> {
    record_corrections::update_movement_document(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_movement_document(
    pool: State<'_, SqlitePool>,
    input: VoidRecord,
) -> Result<(), String> {
    record_corrections::void_movement_document(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_colony_division(
    pool: State<'_, SqlitePool>,
    input: CorrectDivision,
) -> Result<(), String> {
    record_corrections::correct_division(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn void_colony_division(
    pool: State<'_, SqlitePool>,
    input: VoidDivision,
) -> Result<(), String> {
    record_corrections::void_division(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn correct_box_occupancy(
    pool: State<'_, SqlitePool>,
    input: CorrectOccupancy,
) -> Result<(), String> {
    record_corrections::correct_occupancy(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn reverse_colony_lifecycle(
    pool: State<'_, SqlitePool>,
    input: ReverseRecord,
) -> Result<(), String> {
    reversals::reverse_lifecycle(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn reverse_colony_movement(
    pool: State<'_, SqlitePool>,
    input: ReverseRecord,
) -> Result<(), String> {
    reversals::reverse_movement(&pool, input)
        .await
        .map_err(message)
}
