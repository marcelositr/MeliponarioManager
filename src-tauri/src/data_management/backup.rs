use super::*;

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
