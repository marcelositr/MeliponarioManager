use super::*;

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
