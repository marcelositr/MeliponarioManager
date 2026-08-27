use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn apply(pool: &SqlitePool, sql: &'static str) {
    sqlx::raw_sql(sql).execute(pool).await.unwrap();
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
