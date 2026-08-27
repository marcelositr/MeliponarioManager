use crate::{dashboard, repository};
use serde::Serialize;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedArtifact {
    pub kind: String,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreStageResult {
    pub source: String,
    pub includes_media: bool,
    pub message: String,
}

fn io_error(context: &str, error: std::io::Error) -> String {
    format!("{context}: {error}")
}

async fn timestamp(pool: &SqlitePool) -> Result<String, String> {
    sqlx::query_scalar::<_, String>("SELECT strftime('%Y%m%d-%H%M%S', 'now')")
        .fetch_one(pool)
        .await
        .map_err(|error| error.to_string())
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(target)
        .map_err(|error| io_error("Não foi possível criar diretório", error))?;
    for entry in
        fs::read_dir(source).map_err(|error| io_error("Não foi possível ler diretório", error))?
    {
        let entry = entry.map_err(|error| io_error("Não foi possível ler item", error))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|error| io_error("Não foi possível identificar item", error))?
            .is_dir()
        {
            copy_tree(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)
                .map_err(|error| io_error("Não foi possível copiar arquivo", error))?;
        }
    }
    Ok(())
}

pub fn apply_pending_restore(data_dir: &Path) -> Result<(), String> {
    let pending_db = data_dir.join("restore.pending.db");
    if !pending_db.exists() {
        return Ok(());
    }

    let current_db = data_dir.join("meliponario.db");
    let pending_media = data_dir.join("restore.pending-media");
    let current_media = data_dir.join("media");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let safety_dir = data_dir
        .join("backups")
        .join(format!("pre-restore-{stamp}"));
    fs::create_dir_all(&safety_dir)
        .map_err(|error| io_error("Não foi possível criar backup de segurança", error))?;

    if current_db.exists() {
        fs::copy(&current_db, safety_dir.join("meliponario.db"))
            .map_err(|error| io_error("Não foi possível preservar o banco atual", error))?;
    }
    if current_media.exists() {
        copy_tree(&current_media, &safety_dir.join("media"))?;
    }

    if current_db.exists() {
        fs::remove_file(&current_db)
            .map_err(|error| io_error("Não foi possível substituir o banco atual", error))?;
    }
    match fs::rename(&pending_db, &current_db) {
        Ok(()) => {}
        Err(_) => {
            fs::copy(&pending_db, &current_db)
                .map_err(|error| io_error("Não foi possível aplicar o banco restaurado", error))?;
            fs::remove_file(&pending_db)
                .map_err(|error| io_error("Não foi possível limpar o banco temporário", error))?;
        }
    }
    let _ = fs::remove_file(data_dir.join("meliponario.db-wal"));
    let _ = fs::remove_file(data_dir.join("meliponario.db-shm"));

    if pending_media.exists() {
        if current_media.exists() {
            fs::remove_dir_all(&current_media)
                .map_err(|error| io_error("Não foi possível substituir a mídia atual", error))?;
        }
        match fs::rename(&pending_media, &current_media) {
            Ok(()) => {}
            Err(_) => {
                copy_tree(&pending_media, &current_media)?;
                fs::remove_dir_all(&pending_media)
                    .map_err(|error| io_error("Não foi possível limpar mídia temporária", error))?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn create_full_backup(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<GeneratedArtifact, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let created_at = timestamp(&pool).await?;
    let suffix = Uuid::new_v4().to_string();
    let target_dir = data_dir
        .join("backups")
        .join(format!("backup-{created_at}-{}", &suffix[..8]));
    fs::create_dir_all(&target_dir)
        .map_err(|error| io_error("Não foi possível criar diretório de backup", error))?;
    let target_db = target_dir.join("meliponario.db");
    let target_db_string = target_db.to_string_lossy().into_owned();
    sqlx::query("VACUUM INTO ?")
        .bind(target_db_string)
        .execute(&*pool)
        .await
        .map_err(|error| error.to_string())?;
    copy_tree(&data_dir.join("media"), &target_dir.join("media"))?;

    Ok(GeneratedArtifact {
        kind: "backup".into(),
        path: target_dir.to_string_lossy().into_owned(),
        created_at,
    })
}

async fn validate_database(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("O backup precisa conter um arquivo meliponario.db válido.".into());
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .create_if_missing(false);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| format!("Não foi possível abrir o banco do backup: {error}"))?;
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(|error| error.to_string())?;
    if !integrity.eq_ignore_ascii_case("ok") {
        pool.close().await;
        return Err(format!(
            "Falha na verificação de integridade do backup: {integrity}"
        ));
    }
    let has_core: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='colonies')",
    )
    .fetch_one(&pool)
    .await
    .map_err(|error| error.to_string())?;
    pool.close().await;
    if !has_core {
        return Err("O arquivo informado não parece ser um banco do MeliponarioManager.".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn stage_restore(
    app: AppHandle,
    backup_path: String,
) -> Result<RestoreStageResult, String> {
    let trimmed = backup_path.trim();
    if trimmed.is_empty() {
        return Err("Informe o diretório do backup ou o arquivo meliponario.db.".into());
    }
    let source = PathBuf::from(trimmed);
    let source_db = if source.is_dir() {
        source.join("meliponario.db")
    } else {
        source.clone()
    };
    validate_database(&source_db).await?;

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&data_dir)
        .map_err(|error| io_error("Não foi possível acessar os dados da aplicação", error))?;
    fs::copy(&source_db, data_dir.join("restore.pending.db"))
        .map_err(|error| io_error("Não foi possível preparar o banco para restauração", error))?;

    let source_media = if source.is_dir() {
        source.join("media")
    } else {
        source.parent().unwrap_or(Path::new("")).join("media")
    };
    let pending_media = data_dir.join("restore.pending-media");
    if pending_media.exists() {
        fs::remove_dir_all(&pending_media)
            .map_err(|error| io_error("Não foi possível limpar mídia pendente", error))?;
    }
    let includes_media = source_media.is_dir();
    if includes_media {
        copy_tree(&source_media, &pending_media)?;
    }

    Ok(RestoreStageResult {
        source: source.to_string_lossy().into_owned(),
        includes_media,
        message: "Restauração validada e preparada. Feche e abra novamente o aplicativo para aplicá-la; o estado atual será preservado em um backup de segurança antes da troca.".into(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortableExport {
    generated_at: String,
    meliponaries: Vec<crate::domain::Meliponary>,
    species: Vec<crate::domain::Species>,
    boxes: Vec<crate::domain::HiveBox>,
    colonies: Vec<crate::domain::Colony>,
    occupancies: Vec<crate::domain::ColonyBoxOccupancy>,
    timeline_by_colony: Vec<ColonyTimelineExport>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ColonyTimelineExport {
    colony_id: String,
    colony_code: String,
    timeline: Vec<crate::history::TimelineEntry>,
}

#[tauri::command]
pub async fn export_portable_json(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<GeneratedArtifact, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let created_at = timestamp(&pool).await?;
    let export_dir = data_dir.join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|error| io_error("Não foi possível criar diretório de exportação", error))?;

    let meliponaries = repository::list_meliponaries(&pool)
        .await
        .map_err(|error| error.to_string())?;
    let species = repository::list_species(&pool)
        .await
        .map_err(|error| error.to_string())?;
    let boxes = repository::list_boxes(&pool)
        .await
        .map_err(|error| error.to_string())?;
    let colonies = repository::list_colonies(&pool)
        .await
        .map_err(|error| error.to_string())?;
    let occupancies = sqlx::query_as::<_, crate::domain::ColonyBoxOccupancy>(
        "SELECT id, colony_id, box_id, started_at, ended_at, reason, notes FROM colony_box_occupancies ORDER BY started_at",
    ).fetch_all(&*pool).await.map_err(|error| error.to_string())?;
    let mut timeline_by_colony = Vec::with_capacity(colonies.len());
    for colony in &colonies {
        let timeline = crate::timeline::by_colony(&pool, &colony.id)
            .await
            .map_err(|error| error.to_string())?;
        timeline_by_colony.push(ColonyTimelineExport {
            colony_id: colony.id.clone(),
            colony_code: colony.code.clone(),
            timeline,
        });
    }

    let export = PortableExport {
        generated_at: created_at.clone(),
        meliponaries,
        species,
        boxes,
        colonies,
        occupancies,
        timeline_by_colony,
    };
    let bytes = serde_json::to_vec_pretty(&export).map_err(|error| error.to_string())?;
    let path = export_dir.join(format!("plantel-{created_at}.json"));
    fs::write(&path, bytes)
        .map_err(|error| io_error("Não foi possível gravar a exportação", error))?;
    Ok(GeneratedArtifact {
        kind: "json".into(),
        path: path.to_string_lossy().into_owned(),
        created_at,
    })
}

#[tauri::command]
pub async fn generate_management_report(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<GeneratedArtifact, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let created_at = timestamp(&pool).await?;
    let export_dir = data_dir.join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|error| io_error("Não foi possível criar diretório de relatórios", error))?;
    let summary = repository::core_summary(&pool)
        .await
        .map_err(|error| error.to_string())?;
    let overview = dashboard::overview(&pool)
        .await
        .map_err(|error| error.to_string())?;

    let mut report = format!("# Relatório do MeliponarioManager\n\nGerado em: {created_at}\n\n## Estrutura\n\n- Meliponários: {}\n- Espécies: {}\n- Colônias: {}\n- Caixas: {}\n- Caixas ocupadas: {}\n- Caixas ativas e livres: {}\n\n## Situação das colônias\n", summary.meliponaries, summary.species, summary.colonies, summary.boxes, overview.occupied_boxes, overview.free_boxes);
    for item in &overview.colony_statuses {
        report.push_str(&format!("- {}: {}\n", item.label, item.count));
    }
    report.push_str("\n## Distribuição por espécie\n");
    for item in &overview.species_distribution {
        report.push_str(&format!("- {}: {}\n", item.label, item.count));
    }
    report.push_str(&format!(
        "\n## Pendências\n\nAlertas atuais: {}\n",
        overview.alerts.len()
    ));
    for alert in overview.alerts.iter().take(20) {
        report.push_str(&format!(
            "- {}: {}{}\n",
            alert.colony_code,
            alert.title,
            alert
                .due_at
                .as_ref()
                .map(|value| format!(" ({value})"))
                .unwrap_or_default()
        ));
    }

    let path = export_dir.join(format!("relatorio-{created_at}.md"));
    fs::write(&path, report)
        .map_err(|error| io_error("Não foi possível gravar o relatório", error))?;
    Ok(GeneratedArtifact {
        kind: "report".into(),
        path: path.to_string_lossy().into_owned(),
        created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validates_migrated_database_for_restore() {
        let root =
            std::env::temp_dir().join(format!("meliponariomanager-restore-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let db = root.join("meliponario.db");
        let pool: SqlitePool = crate::database::initialize(&db).await.unwrap();
        pool.close().await;
        validate_database(&db).await.unwrap();
        let _ = fs::remove_dir_all(&root);
    }
}
