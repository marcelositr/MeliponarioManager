use crate::{dashboard, repository};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Column, Row, SqlitePool, TypeInfo,
};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

const BACKUP_FORMAT: &str = "meliponariomanager-backup";
const BACKUP_FORMAT_VERSION: u32 = 1;
const PORTABLE_FORMAT: &str = "meliponariomanager-portable-json";
const PORTABLE_FORMAT_VERSION: u32 = 1;
const CURRENT_SCHEMA_VERSION: i64 = 17;
const MIN_SUPPORTED_SCHEMA_VERSION: i64 = 12;
const DATABASE_FILE: &str = "meliponario.db";
const MEDIA_ROOT: &str = "media";
const BACKUP_MANIFEST: &str = "manifest.json";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupManifest {
    format: String,
    format_version: u32,
    created_at: String,
    app_version: String,
    schema_version: i64,
    database: String,
    media_root: String,
    assets: Vec<BackupAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupAsset {
    relative_path: String,
    byte_size: u64,
    sha256: String,
}

#[derive(Debug, Clone)]
struct ValidatedRestoreSource {
    database: PathBuf,
    media: Option<PathBuf>,
    schema_version: i64,
    manifest: Option<BackupManifest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PortableExport {
    format: &'static str,
    format_version: u32,
    generated_at: String,
    app_version: &'static str,
    schema_version: i64,
    assets_embedded: bool,
    tables: PortableTables,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PortableTables {
    meliponaries: Vec<Value>,
    species: Vec<Value>,
    boxes: Vec<Value>,
    box_state_records: Vec<Value>,
    colonies: Vec<Value>,
    colony_box_occupancies: Vec<Value>,
    inspections: Vec<Value>,
    inspection_photos: Vec<Value>,
    feedings: Vec<Value>,
    production_records: Vec<Value>,
    box_maintenance_records: Vec<Value>,
    colony_events: Vec<Value>,
    colony_divisions: Vec<Value>,
    colony_movements: Vec<Value>,
    transport_returns: Vec<Value>,
    movement_documents: Vec<Value>,
    colony_lifecycle_records: Vec<Value>,
    scheduled_tasks: Vec<Value>,
    audit_records: Vec<Value>,
    managed_attachments: Vec<Value>,
}

fn storage_error(context: &str) -> String {
    context.to_owned()
}

async fn timestamp(pool: &SqlitePool) -> Result<String, String> {
    sqlx::query_scalar::<_, String>("SELECT strftime('%Y%m%d-%H%M%S', 'now', 'localtime')")
        .fetch_one(pool)
        .await
        .map_err(|_| "Não foi possível obter a data para o arquivo gerado.".to_owned())
}

fn ensure_plain_relative_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("O backup contém um caminho de arquivo inválido.".to_owned());
    }
    Ok(())
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|_| storage_error("Não foi possível verificar um arquivo do backup."))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| storage_error("Não foi possível verificar um arquivo do backup."))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(lower_hex(hasher.finalize().as_ref()))
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    if !source.exists() {
        fs::create_dir_all(target)
            .map_err(|_| storage_error("Não foi possível preparar a árvore de arquivos."))?;
        return Ok(());
    }
    fs::create_dir_all(target)
        .map_err(|_| storage_error("Não foi possível preparar a árvore de arquivos."))?;
    let entries = fs::read_dir(source)
        .map_err(|_| storage_error("Não foi possível ler a árvore de arquivos."))?;
    for entry in entries {
        let entry = entry.map_err(|_| storage_error("Não foi possível ler um arquivo."))?;
        let file_type = entry
            .file_type()
            .map_err(|_| storage_error("Não foi possível identificar um arquivo."))?;
        if file_type.is_symlink() {
            return Err("A árvore de arquivos contém um link simbólico não suportado.".to_owned());
        }
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)
                .map_err(|_| storage_error("Não foi possível copiar um arquivo gerenciado."))?;
        }
    }
    Ok(())
}

fn collect_assets(root: &Path) -> Result<Vec<BackupAsset>, String> {
    fn walk(base: &Path, current: &Path, output: &mut Vec<BackupAsset>) -> Result<(), String> {
        if !current.exists() {
            return Ok(());
        }
        let entries = fs::read_dir(current)
            .map_err(|_| storage_error("Não foi possível verificar os arquivos do backup."))?;
        for entry in entries {
            let entry = entry
                .map_err(|_| storage_error("Não foi possível verificar um arquivo do backup."))?;
            let file_type = entry
                .file_type()
                .map_err(|_| storage_error("Não foi possível identificar um arquivo do backup."))?;
            if file_type.is_symlink() {
                return Err(
                    "A árvore de arquivos contém um link simbólico não suportado.".to_owned(),
                );
            }
            let path = entry.path();
            if file_type.is_dir() {
                walk(base, &path, output)?;
            } else if file_type.is_file() {
                let relative = path
                    .strip_prefix(base)
                    .map_err(|_| {
                        storage_error("Não foi possível identificar um arquivo do backup.")
                    })?
                    .to_string_lossy()
                    .replace('\\', "/");
                ensure_plain_relative_path(&relative)?;
                let byte_size = entry
                    .metadata()
                    .map_err(|_| {
                        storage_error("Não foi possível verificar o tamanho de um arquivo.")
                    })?
                    .len();
                let sha256 = sha256_file(&path)?;
                output.push(BackupAsset {
                    relative_path: relative,
                    byte_size,
                    sha256,
                });
            }
        }
        Ok(())
    }

    let mut output = Vec::new();
    walk(root, root, &mut output)?;
    output.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(output)
}

async fn database_schema_version(pool: &SqlitePool) -> Result<i64, String> {
    let version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
            .fetch_one(pool)
            .await
            .map_err(|_| {
                "O banco não possui histórico de schema reconhecido pelo MeliponarioManager."
                    .to_owned()
            })?;
    version.ok_or_else(|| "O banco não possui uma versão de schema reconhecida.".to_owned())
}

mod backup;
mod exports;
mod restore;

pub use backup::create_full_backup;
pub use exports::{export_portable_json, generate_management_report};
pub use restore::{apply_pending_restore, stage_restore};

#[cfg(test)]
mod tests;
