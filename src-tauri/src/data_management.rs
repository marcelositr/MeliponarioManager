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
    Ok(format!("{:x}", hasher.finalize()))
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

async fn validate_database(path: &Path) -> Result<i64, String> {
    if !path.is_file() {
        return Err("O backup precisa conter um arquivo meliponario.db válido.".to_owned());
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .create_if_missing(false);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| "Não foi possível abrir o banco do backup.".to_owned())?;

    let validation = async {
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&pool)
            .await
            .map_err(|_| "Não foi possível verificar a integridade do banco do backup.".to_owned())?;
        if !integrity.eq_ignore_ascii_case("ok") {
            return Err("O banco do backup está corrompido ou inconsistente.".to_owned());
        }

        let required_tables = [
            "meliponaries",
            "species",
            "boxes",
            "colonies",
            "colony_box_occupancies",
            "inspections",
            "inspection_photos",
        ];
        for table in required_tables {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?)",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .map_err(|_| "Não foi possível validar a estrutura do banco do backup.".to_owned())?;
            if !exists {
                return Err(
                    "O banco informado não possui a estrutura esperada do MeliponarioManager."
                        .to_owned(),
                );
            }
        }

        let schema_version = database_schema_version(&pool).await?;
        if !(MIN_SUPPORTED_SCHEMA_VERSION..=CURRENT_SCHEMA_VERSION).contains(&schema_version) {
            return Err(format!(
                "A versão de dados deste backup não é compatível. São aceitos schemas de {MIN_SUPPORTED_SCHEMA_VERSION} a {CURRENT_SCHEMA_VERSION}."
            ));
        }
        Ok(schema_version)
    }
    .await;
    pool.close().await;
    validation
}

fn read_manifest(path: &Path) -> Result<BackupManifest, String> {
    let bytes =
        fs::read(path).map_err(|_| "Não foi possível ler o manifest do backup.".to_owned())?;
    serde_json::from_slice(&bytes).map_err(|_| "O manifest do backup é inválido.".to_owned())
}

fn validate_manifest(
    root: &Path,
    manifest: &BackupManifest,
    database_schema: i64,
) -> Result<(), String> {
    if manifest.format != BACKUP_FORMAT || manifest.format_version != BACKUP_FORMAT_VERSION {
        return Err(
            "O formato deste backup não é compatível com esta versão do aplicativo.".to_owned(),
        );
    }
    if manifest.database != DATABASE_FILE || manifest.media_root != MEDIA_ROOT {
        return Err("O manifest do backup possui estrutura de arquivos incompatível.".to_owned());
    }
    if manifest.schema_version != database_schema {
        return Err(
            "O manifest e o banco do backup declaram versões de schema diferentes.".to_owned(),
        );
    }
    if manifest.schema_version > CURRENT_SCHEMA_VERSION {
        return Err("Este backup foi criado por uma estrutura de dados mais nova.".to_owned());
    }

    let media_root = root.join(MEDIA_ROOT);
    if !media_root.is_dir() {
        return Err("O backup completo não contém a pasta de arquivos gerenciados.".to_owned());
    }

    let mut declared = BTreeSet::new();
    for asset in &manifest.assets {
        ensure_plain_relative_path(&asset.relative_path)?;
        let relative = Path::new(&asset.relative_path);
        if relative
            .components()
            .next()
            .and_then(|component| match component {
                Component::Normal(value) => value.to_str(),
                _ => None,
            })
            != Some(MEDIA_ROOT)
        {
            return Err("O manifest declara um arquivo fora da área de mídia.".to_owned());
        }
        if !declared.insert(asset.relative_path.clone()) {
            return Err("O manifest declara o mesmo arquivo mais de uma vez.".to_owned());
        }
        let full_path = root.join(relative);
        let metadata = fs::metadata(&full_path).map_err(|_| {
            format!(
                "O backup está incompleto: falta o arquivo {}.",
                asset.relative_path
            )
        })?;
        if !metadata.is_file() || metadata.len() != asset.byte_size {
            return Err(format!(
                "O backup está inconsistente: o arquivo {} não corresponde ao manifest.",
                asset.relative_path
            ));
        }
        if sha256_file(&full_path)? != asset.sha256 {
            return Err(format!(
                "O backup está inconsistente: o conteúdo do arquivo {} foi alterado.",
                asset.relative_path
            ));
        }
    }

    let actual = collect_assets(&media_root)?
        .into_iter()
        .map(|asset| format!("{MEDIA_ROOT}/{}", asset.relative_path))
        .collect::<BTreeSet<_>>();
    if actual != declared {
        return Err(
            "O conteúdo da pasta de mídia não corresponde ao manifest do backup.".to_owned(),
        );
    }
    Ok(())
}

async fn validate_restore_source(source: &Path) -> Result<ValidatedRestoreSource, String> {
    if source.is_dir() {
        let database = source.join(DATABASE_FILE);
        let schema_version = validate_database(&database).await?;
        let manifest_path = source.join(BACKUP_MANIFEST);
        if manifest_path.is_file() {
            let manifest = read_manifest(&manifest_path)?;
            validate_manifest(source, &manifest, schema_version)?;
            return Ok(ValidatedRestoreSource {
                database,
                media: Some(source.join(MEDIA_ROOT)),
                schema_version,
                manifest: Some(manifest),
            });
        }

        let media = source.join(MEDIA_ROOT);
        return Ok(ValidatedRestoreSource {
            database,
            media: media.is_dir().then_some(media),
            schema_version,
            manifest: None,
        });
    }

    let schema_version = validate_database(source).await?;
    Ok(ValidatedRestoreSource {
        database: source.to_path_buf(),
        media: None,
        schema_version,
        manifest: None,
    })
}

fn write_backup_manifest(
    target_dir: &Path,
    created_at: &str,
    schema_version: i64,
) -> Result<BackupManifest, String> {
    let media_dir = target_dir.join(MEDIA_ROOT);
    let assets = collect_assets(&media_dir)?
        .into_iter()
        .map(|asset| BackupAsset {
            relative_path: format!("{MEDIA_ROOT}/{}", asset.relative_path),
            byte_size: asset.byte_size,
            sha256: asset.sha256,
        })
        .collect();
    let manifest = BackupManifest {
        format: BACKUP_FORMAT.to_owned(),
        format_version: BACKUP_FORMAT_VERSION,
        created_at: created_at.to_owned(),
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        schema_version,
        database: DATABASE_FILE.to_owned(),
        media_root: MEDIA_ROOT.to_owned(),
        assets,
    };
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| "Não foi possível gerar o manifest do backup.".to_owned())?;
    fs::write(target_dir.join(BACKUP_MANIFEST), bytes)
        .map_err(|_| "Não foi possível gravar o manifest do backup.".to_owned())?;
    Ok(manifest)
}

async fn create_backup_at(
    pool: &SqlitePool,
    data_dir: &Path,
    target_dir: &Path,
    created_at: &str,
) -> Result<BackupManifest, String> {
    if target_dir.exists() {
        return Err("O destino do backup já existe.".to_owned());
    }
    fs::create_dir_all(target_dir)
        .map_err(|_| "Não foi possível criar a pasta do backup.".to_owned())?;

    let result = async {
        let target_db = target_dir.join(DATABASE_FILE);
        let target_db_string = target_db.to_string_lossy().into_owned();
        sqlx::query("VACUUM INTO ?")
            .bind(target_db_string)
            .execute(pool)
            .await
            .map_err(|_| "Não foi possível criar a cópia consistente do banco.".to_owned())?;
        copy_tree(&data_dir.join(MEDIA_ROOT), &target_dir.join(MEDIA_ROOT))?;
        let schema_version = database_schema_version(pool).await?;
        write_backup_manifest(target_dir, created_at, schema_version)
    }
    .await;

    if result.is_err() {
        let _ = fs::remove_dir_all(target_dir);
    }
    result
}

fn remove_if_exists(path: &Path) -> Result<(), String> {
    if path.is_file() {
        fs::remove_file(path)
            .map_err(|_| "Não foi possível limpar um arquivo temporário.".to_owned())?;
    } else if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|_| "Não foi possível limpar uma pasta temporária.".to_owned())?;
    }
    Ok(())
}

async fn stage_restore_at(data_dir: &Path, source: &Path) -> Result<RestoreStageResult, String> {
    let validated = validate_restore_source(source).await?;
    fs::create_dir_all(data_dir)
        .map_err(|_| "Não foi possível acessar os dados da aplicação.".to_owned())?;

    let pending_db = data_dir.join("restore.pending.db");
    let pending_media = data_dir.join("restore.pending-media");
    let pending_manifest = data_dir.join("restore.pending-manifest.json");
    remove_if_exists(&pending_db)?;
    remove_if_exists(&pending_media)?;
    remove_if_exists(&pending_manifest)?;

    if fs::copy(&validated.database, &pending_db).is_err() {
        return Err("Não foi possível preparar o banco para restauração.".to_owned());
    }
    if let Some(media) = validated.media.as_ref() {
        if let Err(error) = copy_tree(media, &pending_media) {
            let _ = fs::remove_file(&pending_db);
            return Err(error);
        }
    }
    if let Some(manifest) = validated.manifest.as_ref() {
        let bytes = serde_json::to_vec_pretty(manifest)
            .map_err(|_| "Não foi possível preparar o manifest da restauração.".to_owned())?;
        if fs::write(&pending_manifest, bytes).is_err() {
            let _ = fs::remove_file(&pending_db);
            let _ = fs::remove_dir_all(&pending_media);
            return Err("Não foi possível preparar o manifest da restauração.".to_owned());
        }
    }

    let includes_media = validated.media.is_some();
    let source_description = if validated.manifest.is_some() {
        "backup completo com manifest"
    } else if includes_media {
        "backup legado com mídia"
    } else {
        "banco legado sem mídia"
    };
    Ok(RestoreStageResult {
        source: source.to_string_lossy().into_owned(),
        includes_media,
        message: format!(
            "Restauração validada ({source_description}, schema {}). Feche e abra novamente o aplicativo para aplicá-la; antes da troca o estado atual será preservado em um backup de segurança.",
            validated.schema_version
        ),
    })
}

fn restore_rollback(
    current_db: &Path,
    current_media: &Path,
    rollback_db: &Path,
    rollback_media: &Path,
) {
    let _ = remove_if_exists(current_db);
    if rollback_db.exists() {
        let _ = fs::rename(rollback_db, current_db);
    }
    if rollback_media.exists() {
        let _ = remove_if_exists(current_media);
        let _ = fs::rename(rollback_media, current_media);
    }
}

pub fn apply_pending_restore(data_dir: &Path) -> Result<(), String> {
    let pending_db = data_dir.join("restore.pending.db");
    if !pending_db.is_file() {
        return Ok(());
    }

    let current_db = data_dir.join(DATABASE_FILE);
    let current_media = data_dir.join(MEDIA_ROOT);
    let pending_media = data_dir.join("restore.pending-media");
    let pending_manifest = data_dir.join("restore.pending-manifest.json");
    let rollback_db = data_dir.join("restore.rollback.db");
    let rollback_media = data_dir.join("restore.rollback-media");
    remove_if_exists(&rollback_db)?;
    remove_if_exists(&rollback_media)?;

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "Não foi possível preparar a restauração.".to_owned())?
        .as_secs();
    let safety_dir = data_dir
        .join("backups")
        .join(format!("pre-restore-{stamp}"));
    fs::create_dir_all(&safety_dir).map_err(|_| {
        "Não foi possível criar o backup de segurança antes da restauração.".to_owned()
    })?;
    if current_db.is_file() {
        fs::copy(&current_db, safety_dir.join(DATABASE_FILE)).map_err(|_| {
            "Não foi possível preservar o banco atual antes da restauração.".to_owned()
        })?;
    }
    if current_media.is_dir() {
        copy_tree(&current_media, &safety_dir.join(MEDIA_ROOT))?;
    } else {
        fs::create_dir_all(safety_dir.join(MEDIA_ROOT))
            .map_err(|_| "Não foi possível preservar a área de arquivos atual.".to_owned())?;
    }

    if current_db.is_file() {
        fs::rename(&current_db, &rollback_db)
            .map_err(|_| "Não foi possível preparar a troca segura do banco atual.".to_owned())?;
    }
    if fs::rename(&pending_db, &current_db).is_err() {
        restore_rollback(&current_db, &current_media, &rollback_db, &rollback_media);
        return Err(
            "Não foi possível aplicar o banco restaurado; o estado anterior foi preservado."
                .to_owned(),
        );
    }

    if pending_media.is_dir() {
        if current_media.is_dir() && fs::rename(&current_media, &rollback_media).is_err() {
            restore_rollback(&current_db, &current_media, &rollback_db, &rollback_media);
            return Err("Não foi possível preparar a troca segura dos arquivos; o estado anterior foi preservado.".to_owned());
        }
        if fs::rename(&pending_media, &current_media).is_err() {
            restore_rollback(&current_db, &current_media, &rollback_db, &rollback_media);
            return Err(
                "Não foi possível aplicar os arquivos restaurados; o estado anterior foi preservado."
                    .to_owned(),
            );
        }
    }

    remove_if_exists(&rollback_db)?;
    remove_if_exists(&rollback_media)?;
    let _ = fs::remove_file(data_dir.join("meliponario.db-wal"));
    let _ = fs::remove_file(data_dir.join("meliponario.db-shm"));
    let _ = fs::remove_file(pending_manifest);
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
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    let created_at = timestamp(&pool).await?;
    let suffix = Uuid::new_v4().to_string();
    let target_dir = data_dir
        .join("backups")
        .join(format!("backup-{created_at}-{}", &suffix[..8]));
    create_backup_at(&pool, &data_dir, &target_dir, &created_at).await?;
    Ok(GeneratedArtifact {
        kind: "backup".to_owned(),
        path: target_dir.to_string_lossy().into_owned(),
        created_at,
    })
}

#[tauri::command]
pub async fn stage_restore(
    app: AppHandle,
    backup_path: String,
) -> Result<RestoreStageResult, String> {
    let trimmed = backup_path.trim();
    if trimmed.is_empty() {
        return Err("Selecione um backup completo ou um banco legado compatível.".to_owned());
    }
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    stage_restore_at(&data_dir, Path::new(trimmed)).await
}

fn blob_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

async fn export_rows(pool: &SqlitePool, sql: &'static str) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(sql).fetch_all(pool).await.map_err(|_| {
        "Não foi possível consultar uma estrutura para a exportação JSON.".to_owned()
    })?;
    let mut output = Vec::with_capacity(rows.len());
    for row in rows {
        let mut object = Map::new();
        for (index, column) in row.columns().iter().enumerate() {
            let value = match column.type_info().name() {
                "INTEGER" => row
                    .try_get::<Option<i64>, _>(index)
                    .map(|value| value.map_or(Value::Null, Value::from)),
                "REAL" => row
                    .try_get::<Option<f64>, _>(index)
                    .map(|value| value.map_or(Value::Null, Value::from)),
                "BLOB" => row.try_get::<Option<Vec<u8>>, _>(index).map(|value| {
                    value.map_or(Value::Null, |bytes| Value::String(blob_hex(&bytes)))
                }),
                _ => row
                    .try_get::<Option<String>, _>(index)
                    .map(|value| value.map_or(Value::Null, Value::String)),
            }
            .map_err(|_| {
                "Não foi possível serializar uma estrutura para a exportação JSON.".to_owned()
            })?;
            object.insert(column.name().to_owned(), value);
        }
        output.push(Value::Object(object));
    }
    Ok(output)
}

async fn portable_tables(pool: &SqlitePool) -> Result<PortableTables, String> {
    Ok(PortableTables {
        meliponaries: export_rows(pool, "SELECT * FROM meliponaries ORDER BY created_at, id")
            .await?,
        species: export_rows(pool, "SELECT * FROM species ORDER BY created_at, id").await?,
        boxes: export_rows(pool, "SELECT * FROM boxes ORDER BY created_at, id").await?,
        box_state_records: export_rows(
            pool,
            "SELECT * FROM box_state_records ORDER BY occurred_at, created_at, id",
        )
        .await?,
        colonies: export_rows(pool, "SELECT * FROM colonies ORDER BY created_at, id").await?,
        colony_box_occupancies: export_rows(
            pool,
            "SELECT * FROM colony_box_occupancies ORDER BY started_at, created_at, id",
        )
        .await?,
        inspections: export_rows(
            pool,
            "SELECT * FROM inspections ORDER BY inspected_at, created_at, id",
        )
        .await?,
        inspection_photos: export_rows(
            pool,
            "SELECT * FROM inspection_photos ORDER BY created_at, id",
        )
        .await?,
        feedings: export_rows(
            pool,
            "SELECT * FROM feedings ORDER BY fed_at, created_at, id",
        )
        .await?,
        production_records: export_rows(
            pool,
            "SELECT * FROM production_records ORDER BY harvested_at, created_at, id",
        )
        .await?,
        box_maintenance_records: export_rows(
            pool,
            "SELECT * FROM box_maintenance_records ORDER BY maintained_at, created_at, id",
        )
        .await?,
        colony_events: export_rows(
            pool,
            "SELECT * FROM colony_events ORDER BY occurred_at, created_at, id",
        )
        .await?,
        colony_divisions: export_rows(
            pool,
            "SELECT * FROM colony_divisions ORDER BY performed_at, created_at, id",
        )
        .await?,
        colony_movements: export_rows(
            pool,
            "SELECT * FROM colony_movements ORDER BY moved_at, created_at, id",
        )
        .await?,
        transport_returns: export_rows(
            pool,
            "SELECT * FROM transport_returns ORDER BY returned_at, created_at, id",
        )
        .await?,
        movement_documents: export_rows(
            pool,
            "SELECT * FROM movement_documents ORDER BY created_at, id",
        )
        .await?,
        colony_lifecycle_records: export_rows(
            pool,
            "SELECT * FROM colony_lifecycle_records ORDER BY occurred_at, created_at, id",
        )
        .await?,
        scheduled_tasks: export_rows(
            pool,
            "SELECT * FROM scheduled_tasks ORDER BY created_at, id",
        )
        .await?,
        audit_records: export_rows(
            pool,
            "SELECT * FROM audit_records ORDER BY changed_at, created_at, id",
        )
        .await?,
        managed_attachments: export_rows(
            pool,
            "SELECT * FROM managed_attachments ORDER BY created_at, id",
        )
        .await?,
    })
}

#[tauri::command]
pub async fn export_portable_json(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<GeneratedArtifact, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    let created_at = timestamp(&pool).await?;
    let export_dir = data_dir.join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|_| "Não foi possível preparar a pasta de exportações.".to_owned())?;
    let schema_version = database_schema_version(&pool).await?;
    let export = PortableExport {
        format: PORTABLE_FORMAT,
        format_version: PORTABLE_FORMAT_VERSION,
        generated_at: created_at.clone(),
        app_version: env!("CARGO_PKG_VERSION"),
        schema_version,
        assets_embedded: false,
        tables: portable_tables(&pool).await?,
    };
    let bytes = serde_json::to_vec_pretty(&export)
        .map_err(|_| "Não foi possível serializar a exportação JSON.".to_owned())?;
    let path = export_dir.join(format!("estrutura-{created_at}.json"));
    fs::write(&path, bytes).map_err(|_| "Não foi possível gravar a exportação JSON.".to_owned())?;
    Ok(GeneratedArtifact {
        kind: "json".to_owned(),
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
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    let created_at = timestamp(&pool).await?;
    let export_dir = data_dir.join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|_| "Não foi possível preparar a pasta de relatórios.".to_owned())?;
    let summary = repository::core_summary(&pool)
        .await
        .map_err(|_| "Não foi possível consultar o resumo do plantel.".to_owned())?;
    let overview = dashboard::overview(&pool)
        .await
        .map_err(|_| "Não foi possível consultar a visão operacional.".to_owned())?;

    let mut report = format!(
        "# Relatório do MeliponarioManager\
\
Gerado em: {created_at}\
\
## Estrutura\
\
- Meliponários: {}\
- Espécies: {}\
- Colônias: {}\
- Caixas: {}\
- Caixas ocupadas: {}\
- Caixas ativas e livres: {}\
\
## Situação das colônias\
",
        summary.meliponaries,
        summary.species,
        summary.colonies,
        summary.boxes,
        overview.occupied_boxes,
        overview.free_boxes
    );
    for item in &overview.colony_statuses {
        report.push_str(&format!(
            "- {}: {}\
",
            item.label, item.count
        ));
    }
    report.push_str(
        "\
## Distribuição por espécie\
",
    );
    for item in &overview.species_distribution {
        report.push_str(&format!(
            "- {}: {}\
",
            item.label, item.count
        ));
    }
    report.push_str(&format!(
        "\
## Pendências\
\
Alertas atuais: {}\
",
        overview.alerts.len()
    ));
    for alert in overview.alerts.iter().take(20) {
        let context = if let Some(colony_code) = alert.colony_code.as_deref() {
            format!("Colônia {colony_code}")
        } else if let Some(box_code) = alert.box_code.as_deref() {
            format!("Caixa {box_code}")
        } else {
            "Meliponário".to_owned()
        };
        report.push_str(&format!(
            "- {context}: {}{}\
",
            alert.title,
            alert
                .due_at
                .as_ref()
                .map(|value| format!(" ({value})"))
                .unwrap_or_default()
        ));
    }

    let path = export_dir.join(format!("relatorio-{created_at}.md"));
    fs::write(&path, report).map_err(|_| "Não foi possível gravar o relatório.".to_owned())?;
    Ok(GeneratedArtifact {
        kind: "report".to_owned(),
        path: path.to_string_lossy().into_owned(),
        created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("meliponariomanager-{label}-{}", Uuid::new_v4()))
    }

    async fn seeded_installation(root: &Path) -> SqlitePool {
        fs::create_dir_all(root).unwrap();
        let pool = crate::database::initialize(&root.join(DATABASE_FILE))
            .await
            .unwrap();
        sqlx::query("INSERT INTO meliponaries(id,name) VALUES('m1','Principal')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO species(id,common_name) VALUES('s1','Jataí')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO boxes(id,meliponary_id,code) VALUES('b1','m1','CX-001')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO colonies(id,meliponary_id,species_id,code,installed_at) VALUES('c1','m1','s1','JAT-001','2026-01-01 09:00:00')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at) VALUES('o1','c1','b1','2026-01-01 09:00:00')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO inspections(id,colony_id,box_id,inspected_at,strength,observations) VALUES('i1','c1','b1','2026-01-02 09:00:00','medium','Normal')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feedings(id,colony_id,box_id,fed_at,food_type,quantity,unit) VALUES('f1','c1','b1','2026-01-03 09:00:00','Xarope',40,'ml')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO production_records(id,colony_id,box_id,harvested_at,product_type,quantity,unit) VALUES('p1','c1','b1','2026-01-04 09:00:00','honey',0.5,'kg')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO box_maintenance_records(id,box_id,colony_id,maintained_at,maintenance_type,description,cost) VALUES('bm1','b1','c1','2026-01-05 09:00:00','cleaning','Limpeza',12.5)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,colony_id,box_id,task_type,title,scheduled_for,status,reschedule_reason) VALUES('t1','m1','c1','b1','inspection','Inspecionar','2026-01-06 09:00:00','rescheduled','Chuva')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,colony_id,box_id,task_type,title,scheduled_for,status,rescheduled_from_id,completed_at,completed_by_type,completed_by_id) VALUES('t2','m1','c1','b1','inspection','Inspecionar','2026-01-07 09:00:00','completed','t1','2026-01-07 08:30:00','inspection','i1')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO colony_movements(id,colony_id,movement_type,moved_at,from_meliponary_id,from_box_id,destination) VALUES('mv1','c1','transport','2026-01-08 09:00:00','m1','b1','Exposição')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO transport_returns(id,movement_id,returned_at,notes) VALUES('tr1','mv1','2026-01-09 09:00:00','Retorno')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO colony_lifecycle_records(id,colony_id,box_id,action,occurred_at,previous_status,new_status,reason) VALUES('lc1','c1','b1','deactivate','2026-01-10 09:00:00','active','inactive','Teste histórico')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO audit_records(id,entity_type,entity_id,action,changed_at,reason,before_json,after_json) VALUES('au1','inspection','i1','correct','2026-01-11 09:00:00','Ajuste','{}','{}')")
            .execute(&pool)
            .await
            .unwrap();

        let photo_rel = "media/inspections/i1/photo.jpg";
        let attachment_rel = "media/attachments/meliponaries/m1/anexo.pdf";
        fs::create_dir_all(root.join("media/inspections/i1")).unwrap();
        fs::create_dir_all(root.join("media/attachments/meliponaries/m1")).unwrap();
        fs::write(root.join(photo_rel), b"photo-bytes").unwrap();
        fs::write(root.join(attachment_rel), b"attachment-bytes").unwrap();
        sqlx::query("INSERT INTO inspection_photos(id,inspection_id,relative_path,original_name,mime_type,byte_size) VALUES('ph1','i1',?,'foto.jpg','image/jpeg',11)")
            .bind(photo_rel)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO managed_attachments(id,meliponary_id,original_name,relative_path,extension,mime_type,byte_size,description) VALUES('a1','m1','licenca.pdf',?,'pdf','application/pdf',16,'Documento de teste')")
            .bind(attachment_rel)
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn backup_manifest_restore_recovers_database_and_assets() {
        let source = temp_root("backup-source");
        let pool = seeded_installation(&source).await;
        let backup = source.join("backup-test");
        let manifest = create_backup_at(&pool, &source, &backup, "20260829-170000")
            .await
            .unwrap();
        assert_eq!(manifest.format, BACKUP_FORMAT);
        assert_eq!(manifest.format_version, 1);
        assert_eq!(manifest.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(manifest.assets.len(), 2);
        assert!(manifest.assets.iter().all(|asset| asset.sha256.len() == 64));
        validate_restore_source(&backup).await.unwrap();
        pool.close().await;

        let destination = temp_root("backup-destination");
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join(DATABASE_FILE), b"previous-state").unwrap();
        stage_restore_at(&destination, &backup).await.unwrap();
        apply_pending_restore(&destination).unwrap();

        let restored = crate::database::initialize(&destination.join(DATABASE_FILE))
            .await
            .unwrap();
        macro_rules! assert_table_count {
            ($sql:literal, $expected:expr, $table:literal) => {{
                let count: i64 = sqlx::query_scalar($sql).fetch_one(&restored).await.unwrap();
                assert_eq!(count, $expected, "table {}", $table);
            }};
        }
        assert_table_count!("SELECT COUNT(*) FROM meliponaries", 1_i64, "meliponaries");
        assert_table_count!("SELECT COUNT(*) FROM inspections", 1_i64, "inspections");
        assert_table_count!("SELECT COUNT(*) FROM feedings", 1_i64, "feedings");
        assert_table_count!(
            "SELECT COUNT(*) FROM production_records",
            1_i64,
            "production_records"
        );
        assert_table_count!(
            "SELECT COUNT(*) FROM box_maintenance_records",
            1_i64,
            "box_maintenance_records"
        );
        assert_table_count!(
            "SELECT COUNT(*) FROM scheduled_tasks",
            2_i64,
            "scheduled_tasks"
        );
        assert_table_count!(
            "SELECT COUNT(*) FROM colony_movements",
            1_i64,
            "colony_movements"
        );
        assert_table_count!(
            "SELECT COUNT(*) FROM transport_returns",
            1_i64,
            "transport_returns"
        );
        assert_table_count!(
            "SELECT COUNT(*) FROM colony_lifecycle_records",
            1_i64,
            "colony_lifecycle_records"
        );
        assert_table_count!("SELECT COUNT(*) FROM audit_records", 1_i64, "audit_records");
        assert_table_count!(
            "SELECT COUNT(*) FROM inspection_photos",
            1_i64,
            "inspection_photos"
        );
        assert_table_count!(
            "SELECT COUNT(*) FROM managed_attachments",
            1_i64,
            "managed_attachments"
        );
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&restored)
            .await
            .unwrap();
        assert_eq!(integrity, "ok");
        assert!(destination.join("media/inspections/i1/photo.jpg").is_file());
        assert!(destination
            .join("media/attachments/meliponaries/m1/anexo.pdf")
            .is_file());
        restored.close().await;
        let _ = fs::remove_dir_all(source);
        let _ = fs::remove_dir_all(destination);
    }

    #[tokio::test]
    async fn restore_rejects_corrupt_incompatible_and_incomplete_backups_without_touching_current()
    {
        let root = temp_root("restore-invalid");
        fs::create_dir_all(&root).unwrap();
        let current = root.join(DATABASE_FILE);
        fs::write(&current, b"keep-current").unwrap();

        let corrupt = root.join("corrupt.db");
        fs::write(&corrupt, b"not sqlite").unwrap();
        assert!(stage_restore_at(&root, &corrupt).await.is_err());
        assert_eq!(fs::read(&current).unwrap(), b"keep-current");

        let source = temp_root("restore-source");
        let pool = seeded_installation(&source).await;
        let backup = source.join("backup-test");
        create_backup_at(&pool, &source, &backup, "20260829-170000")
            .await
            .unwrap();
        pool.close().await;

        let manifest_path = backup.join(BACKUP_MANIFEST);
        let original_manifest = fs::read(&manifest_path).unwrap();
        let mut manifest: BackupManifest = serde_json::from_slice(&original_manifest).unwrap();
        manifest.format_version = BACKUP_FORMAT_VERSION + 1;
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(stage_restore_at(&root, &backup).await.is_err());
        assert_eq!(fs::read(&current).unwrap(), b"keep-current");

        fs::write(&manifest_path, &original_manifest).unwrap();
        let manifest: BackupManifest = serde_json::from_slice(&original_manifest).unwrap();
        let changed = &manifest.assets[0].relative_path;
        let original_asset = fs::read(backup.join(changed)).unwrap();
        let mut changed_asset = original_asset.clone();
        changed_asset[0] ^= 0xff;
        fs::write(backup.join(changed), &changed_asset).unwrap();
        assert_eq!(changed_asset.len(), original_asset.len());
        assert!(stage_restore_at(&root, &backup).await.is_err());
        assert_eq!(fs::read(&current).unwrap(), b"keep-current");
        fs::write(backup.join(changed), original_asset).unwrap();

        let missing = &manifest.assets[0].relative_path;
        fs::remove_file(backup.join(missing)).unwrap();
        assert!(stage_restore_at(&root, &backup).await.is_err());
        assert_eq!(fs::read(&current).unwrap(), b"keep-current");
        assert!(!root.join("restore.pending.db").exists());

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(source);
    }

    #[tokio::test]
    async fn portable_json_is_versioned_structural_and_does_not_embed_assets() {
        let root = temp_root("portable-json");
        let pool = seeded_installation(&root).await;
        let tables = portable_tables(&pool).await.unwrap();
        assert_eq!(tables.inspections.len(), 1);
        assert_eq!(tables.scheduled_tasks.len(), 2);
        assert_eq!(tables.transport_returns.len(), 1);
        assert_eq!(tables.inspection_photos.len(), 1);
        assert_eq!(tables.managed_attachments.len(), 1);
        let export = PortableExport {
            format: PORTABLE_FORMAT,
            format_version: PORTABLE_FORMAT_VERSION,
            generated_at: "20260829-170000".to_owned(),
            app_version: env!("CARGO_PKG_VERSION"),
            schema_version: database_schema_version(&pool).await.unwrap(),
            assets_embedded: false,
            tables,
        };
        let json = serde_json::to_value(export).unwrap();
        assert_eq!(json["format"], PORTABLE_FORMAT);
        assert_eq!(json["formatVersion"], 1);
        assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION);
        assert_eq!(json["assetsEmbedded"], false);
        assert_eq!(
            json["tables"]["managedAttachments"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        pool.close().await;
        let _ = fs::remove_dir_all(root);
    }
}
