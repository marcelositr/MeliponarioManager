mod database;

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
        .run(tauri::generate_context!())
        .expect("error while running MeliponarioManager");
}
