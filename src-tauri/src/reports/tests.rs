use super::*;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn seed() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();

    sqlx::query("INSERT INTO meliponaries(id,name) VALUES('m1','Principal'),('m2','Destino')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO species(id,common_name) VALUES('s1','Jataí')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO boxes(id,meliponary_id,code,status) VALUES('b1','m1','CX-001','active')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO colonies(id,meliponary_id,species_id,code,status) VALUES('c1','m1','s1','JAT-001','active')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at) VALUES('o1','c1','b1','2026-01-01 09:00:00')")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

fn august_filter() -> ReportFilter {
    ReportFilter {
        start_date: "2026-08-01".to_owned(),
        end_date: "2026-08-31".to_owned(),
        meliponary_id: Some("m1".to_owned()),
    }
}

#[tokio::test]
async fn period_is_inclusive_and_effective_production_never_mixes_units() {
    let pool = seed().await;
    sqlx::query(
        "INSERT INTO production_records(id,colony_id,harvested_at,product_type,quantity,unit,notes)
         VALUES
          ('p-start','c1','2026-08-01 00:00:00','honey',1.0,'kg','início'),
          ('p-end','c1','2026-08-31 23:59:59','honey',2.0,'kg','fim'),
          ('p-liters','c1','2026-08-15 12:00:00','honey',3.0,'L','volume'),
          ('p-out','c1','2026-09-01 00:00:00','honey',50.0,'kg','fora'),
          ('p-void','c1','2026-08-20 12:00:00','honey',99.0,'kg','anulado')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE production_records SET voided_at='2026-08-21 10:00:00', void_reason='erro' WHERE id='p-void'")
        .execute(&pool)
        .await
        .unwrap();

    let report = production::production_report(
        &pool,
        &ProductionReportInput {
            filter: august_filter(),
            species_id: None,
            colony_id: None,
            product_type: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(report.rows.len(), 3);
    assert_eq!(report.rows.first().unwrap().id, "p-start");
    assert_eq!(report.rows.last().unwrap().id, "p-end");
    assert_eq!(report.by_product_unit.len(), 2);
    assert!(report
        .by_product_unit
        .iter()
        .any(|item| item.unit == "kg" && (item.quantity - 3.0).abs() < f64::EPSILON));
    assert!(report
        .by_product_unit
        .iter()
        .any(|item| item.unit == "L" && (item.quantity - 3.0).abs() < f64::EPSILON));
}

#[tokio::test]
async fn operational_report_respects_reversals_transport_returns_and_real_costs() {
    let pool = seed().await;
    sqlx::query(
        "INSERT INTO colonies(id,meliponary_id,species_id,code,status) VALUES
         ('c-weak','m1','s1','JAT-W','weak'),
         ('c-recovering','m1','s1','JAT-R','recovering')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO colony_movements(id,colony_id,movement_type,moved_at,from_meliponary_id,to_meliponary_id)
         VALUES
          ('transfer-valid','c1','internal_transfer','2026-08-05 10:00:00','m1','m2'),
          ('transfer-reversed','c1','internal_transfer','2026-08-06 10:00:00','m1','m2')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE colony_movements SET reversed_at='2026-08-07 09:00:00', reversal_reason='correção' WHERE id='transfer-reversed'")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO colony_movements(id,colony_id,movement_type,moved_at,from_meliponary_id,destination)
         VALUES('transport-a','c1','transport','2026-08-10 08:00:00','m1','Feira A')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO transport_returns(id,movement_id,returned_at,notes) VALUES('ret-a','transport-a','2026-08-11 18:00:00','retorno')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO colony_movements(id,colony_id,movement_type,moved_at,from_meliponary_id,destination)
         VALUES('transport-b','c1','transport','2026-08-20 08:00:00','m1','Feira B')",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO box_maintenance_records(id,box_id,colony_id,maintained_at,maintenance_type,cost)
         VALUES('maint-ok','b1','c1','2026-08-12 09:00:00','repair',125.5),
               ('maint-void','b1','c1','2026-08-13 09:00:00','repair',900.0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE box_maintenance_records SET voided_at='2026-08-14 09:00:00', void_reason='erro' WHERE id='maint-void'")
        .execute(&pool)
        .await
        .unwrap();

    let report = operational::operational_report(&pool, &august_filter())
        .await
        .unwrap();
    assert_eq!(report.plantel.total_colonies, 3);
    assert_eq!(report.plantel.active_colonies, 3);
    assert!(report
        .plantel
        .colony_statuses
        .iter()
        .any(|row| row.key == "active" && row.count == 3));
    assert!(!report
        .plantel
        .colony_statuses
        .iter()
        .any(|row| matches!(row.key.as_str(), "weak" | "recovering")));
    assert_eq!(report.movements.transfers, 1);
    assert_eq!(report.movements.temporary_started, 2);
    assert_eq!(report.movements.returns_completed, 1);
    assert_eq!(report.movements.temporary_open_at_end, 1);
    assert_eq!(report.management.maintenance, 1);

    let costs = operational::cost_report(&pool, &august_filter())
        .await
        .unwrap();
    assert_eq!(costs.rows.len(), 1);
    assert!((costs.total - 125.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn agenda_counts_reschedule_and_successor_once_each() {
    let pool = seed().await;
    sqlx::query(
        "INSERT INTO scheduled_tasks(
            id,meliponary_id,colony_id,task_type,title,scheduled_for,status,reschedule_reason,created_at
         ) VALUES(
            'task-original','m1','c1','inspection','Inspeção original','2026-08-10 10:00:00','rescheduled','chuva','2026-08-01 08:00:00'
         )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO scheduled_tasks(
            id,meliponary_id,colony_id,task_type,title,scheduled_for,status,completed_at,rescheduled_from_id,created_at
         ) VALUES(
            'task-successor','m1','c1','inspection','Inspeção reagendada','2026-08-11 10:00:00','completed','2026-08-11 09:30:00','task-original','2026-08-10 08:00:00'
         ),(
            'task-late','m1','c1','feeding','Alimentação','2026-08-12 10:00:00','completed','2026-08-12 12:00:00',NULL,'2026-08-02 08:00:00'
         )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let report = operational::agenda_report(&pool, &august_filter())
        .await
        .unwrap();
    assert_eq!(report.metrics.scheduled, 3);
    assert_eq!(report.metrics.rescheduled, 1);
    assert_eq!(report.metrics.completed, 2);
    assert_eq!(report.metrics.completed_on_time, 1);
    assert_eq!(report.metrics.completed_late, 1);
    assert_eq!(report.rows.len(), 3);
}

#[tokio::test]
async fn colony_effective_history_is_chronological_and_full_mode_keeps_invalidated_facts() {
    let pool = seed().await;
    sqlx::query(
        "INSERT INTO colony_events(id,colony_id,event_type,occurred_at,title,severity)
         VALUES('event-1','c1','observation','2026-08-03 10:00:00','Primeiro fato','info')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO production_records(id,colony_id,harvested_at,product_type,quantity,unit)
         VALUES('prod-valid','c1','2026-08-04 10:00:00','honey',1.0,'kg'),
               ('prod-void','c1','2026-08-05 10:00:00','honey',2.0,'kg')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE production_records SET voided_at='2026-08-06 10:00:00', void_reason='duplicado' WHERE id='prod-void'")
        .execute(&pool)
        .await
        .unwrap();

    let effective = colony::colony_report(
        &pool,
        &ColonyReportInput {
            filter: august_filter(),
            colony_id: "c1".to_owned(),
            include_audit: false,
        },
    )
    .await
    .unwrap();
    assert!(effective
        .timeline
        .windows(2)
        .all(|pair| pair[0].occurred_at <= pair[1].occurred_at));
    assert!(!effective
        .timeline
        .iter()
        .any(|row| row.source_id == "prod-void"));

    let full = colony::colony_report(
        &pool,
        &ColonyReportInput {
            filter: august_filter(),
            colony_id: "c1".to_owned(),
            include_audit: true,
        },
    )
    .await
    .unwrap();
    assert!(full
        .timeline
        .iter()
        .any(|row| row.source_id == "prod-void" && row.state == "voided"));
}
