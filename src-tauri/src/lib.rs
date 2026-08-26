use serde::Serialize;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use std::str::FromStr;
use tauri::{Manager, State};

#[derive(Debug, Serialize)]
struct AppStatus {
    app_name: &'static str,
    version: &'static str,
    database: &'static str,
}

#[tauri::command]
async fn app_status(pool: State<'_, Pool<Sqlite>>) -> Result<AppStatus, String> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool.inner())
        .await
        .map_err(|error| error.to_string())?;

    Ok(AppStatus {
        app_name: "MeliponarioManager",
        version: env!("CARGO_PKG_VERSION"),
        database: "SQLite conectado",
    })
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let database_path = app_data_dir.join("meliponario.db");
            let database_url = format!("sqlite://{}", database_path.to_string_lossy());
            let options = SqliteConnectOptions::from_str(&database_url)?
                .create_if_missing(true)
                .foreign_keys(true);

            let pool = tauri::async_runtime::block_on(async {
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect_with(options)
                    .await?;

                sqlx::migrate!("./migrations").run(&pool).await?;

                Ok::<_, sqlx::Error>(pool)
            })?;

            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app_status])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o MeliponarioManager");
}
