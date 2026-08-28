mod admin_commands;
mod agenda;
mod agenda_commands;
mod alerts;
mod audit;
mod box_states;
mod commands;
mod dashboard;
mod data_management;
mod database;
mod divisions;
mod documents;
mod domain;
mod feeding;
mod history;
mod inspections;
mod lifecycle;
mod maintenance;
mod master_data;
mod media;
#[cfg(test)]
mod migration_tests;
mod movements;
mod operational;
mod production;
mod record_corrections;
mod record_states;
mod repository;
mod reversals;
#[cfg(test)]
mod stage4_migration_tests;
mod time;
mod timeline;

use sqlx::SqlitePool;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            data_management::apply_pending_restore(&data_dir).map_err(std::io::Error::other)?;
            std::fs::create_dir_all(data_dir.join("media").join("inspections"))?;

            let database_path = data_dir.join("meliponario.db");
            let pool: SqlitePool =
                tauri::async_runtime::block_on(database::initialize(&database_path))?;
            app.manage(pool);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_core_summary,
            commands::get_inspection_count,
            commands::get_inspection_photo_count,
            commands::get_event_count,
            commands::get_division_count,
            commands::get_feeding_count,
            commands::get_production_count,
            commands::get_movement_count,
            commands::get_movement_document_count,
            commands::get_alert_count,
            commands::get_box_maintenance_count,
            commands::get_lifecycle_count,
            dashboard::get_dashboard_overview,
            data_management::create_full_backup,
            data_management::export_portable_json,
            data_management::generate_management_report,
            data_management::stage_restore,
            commands::list_alerts,
            commands::create_meliponary,
            commands::list_meliponaries,
            commands::create_species,
            commands::list_species,
            commands::create_box,
            commands::list_boxes,
            commands::change_box_state,
            commands::list_box_state_history,
            commands::create_box_maintenance,
            commands::list_box_maintenance,
            commands::create_colony,
            commands::list_colonies,
            commands::place_colony,
            commands::change_colony_lifecycle,
            commands::list_colony_lifecycle,
            commands::create_inspection,
            commands::list_colony_inspections,
            commands::import_inspection_photo,
            commands::list_inspection_photos,
            commands::list_colony_photos,
            commands::delete_inspection_photo,
            commands::create_colony_event,
            commands::list_colony_events,
            commands::get_colony_timeline,
            commands::create_colony_division,
            commands::list_colony_divisions,
            commands::get_colony_genealogy,
            commands::create_feeding,
            commands::list_colony_feedings,
            commands::create_production_record,
            commands::list_colony_production,
            commands::create_colony_movement,
            commands::list_colony_movements,
            commands::create_movement_document,
            commands::list_movement_documents,
            commands::list_colony_documents,
            commands::get_movement_traceability,
            admin_commands::edit_meliponary,
            admin_commands::archive_meliponary,
            admin_commands::reactivate_meliponary,
            admin_commands::delete_meliponary,
            admin_commands::edit_species,
            admin_commands::archive_species,
            admin_commands::reactivate_species,
            admin_commands::delete_species,
            admin_commands::edit_box,
            admin_commands::delete_box,
            admin_commands::edit_colony,
            admin_commands::delete_colony,
            admin_commands::list_audit_records,
            admin_commands::correct_inspection,
            admin_commands::void_inspection,
            admin_commands::correct_feeding,
            admin_commands::void_feeding,
            admin_commands::correct_production_record,
            admin_commands::void_production_record,
            admin_commands::correct_box_maintenance,
            admin_commands::void_box_maintenance,
            admin_commands::correct_colony_event,
            admin_commands::void_colony_event,
            admin_commands::correct_movement_details,
            admin_commands::void_transport,
            admin_commands::update_movement_document,
            admin_commands::void_movement_document,
            admin_commands::correct_colony_division,
            admin_commands::void_colony_division,
            admin_commands::correct_box_occupancy,
            admin_commands::reverse_colony_lifecycle,
            admin_commands::reverse_colony_movement,
            record_states::list_record_admin_states,
            agenda_commands::create_task,
            agenda_commands::list_tasks,
            agenda_commands::get_task,
            agenda_commands::get_agenda_summary,
            agenda_commands::reschedule_task,
            agenda_commands::cancel_task,
            agenda_commands::skip_task,
            agenda_commands::complete_generic_task,
            agenda_commands::duplicate_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MeliponarioManager");
}
