use super::*;

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("meliponariomanager-{label}-{}", Uuid::new_v4()))
}

#[test]
fn sha256_file_returns_known_lowercase_hex_digest() {
    let root = temp_root("sha256-file");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("payload.bin");
    fs::write(&path, b"abc").unwrap();

    assert_eq!(
        sha256_file(&path).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    let _ = fs::remove_dir_all(root);
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
    sqlx::query("INSERT INTO inspection_photos(id,inspection_id,relative_path,original_name,mime_type,byte_size,captured_at) VALUES('ph1','i1',?,'foto.jpg','image/jpeg',11,'2026-01-02 09:00:00')")
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
