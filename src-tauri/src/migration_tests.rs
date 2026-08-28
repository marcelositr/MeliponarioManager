use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn apply(pool: &SqlitePool, sql: &'static str) {
    sqlx::raw_sql(sql).execute(pool).await.unwrap();
}

async fn apply_through_0013(pool: &SqlitePool) {
    for migration in [
        include_str!("../../migrations/0001_bootstrap.sql"),
        include_str!("../../migrations/0002_core_domain.sql"),
        include_str!("../../migrations/0003_inspections.sql"),
        include_str!("../../migrations/0004_colony_events.sql"),
        include_str!("../../migrations/0005_colony_divisions.sql"),
        include_str!("../../migrations/0006_feedings.sql"),
        include_str!("../../migrations/0007_production.sql"),
        include_str!("../../migrations/0008_colony_movements.sql"),
        include_str!("../../migrations/0009_box_maintenance.sql"),
        include_str!("../../migrations/0010_colony_lifecycle.sql"),
        include_str!("../../migrations/0011_movement_documents.sql"),
        include_str!("../../migrations/0012_inspection_photos.sql"),
        include_str!("../../migrations/0013_box_state_integrity.sql"),
    ] {
        apply(pool, migration).await;
    }
}

#[tokio::test]
async fn v071_upgrade_preserves_operational_history_and_adds_box_state_integrity() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    for migration in [
        include_str!("../../migrations/0001_bootstrap.sql"),
        include_str!("../../migrations/0002_core_domain.sql"),
        include_str!("../../migrations/0003_inspections.sql"),
        include_str!("../../migrations/0004_colony_events.sql"),
        include_str!("../../migrations/0005_colony_divisions.sql"),
        include_str!("../../migrations/0006_feedings.sql"),
        include_str!("../../migrations/0007_production.sql"),
        include_str!("../../migrations/0008_colony_movements.sql"),
        include_str!("../../migrations/0009_box_maintenance.sql"),
        include_str!("../../migrations/0010_colony_lifecycle.sql"),
        include_str!("../../migrations/0011_movement_documents.sql"),
        include_str!("../../migrations/0012_inspection_photos.sql"),
    ] {
        apply(&pool, migration).await;
    }

    sqlx::query("INSERT INTO meliponaries (id, name) VALUES ('m1', 'Principal')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO species (id, common_name) VALUES ('s1', 'Jataí')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO boxes (id, meliponary_id, code) VALUES ('b1','m1','CX-001')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO colonies (id, meliponary_id, species_id, code, installed_at)
         VALUES ('c1','m1','s1','JAT-001','2026-01-01 09:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_box_occupancies (id, colony_id, box_id, started_at)
         VALUES ('o1','c1','b1','2026-01-01 09:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO inspections (id, colony_id, box_id, inspected_at, strength)
         VALUES ('i1','c1','b1','2026-01-10 10:00:00','medium')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO inspection_photos (
            id, inspection_id, relative_path, original_name, mime_type, byte_size, captured_at
         ) VALUES ('p1','i1','media/inspections/i1/p1.jpg','p1.jpg','image/jpeg',123,'2026-01-10 10:05:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_movements (
            id, colony_id, movement_type, moved_at, from_meliponary_id, from_box_id, destination
         ) VALUES ('mv1','c1','transport','2026-01-20 10:00:00','m1','b1','Feira')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO movement_documents (
            id, movement_id, document_type, reference_number, issued_at
         ) VALUES ('d1','mv1','other','REF-001','2026-01-20 08:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_events (
            id, colony_id, box_id, event_type, occurred_at, severity
         ) VALUES ('e1','c1','b1','observation','2026-01-15 10:00:00','info')",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply(
        &pool,
        include_str!("../../migrations/0013_box_state_integrity.sql"),
    )
    .await;

    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM colonies")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM colony_box_occupancies")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM inspection_photos")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM movement_documents")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM colony_events")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'box_state_records'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' AND name = 'trg_occupancy_requires_active_box'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' AND name = 'trg_nonactive_box_requires_no_active_occupancy'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn stage2_upgrade_to_0014_preserves_full_realistic_history() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    apply_through_0013(&pool).await;

    sqlx::query(
        "INSERT INTO meliponaries(id,name,responsible_name,location)
         VALUES ('m1','Principal','Marcelo','Setor A'),('m2','Apoio','Equipe','Setor B')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO species(id,common_name,scientific_name,genus)
         VALUES('s1','Jataí','Tetragonisca angustula','Tetragonisca')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO boxes(id,meliponary_id,code,model,material)
         VALUES('b1','m1','CX-001','INPA','Madeira'),('b2','m1','CX-002','INPA','Madeira')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colonies(
            id,meliponary_id,species_id,code,origin_type,installed_at,status,mother_colony_id
         ) VALUES
            ('c1','m1','s1','JAT-001','acquisition','2026-01-01 09:00:00','active',NULL),
            ('c2','m1','s1','JAT-002','multiplication','2026-02-10 10:00:00','inactive','c1')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at,ended_at,reason)
         VALUES('o1','c1','b1','2026-01-01 09:00:00','2026-03-01 09:00:00','Instalação')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO inspections(
            id,colony_id,box_id,inspected_at,strength,observations,next_inspection_at
         ) VALUES('i1','c1','b1','2026-01-10 10:00:00','medium','Normal','2026-01-20 10:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO inspection_photos(
            id,inspection_id,relative_path,original_name,mime_type,byte_size,captured_at
         ) VALUES('p1','i1','media/inspections/i1/p1.jpg','p1.jpg','image/jpeg',321,'2026-01-10 10:05:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO feedings(
            id,colony_id,box_id,fed_at,food_type,quantity,unit,next_feeding_at
         ) VALUES('f1','c1','b1','2026-01-12 10:00:00','Xarope',40,'ml','2026-01-19 10:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO production_records(
            id,colony_id,box_id,harvested_at,product_type,quantity,unit,purpose
         ) VALUES('pr1','c1','b1','2026-01-15 10:00:00','honey',80,'ml','Consumo')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO box_maintenance_records(
            id,box_id,colony_id,maintained_at,maintenance_type,description,cost
         ) VALUES('ma1','b1','c1','2026-01-18 10:00:00','repair','Tampa',15.5)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_events(
            id,colony_id,box_id,event_type,occurred_at,title,severity
         ) VALUES('e1','c1','b1','observation','2026-01-20 10:00:00','Observação','info')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_movements(
            id,colony_id,movement_type,moved_at,from_meliponary_id,from_box_id,destination,notes
         ) VALUES('mv1','c1','transport','2026-01-25 10:00:00','m1','b1','Exposição','Retorno no mesmo dia')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO movement_documents(
            id,movement_id,document_type,reference_number,source_system,issuer,issued_at,valid_until
         ) VALUES('doc1','mv1','gta','GTA-001','GEDAVE','Órgão','2026-01-25 08:00:00','2026-01-26 23:59:59')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_lifecycle_records(
            id,colony_id,action,occurred_at,previous_status,new_status,reason
         ) VALUES('life1','c2','deactivate','2026-02-20 10:00:00','active','inactive','Reserva')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_divisions(
            id,parent_colony_id,daughter_colony_id,source_box_id,performed_at,result,notes
         ) VALUES('div1','c1','c2','b1','2026-02-10 10:00:00','successful','Divisão assistida')",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply(
        &pool,
        include_str!("../../migrations/0014_audit_and_record_corrections.sql"),
    )
    .await;

    for (table, sql) in [
        ("meliponaries", "SELECT COUNT(*) FROM meliponaries"),
        ("species", "SELECT COUNT(*) FROM species"),
        ("boxes", "SELECT COUNT(*) FROM boxes"),
        ("colonies", "SELECT COUNT(*) FROM colonies"),
        (
            "colony_box_occupancies",
            "SELECT COUNT(*) FROM colony_box_occupancies",
        ),
        ("inspections", "SELECT COUNT(*) FROM inspections"),
        ("feedings", "SELECT COUNT(*) FROM feedings"),
        (
            "production_records",
            "SELECT COUNT(*) FROM production_records",
        ),
        (
            "box_maintenance_records",
            "SELECT COUNT(*) FROM box_maintenance_records",
        ),
        ("colony_events", "SELECT COUNT(*) FROM colony_events"),
        ("colony_movements", "SELECT COUNT(*) FROM colony_movements"),
        (
            "movement_documents",
            "SELECT COUNT(*) FROM movement_documents",
        ),
        (
            "colony_lifecycle_records",
            "SELECT COUNT(*) FROM colony_lifecycle_records",
        ),
        ("colony_divisions", "SELECT COUNT(*) FROM colony_divisions"),
        (
            "inspection_photos",
            "SELECT COUNT(*) FROM inspection_photos",
        ),
    ] {
        let count: i64 = sqlx::query_scalar(sql).fetch_one(&pool).await.unwrap();
        assert!(count > 0, "{table} perdeu dados durante o upgrade");
    }

    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT s.common_name
             FROM colonies c JOIN species s ON s.id=c.species_id
             WHERE c.id='c1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        "Jataí"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT i.id FROM inspection_photos p JOIN inspections i ON i.id=p.inspection_id
             WHERE p.id='p1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        "i1"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT movement_id FROM movement_documents WHERE id='doc1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        "mv1"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT daughter_colony_id FROM colony_divisions WHERE id='div1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        "c2"
    );

    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_records'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    for (table, column) in [
        ("meliponaries", "archived_at"),
        ("species", "archived_at"),
        ("inspections", "voided_at"),
        ("feedings", "voided_at"),
        ("production_records", "voided_at"),
        ("colony_movements", "reversed_at"),
        ("colony_lifecycle_records", "reversed_at"),
        ("colony_box_occupancies", "corrected_at"),
    ] {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info(?) WHERE name = ?")
                .bind(table)
                .bind(column)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1, "campo {table}.{column} não foi criado");
    }

    let old_basic_query: Vec<(String, String)> = sqlx::query_as(
        "SELECT c.code, s.common_name
         FROM colonies c JOIN species s ON s.id=c.species_id
         ORDER BY c.code",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(old_basic_query.len(), 2);
    assert_eq!(old_basic_query[0].0, "JAT-001");
}
