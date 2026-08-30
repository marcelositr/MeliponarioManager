use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn apply(pool: &SqlitePool, sql: &'static str) {
    sqlx::raw_sql(sql).execute(pool).await.unwrap();
}

#[tokio::test]
async fn stage3_database_backfills_latest_valid_agenda_without_duplicates() {
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
    ] {
        apply(&pool, migration).await;
    }

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
    sqlx::query(
        "INSERT INTO colonies(id,meliponary_id,species_id,code,installed_at)
         VALUES('c1','m1','s1','JAT-001','2026-01-01 09:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at)
         VALUES('o1','c1','b1','2026-01-01 09:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO inspections(id,colony_id,box_id,inspected_at,strength,next_inspection_at,voided_at,void_reason)
         VALUES
           ('i-old','c1','b1','2026-01-10 10:00:00','weak','2026-01-17 10:00:00','2026-01-11 10:00:00','Duplicada'),
           ('i-mid','c1','b1','2026-02-10 10:00:00','medium','2026-02-17 10:00:00',NULL,NULL),
           ('i-new','c1','b1','2026-03-10 10:00:00','strong','2026-03-17 10:00:00',NULL,NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO feedings(id,colony_id,box_id,fed_at,food_type,next_feeding_at)
         VALUES
           ('f-old','c1','b1','2026-02-01 10:00:00','Xarope','2026-02-08 10:00:00'),
           ('f-new','c1','b1','2026-03-01 10:00:00','Xarope','2026-03-08 10:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO box_maintenance_records(id,box_id,colony_id,maintained_at,maintenance_type,next_maintenance_at)
         VALUES
           ('bm-old','b1','c1','2026-01-05 10:00:00','repair','2026-02-05 10:00:00'),
           ('bm-new','b1','c1','2026-03-05 10:00:00','inspection','2026-04-05 10:00:00')",
    )
    .execute(&pool)
    .await
    .unwrap();

    apply(
        &pool,
        include_str!("../../migrations/0015_operational_agenda.sql"),
    )
    .await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_tasks")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 3);

    let inspection_source: String = sqlx::query_scalar(
        "SELECT source_id FROM scheduled_tasks WHERE task_type='inspection' AND status='pending'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(inspection_source, "i-new");
    let feeding_source: String = sqlx::query_scalar(
        "SELECT source_id FROM scheduled_tasks WHERE task_type='feeding' AND status='pending'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(feeding_source, "f-new");
    let maintenance_source: String = sqlx::query_scalar(
        "SELECT source_id FROM scheduled_tasks WHERE task_type='maintenance' AND status='pending'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(maintenance_source, "bm-new");

    let preserved_inspections: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM inspections")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(preserved_inspections, 3);
    let duplicate_sources: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM (
            SELECT source_type,source_id,COUNT(*) amount
            FROM scheduled_tasks WHERE status='pending'
            GROUP BY source_type,source_id HAVING amount > 1
         )",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(duplicate_sources, 0);
}
