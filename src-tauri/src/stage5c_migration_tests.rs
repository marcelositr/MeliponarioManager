use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn apply(pool: &SqlitePool, sql: &'static str) {
    sqlx::raw_sql(sql).execute(pool).await.unwrap();
}

#[tokio::test]
async fn upgrade_through_0017_preserves_existing_data_and_enables_managed_attachments() {
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
        include_str!("../../migrations/0013_box_state_integrity.sql"),
        include_str!("../../migrations/0014_audit_and_record_corrections.sql"),
        include_str!("../../migrations/0015_operational_agenda.sql"),
        include_str!("../../migrations/0016_transport_returns.sql"),
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
        "INSERT INTO scheduled_tasks (
            id, meliponary_id, colony_id, task_type, title, scheduled_for, status
         ) VALUES ('t1','m1','c1','inspection','Revisar colônia','2026-02-01 09:00:00','pending')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_movements (
            id, colony_id, movement_type, moved_at, from_meliponary_id, from_box_id, destination
         ) VALUES ('mv1','c1','transport','2026-02-02 10:00:00','m1','b1','Exposição')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO transport_returns (id, movement_id, returned_at, notes)
         VALUES ('tr1','mv1','2026-02-03 10:00:00','Retorno normal')",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply(
        &pool,
        include_str!("../../migrations/0017_managed_attachments.sql"),
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
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM inspection_photos")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM scheduled_tasks")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transport_returns")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );

    sqlx::query(
        "INSERT INTO managed_attachments (
            id, meliponary_id, original_name, relative_path, extension, mime_type, byte_size
         ) VALUES (
            'a1','m1','licenca.pdf','media/attachments/meliponaries/m1/a1.pdf',
            'pdf','application/pdf',456
         )",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM managed_attachments")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
}
