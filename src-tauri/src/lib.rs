mod commands;
mod database;
mod divisions;
mod domain;
mod feeding;
mod history;
mod inspections;
mod repository;

use sqlx::SqlitePool;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let database_path = data_dir.join("meliponario.db");
            let pool: SqlitePool = tauri::async_runtime::block_on(database::initialize(&database_path))?;
            app.manage(pool);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_core_summary,
            commands::get_inspection_count,
            commands::get_event_count,
            commands::get_division_count,
            commands::get_feeding_count,
            commands::create_meliponary,
            commands::list_meliponaries,
            commands::create_species,
            commands::list_species,
            commands::create_box,
            commands::list_boxes,
            commands::create_colony,
            commands::list_colonies,
            commands::place_colony,
            commands::create_inspection,
            commands::list_colony_inspections,
            commands::create_colony_event,
            commands::list_colony_events,
            commands::get_colony_timeline,
            commands::create_colony_division,
            commands::list_colony_divisions,
            commands::get_colony_genealogy,
            commands::create_feeding,
            commands::list_colony_feedings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MeliponarioManager");
}
