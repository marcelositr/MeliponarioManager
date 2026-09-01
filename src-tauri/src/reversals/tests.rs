use super::*;
use crate::{
    agenda,
    domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
    inspections::{self, CreateInspection},
    lifecycle::{self, ChangeColonyLifecycle},
    movements::{self, CreateMovement},
    repository,
};
use sqlx::sqlite::SqlitePoolOptions;
struct S {
    p: SqlitePool,
    c: String,
    sm: String,
    tm: String,
    sb: String,
    tb: String,
}
async fn seed() -> S {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&p).await.unwrap();
    let sm = repository::create_meliponary(
        &p,
        CreateMeliponary {
            name: "Origem".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let tm = repository::create_meliponary(
        &p,
        CreateMeliponary {
            name: "Destino".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let sp = repository::create_species(
        &p,
        CreateSpecies {
            common_name: "Jataí".into(),
            scientific_name: None,
            genus: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let sb = repository::create_box(
        &p,
        CreateHiveBox {
            meliponary_id: sm.id.clone(),
            code: "A-1".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let tb = repository::create_box(
        &p,
        CreateHiveBox {
            meliponary_id: tm.id.clone(),
            code: "B-1".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let c = repository::create_colony(
        &p,
        CreateColony {
            meliponary_id: sm.id.clone(),
            species_id: sp.id,
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
        &p,
        PlaceColony {
            colony_id: c.id.clone(),
            box_id: sb.id.clone(),
            started_at: Some("2026-01-01 09:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    S {
        p,
        c: c.id,
        sm: sm.id,
        tm: tm.id,
        sb: sb.id,
        tb: tb.id,
    }
}

async fn create_pending_inspection(s: &S) -> String {
    inspections::create(
        &s.p,
        CreateInspection {
            colony_id: s.c.clone(),
            inspected_at: Some("2026-01-10 10:00:00".into()),
            strength: Some("medium".into()),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: Some("2026-04-01 10:00:00".into()),
        },
    )
    .await
    .unwrap();
    agenda::reconcile_inspection(&s.p, &s.c).await.unwrap();
    sqlx::query_scalar(
        "SELECT id FROM scheduled_tasks
         WHERE colony_id=? AND task_type='inspection' AND status='pending'",
    )
    .bind(&s.c)
    .fetch_one(&s.p)
    .await
    .unwrap()
}

#[tokio::test]
async fn safe_lifecycle_reversal_restores_status_box_and_agenda() {
    let s = seed().await;
    let original_task_id = create_pending_inspection(&s).await;
    let r = lifecycle::change(
        &s.p,
        ChangeColonyLifecycle {
            colony_id: s.c.clone(),
            action: "loss".into(),
            occurred_at: Some("2026-02-01 10:00:00".into()),
            reason: Some("Erro".into()),
            notes: None,
        },
    )
    .await
    .unwrap();
    let cancelled: String = sqlx::query_scalar("SELECT status FROM scheduled_tasks WHERE id=?")
        .bind(&original_task_id)
        .fetch_one(&s.p)
        .await
        .unwrap();
    assert_eq!(cancelled, "cancelled");

    reverse_lifecycle(
        &s.p,
        ReverseRecord {
            id: r.id.clone(),
            reason: "Engano".into(),
        },
    )
    .await
    .unwrap();
    let st: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
        .bind(&s.c)
        .fetch_one(&s.p)
        .await
        .unwrap();
    let b: String = sqlx::query_scalar(
        "SELECT box_id FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL",
    )
    .bind(&s.c)
    .fetch_one(&s.p)
    .await
    .unwrap();
    assert_eq!(st, "active");
    assert_eq!(b, s.sb);
    let restored: (String, Option<String>) = sqlx::query_as(
        "SELECT meliponary_id,box_id FROM scheduled_tasks
         WHERE colony_id=? AND task_type='inspection' AND status='pending'",
    )
    .bind(&s.c)
    .fetch_one(&s.p)
    .await
    .unwrap();
    assert_eq!(restored.0, s.sm);
    assert_eq!(restored.1.as_deref(), Some(s.sb.as_str()));
    assert!(sqlx::query_scalar::<_, Option<String>>(
        "SELECT reversed_at FROM colony_lifecycle_records WHERE id=?"
    )
    .bind(r.id)
    .fetch_one(&s.p)
    .await
    .unwrap()
    .is_some());
}
#[tokio::test]
async fn conflicting_lifecycle_reversal_is_rejected() {
    let s = seed().await;
    let r = lifecycle::change(
        &s.p,
        ChangeColonyLifecycle {
            colony_id: s.c.clone(),
            action: "deactivate".into(),
            occurred_at: Some("2026-02-01 10:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO colony_events(id,colony_id,event_type,occurred_at,severity)VALUES('e1',?,'observation','2026-02-02 10:00:00','info')").bind(&s.c).execute(&s.p).await.unwrap();
    assert!(reverse_lifecycle(
        &s.p,
        ReverseRecord {
            id: r.id,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
}
#[tokio::test]
async fn internal_transfer_reversal_is_transactional_and_restores_agenda_context() {
    let s = seed().await;
    let task_id = create_pending_inspection(&s).await;
    let m = movements::create(
        &s.p,
        CreateMovement {
            colony_id: s.c.clone(),
            movement_type: "internal_transfer".into(),
            moved_at: Some("2026-02-01 10:00:00".into()),
            to_meliponary_id: Some(s.tm.clone()),
            to_box_id: Some(s.tb.clone()),
            destination: None,
            document_reference: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let moved_task: (String, Option<String>) =
        sqlx::query_as("SELECT meliponary_id,box_id FROM scheduled_tasks WHERE id=?")
            .bind(&task_id)
            .fetch_one(&s.p)
            .await
            .unwrap();
    assert_eq!(moved_task.0, s.tm);
    assert_eq!(moved_task.1.as_deref(), Some(s.tb.as_str()));

    reverse_movement(
        &s.p,
        ReverseRecord {
            id: m.id.clone(),
            reason: "Destino incorreto".into(),
        },
    )
    .await
    .unwrap();
    let mel: String = sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id=?")
        .bind(&s.c)
        .fetch_one(&s.p)
        .await
        .unwrap();
    let b: String = sqlx::query_scalar(
        "SELECT box_id FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL",
    )
    .bind(&s.c)
    .fetch_one(&s.p)
    .await
    .unwrap();
    assert_eq!(mel, s.sm);
    assert_eq!(b, s.sb);
    let restored_task: (String, Option<String>) =
        sqlx::query_as("SELECT meliponary_id,box_id FROM scheduled_tasks WHERE id=?")
            .bind(&task_id)
            .fetch_one(&s.p)
            .await
            .unwrap();
    assert_eq!(restored_task.0, s.sm);
    assert_eq!(restored_task.1.as_deref(), Some(s.sb.as_str()));
    let kept: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM colony_movements WHERE id=? AND reversed_at IS NOT NULL",
    )
    .bind(m.id)
    .fetch_one(&s.p)
    .await
    .unwrap();
    assert_eq!(kept, 1);
}
#[tokio::test]
async fn external_transfer_reversal_restores_initial_active_status_without_lifecycle_history() {
    let s = seed().await;
    create_pending_inspection(&s).await;
    let lifecycle_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM colony_lifecycle_records WHERE colony_id=?")
            .bind(&s.c)
            .fetch_one(&s.p)
            .await
            .unwrap();
    assert_eq!(lifecycle_before, 0);

    let movement = movements::create(
        &s.p,
        CreateMovement {
            colony_id: s.c.clone(),
            movement_type: "external_transfer".into(),
            moved_at: Some("2026-02-01 10:00:00".into()),
            to_meliponary_id: None,
            to_box_id: None,
            destination: Some("Outro criador".into()),
            document_reference: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let transferred_status: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
        .bind(&s.c)
        .fetch_one(&s.p)
        .await
        .unwrap();
    assert_eq!(transferred_status, "transferred");
    let pending_after_transfer: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scheduled_tasks
         WHERE colony_id=? AND task_type='inspection' AND status='pending'",
    )
    .bind(&s.c)
    .fetch_one(&s.p)
    .await
    .unwrap();
    assert_eq!(pending_after_transfer, 0);

    reverse_movement(
        &s.p,
        ReverseRecord {
            id: movement.id.clone(),
            reason: "Transferência lançada por engano".into(),
        },
    )
    .await
    .unwrap();

    let restored: (String, String) =
        sqlx::query_as("SELECT status,meliponary_id FROM colonies WHERE id=?")
            .bind(&s.c)
            .fetch_one(&s.p)
            .await
            .unwrap();
    assert_eq!(restored.0, "active");
    assert_eq!(restored.1, s.sm);
    let restored_box: String = sqlx::query_scalar(
        "SELECT box_id FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL",
    )
    .bind(&s.c)
    .fetch_one(&s.p)
    .await
    .unwrap();
    assert_eq!(restored_box, s.sb);
    let restored_task: (String, Option<String>) = sqlx::query_as(
        "SELECT meliponary_id,box_id FROM scheduled_tasks
         WHERE colony_id=? AND task_type='inspection' AND status='pending'",
    )
    .bind(&s.c)
    .fetch_one(&s.p)
    .await
    .unwrap();
    assert_eq!(restored_task.0, s.sm);
    assert_eq!(restored_task.1.as_deref(), Some(s.sb.as_str()));
    let lifecycle_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM colony_lifecycle_records WHERE colony_id=?")
            .bind(&s.c)
            .fetch_one(&s.p)
            .await
            .unwrap();
    assert_eq!(lifecycle_after, 0);
    let reversed_at: Option<String> =
        sqlx::query_scalar("SELECT reversed_at FROM colony_movements WHERE id=?")
            .bind(movement.id)
            .fetch_one(&s.p)
            .await
            .unwrap();
    assert!(reversed_at.is_some());
}
#[tokio::test]
async fn movement_reversal_with_later_fact_is_rejected() {
    let s = seed().await;
    let m = movements::create(
        &s.p,
        CreateMovement {
            colony_id: s.c.clone(),
            movement_type: "internal_transfer".into(),
            moved_at: Some("2026-02-01 10:00:00".into()),
            to_meliponary_id: Some(s.tm),
            to_box_id: Some(s.tb),
            destination: None,
            document_reference: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO colony_events(id,colony_id,event_type,occurred_at,severity)VALUES('e1',?,'observation','2026-02-02 10:00:00','info')").bind(&s.c).execute(&s.p).await.unwrap();
    assert!(reverse_movement(
        &s.p,
        ReverseRecord {
            id: m.id,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
}
