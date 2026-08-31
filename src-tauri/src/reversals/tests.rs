
use super::*;
use crate::{
    domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
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
#[tokio::test]
async fn safe_lifecycle_reversal_restores_status_and_box() {
    let s = seed().await;
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
async fn internal_transfer_reversal_is_transactional_and_preserves_original() {
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
