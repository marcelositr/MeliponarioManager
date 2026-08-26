use crate::{
    alerts::{self, Alert},
    divisions::{self, ColonyDivision, CreateDivision, GenealogyNode},
    documents::{self, CreateMovementDocument, MovementDocument, MovementTraceability},
    domain::{
        Colony, ColonyBoxOccupancy, CoreSummary, CreateColony, CreateHiveBox, CreateMeliponary,
        CreateSpecies, HiveBox, Meliponary, PlaceColony, Species,
    },
    feeding::{self, CreateFeeding, Feeding},
    history::{self, ColonyEvent, CreateColonyEvent, TimelineEntry},
    inspections::{self, CreateInspection, Inspection},
    lifecycle::{self, ChangeColonyLifecycle, ColonyLifecycleRecord},
    maintenance::{self, BoxMaintenance, CreateBoxMaintenance},
    media::{self, ImportInspectionPhoto, InspectionPhoto},
    movements::{self, ColonyMovement, CreateMovement},
    production::{self, CreateProductionRecord, ProductionRecord},
    repository, timeline,
};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

fn message(error: repository::AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn get_core_summary(pool: State<'_, SqlitePool>) -> Result<CoreSummary, String> {
    repository::core_summary(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_inspection_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    inspections::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_inspection_photo_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    media::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_event_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    history::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_division_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    divisions::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_feeding_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    feeding::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_production_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    production::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_movement_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    movements::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_movement_document_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    documents::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_alert_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    alerts::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_box_maintenance_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    maintenance::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn get_lifecycle_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    lifecycle::count(&pool).await.map_err(message)
}

#[tauri::command]
pub async fn list_alerts(pool: State<'_, SqlitePool>) -> Result<Vec<Alert>, String> {
    alerts::list(&pool).await.map_err(message)
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
pub async fn create_box_maintenance(
    pool: State<'_, SqlitePool>,
    input: CreateBoxMaintenance,
) -> Result<BoxMaintenance, String> {
    maintenance::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_box_maintenance(
    pool: State<'_, SqlitePool>,
    box_id: String,
) -> Result<Vec<BoxMaintenance>, String> {
    maintenance::list_by_box(&pool, &box_id)
        .await
        .map_err(message)
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

#[tauri::command]
pub async fn change_colony_lifecycle(
    pool: State<'_, SqlitePool>,
    input: ChangeColonyLifecycle,
) -> Result<ColonyLifecycleRecord, String> {
    lifecycle::change(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colony_lifecycle(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<ColonyLifecycleRecord>, String> {
    lifecycle::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn create_inspection(
    pool: State<'_, SqlitePool>,
    input: CreateInspection,
) -> Result<Inspection, String> {
    inspections::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colony_inspections(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<Inspection>, String> {
    inspections::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn import_inspection_photo(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    input: ImportInspectionPhoto,
) -> Result<InspectionPhoto, String> {
    let data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    media::import_photo(&pool, &data_dir, input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn list_inspection_photos(
    pool: State<'_, SqlitePool>,
    inspection_id: String,
) -> Result<Vec<InspectionPhoto>, String> {
    media::list_by_inspection(&pool, &inspection_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn list_colony_photos(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<InspectionPhoto>, String> {
    media::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn delete_inspection_photo(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    photo_id: String,
) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    media::delete_photo(&pool, &data_dir, &photo_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn create_colony_event(
    pool: State<'_, SqlitePool>,
    input: CreateColonyEvent,
) -> Result<ColonyEvent, String> {
    history::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colony_events(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<ColonyEvent>, String> {
    history::list_events_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn get_colony_timeline(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<TimelineEntry>, String> {
    timeline::by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn create_colony_division(
    pool: State<'_, SqlitePool>,
    input: CreateDivision,
) -> Result<ColonyDivision, String> {
    divisions::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colony_divisions(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<ColonyDivision>, String> {
    divisions::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn get_colony_genealogy(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<GenealogyNode>, String> {
    divisions::genealogy(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn create_feeding(
    pool: State<'_, SqlitePool>,
    input: CreateFeeding,
) -> Result<Feeding, String> {
    feeding::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colony_feedings(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<Feeding>, String> {
    feeding::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn create_production_record(
    pool: State<'_, SqlitePool>,
    input: CreateProductionRecord,
) -> Result<ProductionRecord, String> {
    production::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colony_production(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<ProductionRecord>, String> {
    production::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn create_colony_movement(
    pool: State<'_, SqlitePool>,
    input: CreateMovement,
) -> Result<ColonyMovement, String> {
    movements::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_colony_movements(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<ColonyMovement>, String> {
    movements::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn create_movement_document(
    pool: State<'_, SqlitePool>,
    input: CreateMovementDocument,
) -> Result<MovementDocument, String> {
    documents::create(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_movement_documents(
    pool: State<'_, SqlitePool>,
    movement_id: String,
) -> Result<Vec<MovementDocument>, String> {
    documents::list_by_movement(&pool, &movement_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn list_colony_documents(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<MovementDocument>, String> {
    documents::list_by_colony(&pool, &colony_id)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn get_movement_traceability(
    pool: State<'_, SqlitePool>,
    movement_id: String,
) -> Result<MovementTraceability, String> {
    documents::traceability(&pool, &movement_id)
        .await
        .map_err(message)
}
