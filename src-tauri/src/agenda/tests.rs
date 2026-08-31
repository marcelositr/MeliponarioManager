use super::*;
use crate::{
    domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
    feeding::{self, CreateFeeding},
    inspections::{self, CreateInspection},
    maintenance::{self, CreateBoxMaintenance},
    repository,
};
use sqlx::sqlite::SqlitePoolOptions;

async fn seeded() -> (SqlitePool, String, String, String) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    let mel = repository::create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Principal".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let species = repository::create_species(
        &pool,
        CreateSpecies {
            common_name: "Jataí".into(),
            scientific_name: None,
            genus: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let box_record = repository::create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: mel.id.clone(),
            code: "CX-001".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let colony = repository::create_colony(
        &pool,
        CreateColony {
            meliponary_id: mel.id.clone(),
            species_id: species.id,
            code: "JAT-001".into(),
            origin_type: None,
            origin_notes: None,
            installed_at: Some("2026-01-01 09:00:00".into()),
            mother_colony_id: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    repository::place_colony(
        &pool,
        PlaceColony {
            colony_id: colony.id.clone(),
            box_id: box_record.id.clone(),
            started_at: Some("2026-01-01 09:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    (pool, mel.id, colony.id, box_record.id)
}

async fn future(pool: &SqlitePool, modifier: &str) -> String {
    sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S','now','localtime',?)")
        .bind(modifier)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn pending_for(
    pool: &SqlitePool,
    colony_id: Option<String>,
    box_id: Option<String>,
    task_type: &str,
) -> Vec<ScheduledTask> {
    list(
        pool,
        TaskQuery {
            view: Some("pending".into()),
            colony_id,
            box_id,
            task_type: Some(task_type.into()),
            ..Default::default()
        },
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn manual_task_supports_views_and_generic_completion() {
    let (pool, meliponary_id, _, _) = seeded().await;
    let now = time::local_now(&pool).await.unwrap();
    let task = create_manual(
        &pool,
        CreateTask {
            meliponary_id,
            colony_id: None,
            box_id: None,
            task_type: "generic".into(),
            title: "Conferir autorização".into(),
            description: None,
            scheduled_for: now,
            priority: Some("attention".into()),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        list(
            &pool,
            TaskQuery {
                view: Some("pending".into()),
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .len(),
        1
    );
    complete_generic(&pool, &task.id).await.unwrap();
    assert_eq!(
        list(
            &pool,
            TaskQuery {
                view: Some("completed".into()),
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .len(),
        1
    );
}

#[tokio::test]
async fn reschedule_preserves_lineage_and_original() {
    let (pool, meliponary_id, _, _) = seeded().await;
    let now = time::local_now(&pool).await.unwrap();
    let task = create_manual(
        &pool,
        CreateTask {
            meliponary_id,
            colony_id: None,
            box_id: None,
            task_type: "generic".into(),
            title: "Limpar área".into(),
            description: None,
            scheduled_for: now.clone(),
            priority: None,
        },
    )
    .await
    .unwrap();
    let next = reschedule(
        &pool,
        RescheduleTask {
            id: task.id.clone(),
            scheduled_for: now,
            reason: Some("Chuva".into()),
        },
    )
    .await
    .unwrap();
    assert_eq!(get(&pool, &task.id).await.unwrap().status, "rescheduled");
    assert_eq!(next.status, "pending");
    assert_eq!(next.rescheduled_from_id.as_deref(), Some(task.id.as_str()));
}

#[tokio::test]
async fn cancellation_and_skip_require_reason() {
    let (pool, meliponary_id, _, _) = seeded().await;
    let now = time::local_now(&pool).await.unwrap();
    let one = create_manual(
        &pool,
        CreateTask {
            meliponary_id: meliponary_id.clone(),
            colony_id: None,
            box_id: None,
            task_type: "generic".into(),
            title: "Uma".into(),
            description: None,
            scheduled_for: now.clone(),
            priority: None,
        },
    )
    .await
    .unwrap();
    assert!(cancel(
        &pool,
        TaskReason {
            id: one.id.clone(),
            reason: " ".into()
        }
    )
    .await
    .is_err());
    assert_eq!(
        cancel(
            &pool,
            TaskReason {
                id: one.id,
                reason: "Sem necessidade".into()
            }
        )
        .await
        .unwrap()
        .status,
        "cancelled"
    );
    let two = create_manual(
        &pool,
        CreateTask {
            meliponary_id,
            colony_id: None,
            box_id: None,
            task_type: "generic".into(),
            title: "Duas".into(),
            description: None,
            scheduled_for: now,
            priority: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        skip(
            &pool,
            TaskReason {
                id: two.id,
                reason: "Decisão consciente".into()
            }
        )
        .await
        .unwrap()
        .status,
        "skipped"
    );
}

#[tokio::test]
async fn inspection_next_date_reconciles_without_duplicate() {
    let (pool, _, colony_id, _) = seeded().await;
    let first = inspections::create(
        &pool,
        CreateInspection {
            colony_id: colony_id.clone(),
            inspected_at: Some("2026-01-10 10:00:00".into()),
            strength: Some("medium".into()),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: Some("2026-01-17 10:00:00".into()),
        },
    )
    .await
    .unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    let pending = pending_for(&pool, Some(colony_id.clone()), None, "inspection").await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].source_id.as_deref(), Some(first.id.as_str()));
    inspections::create(
        &pool,
        CreateInspection {
            colony_id: colony_id.clone(),
            inspected_at: Some("2026-01-20 10:00:00".into()),
            strength: Some("strong".into()),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: Some("2026-01-27 10:00:00".into()),
        },
    )
    .await
    .unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    let pending = pending_for(&pool, Some(colony_id), None, "inspection").await;
    assert_eq!(pending.len(), 1);
    let cancelled = list(
        &pool,
        TaskQuery {
            view: Some("cancelled".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(cancelled.len(), 1);
}

#[tokio::test]
async fn manual_reschedule_of_derived_inspection_survives_reconcile_and_source_changes_win() {
    let (pool, _, colony_id, _) = seeded().await;
    let source_date = future(&pool, "+2 days").await;
    let manual_date = future(&pool, "+4 days").await;
    let corrected_date = future(&pool, "+6 days").await;
    let source = inspections::create(
        &pool,
        CreateInspection {
            colony_id: colony_id.clone(),
            inspected_at: Some(time::local_now(&pool).await.unwrap()),
            strength: Some("medium".into()),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: Some(source_date.clone()),
        },
    )
    .await
    .unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    let original = pending_for(&pool, Some(colony_id.clone()), None, "inspection")
        .await
        .remove(0);
    let manual = reschedule(
        &pool,
        RescheduleTask {
            id: original.id,
            scheduled_for: manual_date.clone(),
            reason: Some("Ajuste operacional".into()),
        },
    )
    .await
    .unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    let pending = pending_for(&pool, Some(colony_id.clone()), None, "inspection").await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, manual.id);
    assert_eq!(pending[0].scheduled_for, manual_date);
    assert_eq!(pending[0].source_id.as_deref(), Some(source.id.as_str()));

    sqlx::query("UPDATE inspections SET next_inspection_at=? WHERE id=?")
        .bind(&corrected_date)
        .bind(&source.id)
        .execute(&pool)
        .await
        .unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    let pending = pending_for(&pool, Some(colony_id.clone()), None, "inspection").await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].scheduled_for, corrected_date);
    assert_ne!(pending[0].id, manual.id);

    sqlx::query(
        "UPDATE inspections SET voided_at=CURRENT_TIMESTAMP,void_reason='teste' WHERE id=?",
    )
    .bind(&source.id)
    .execute(&pool)
    .await
    .unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    assert!(pending_for(&pool, Some(colony_id), None, "inspection")
        .await
        .is_empty());
}

#[tokio::test]
async fn manual_reschedule_of_derived_feeding_survives_plain_reconcile() {
    let (pool, _, colony_id, _) = seeded().await;
    let source_date = future(&pool, "+2 days").await;
    let manual_date = future(&pool, "+4 days").await;
    feeding::create(
        &pool,
        CreateFeeding {
            colony_id: colony_id.clone(),
            fed_at: Some(time::local_now(&pool).await.unwrap()),
            food_type: "Xarope".into(),
            quantity: None,
            unit: None,
            response_notes: None,
            notes: None,
            next_feeding_at: Some(source_date),
        },
    )
    .await
    .unwrap();
    reconcile_feeding(&pool, &colony_id).await.unwrap();
    let original = pending_for(&pool, Some(colony_id.clone()), None, "feeding")
        .await
        .remove(0);
    let manual = reschedule(
        &pool,
        RescheduleTask {
            id: original.id,
            scheduled_for: manual_date.clone(),
            reason: Some("Ajuste operacional".into()),
        },
    )
    .await
    .unwrap();
    reconcile_feeding(&pool, &colony_id).await.unwrap();
    let pending = pending_for(&pool, Some(colony_id), None, "feeding").await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, manual.id);
    assert_eq!(pending[0].scheduled_for, manual_date);
}

#[tokio::test]
async fn manual_reschedule_of_derived_maintenance_survives_plain_reconcile() {
    let (pool, _, _, box_id) = seeded().await;
    let source_date = future(&pool, "+2 days").await;
    let manual_date = future(&pool, "+4 days").await;
    maintenance::create(
        &pool,
        CreateBoxMaintenance {
            box_id: box_id.clone(),
            maintained_at: Some(time::local_now(&pool).await.unwrap()),
            maintenance_type: "cleaning".into(),
            description: None,
            performed_by: None,
            cost: None,
            next_maintenance_at: Some(source_date),
        },
    )
    .await
    .unwrap();
    reconcile_maintenance(&pool, &box_id).await.unwrap();
    let original = pending_for(&pool, None, Some(box_id.clone()), "maintenance")
        .await
        .remove(0);
    let manual = reschedule(
        &pool,
        RescheduleTask {
            id: original.id,
            scheduled_for: manual_date.clone(),
            reason: Some("Ajuste operacional".into()),
        },
    )
    .await
    .unwrap();
    reconcile_maintenance(&pool, &box_id).await.unwrap();
    let pending = pending_for(&pool, None, Some(box_id), "maintenance").await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, manual.id);
    assert_eq!(pending[0].scheduled_for, manual_date);
}

#[tokio::test]
async fn reconcile_all_repairs_missing_derived_task_idempotently() {
    let (pool, _, colony_id, _) = seeded().await;
    let source_date = future(&pool, "+2 days").await;
    inspections::create(
        &pool,
        CreateInspection {
            colony_id: colony_id.clone(),
            inspected_at: Some(time::local_now(&pool).await.unwrap()),
            strength: Some("medium".into()),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: Some(source_date),
        },
    )
    .await
    .unwrap();
    reconcile_inspection(&pool, &colony_id).await.unwrap();
    sqlx::query("DELETE FROM scheduled_tasks WHERE task_type='inspection' AND colony_id=? AND status='pending'")
        .bind(&colony_id)
        .execute(&pool)
        .await
        .unwrap();
    let pending_before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scheduled_tasks
         WHERE task_type='inspection' AND colony_id=? AND status='pending'",
    )
    .bind(&colony_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pending_before, 0);
    reconcile_all(&pool).await.unwrap();
    reconcile_all(&pool).await.unwrap();
    let pending_after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scheduled_tasks
         WHERE task_type='inspection' AND colony_id=? AND status='pending'",
    )
    .bind(&colony_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pending_after, 1);
}

#[tokio::test]
async fn executive_summary_buckets_do_not_overlap() {
    let (pool, meliponary_id, _, _) = seeded().await;
    for (id, scheduled_for) in [
        ("overdue", "2026-09-10 11:00:00"),
        ("today", "2026-09-10 13:00:00"),
        ("next", "2026-09-11 09:00:00"),
        ("future", "2026-09-18 00:00:00"),
    ] {
        sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,task_type,title,scheduled_for) VALUES(?,?,'generic',?,?)")
            .bind(id)
            .bind(&meliponary_id)
            .bind(id)
            .bind(scheduled_for)
            .execute(&pool)
            .await
            .unwrap();
    }
    let summary = summary_at(&pool, Some(&meliponary_id), "2026-09-10 12:00:00")
        .await
        .unwrap();
    assert_eq!(summary.overdue, 1);
    assert_eq!(summary.today, 1);
    assert_eq!(summary.next_seven_days, 1);
    assert_eq!(summary.future, 1);
    assert_eq!(
        summary_at(&pool, Some("missing"), "2026-09-10 12:00:00")
            .await
            .unwrap()
            .overdue,
        0
    );
}
