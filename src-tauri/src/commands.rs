use crate::{
    domain::{
        Colony, ColonyBoxOccupancy, CoreSummary, CreateColony, CreateHiveBox, CreateMeliponary,
        CreateSpecies, HiveBox, Meliponary, PlaceColony, Species,
    },
    repository,
};
use sqlx::SqlitePool;
use tauri::State;

fn message(error: repository::AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn get_core_summary(pool: State<'_, SqlitePool>) -> Result<CoreSummary, String> {
    repository::core_summary(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn create_meliponary(
    pool: State<'_, SqlitePool>,
    input: CreateMeliponary,
) -> Result<Meliponary, String> {
    repository::create_meliponary(&pool, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn list_meliponaries(pool: State<'_, SqlitePool>) -> Result<Vec<Meliponary>, String> {
    repository::list_meliponaries(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn create_species(
    pool: State<'_, SqlitePool>,
    input: CreateSpecies,
) -> Result<Species, String> {
    repository::create_species(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_species(pool: State<'_, SqlitePool>) -> Result<Vec<Species>, String> {
    repository::list_species(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn create_box(
    pool: State<'_, SqlitePool>,
    input: CreateHiveBox,
) -> Result<HiveBox, String> {
    repository::create_box(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_boxes(pool: State<'_, SqlitePool>) -> Result<Vec<HiveBox>, String> {
    repository::list_boxes(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn create_colony(
    pool: State<'_, SqlitePool>,
    input: CreateColony,
) -> Result<Colony, String> {
    repository::create_colony(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colonies(pool: State<'_, SqlitePool>) -> Result<Vec<Colony>, String> {
    repository::list_colonies(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn place_colony(
    pool: State<'_, SqlitePool>,
    input: PlaceColony,
) -> Result<ColonyBoxOccupancy, String> {
    repository::place_colony(&pool, input).await.map_err(message)
}
