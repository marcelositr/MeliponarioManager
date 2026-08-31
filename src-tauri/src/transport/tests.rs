
use super::*;
use crate::{
    movements::{self, CreateMovement},
    record_corrections::{self, VoidRecord},
};
use sqlx::sqlite::SqlitePoolOptions;

async fn seed() -> (SqlitePool, String) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();

    sqlx::query("INSERT INTO meliponaries(id,name) VALUES('m1','Origem')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO species(id,common_name) VALUES('s1','Jataí')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO boxes(id,meliponary_id,code,status) VALUES('b1','m1','CX-1','active')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO colonies(id,meliponary_id,species_id,code,status) VALUES('c1','m1','s1','JAT-1','active')")
            .execute(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at) VALUES('o1','c1','b1','2026-01-01 09:00:00')")
            .execute(&pool)
            .await
            .unwrap();

    (pool, "c1".to_owned())
}

async fn open_transport_at(
    pool: &SqlitePool,
    colony_id: &str,
    moved_at: &str,
    destination: &str,
) -> String {
    movements::create(
        pool,
        CreateMovement {
            colony_id: colony_id.to_owned(),
            movement_type: "transport".to_owned(),
            moved_at: Some(moved_at.to_owned()),
            to_meliponary_id: None,
            to_box_id: None,
            destination: Some(destination.to_owned()),
            document_reference: None,
            notes: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn open_transport(pool: &SqlitePool, colony_id: &str) -> String {
    open_transport_at(pool, colony_id, "2026-02-01 08:00:00", "Exposição").await
}

async fn open_transport_ids(pool: &SqlitePool, colony_id: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT m.id
             FROM colony_movements m
             WHERE m.colony_id = ?
               AND m.movement_type = 'transport'
               AND m.voided_at IS NULL
               AND m.reversed_at IS NULL
               AND NOT EXISTS (
                   SELECT 1 FROM transport_returns r
                   WHERE r.movement_id = m.id AND r.reversed_at IS NULL
               )
             ORDER BY m.moved_at, m.id",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn movement_audit_count(pool: &SqlitePool, movement_id: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_records
             WHERE entity_type = 'movement' AND entity_id = ?",
    )
    .bind(movement_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn transport_return_completes_and_can_be_reopened_without_erasing_history() {
    let (pool, colony_id) = seed().await;
    let movement_id = open_transport(&pool, &colony_id).await;

    let returned = complete(
        &pool,
        CompleteTransport {
            movement_id: movement_id.clone(),
            returned_at: Some("2026-02-02 18:00:00".to_owned()),
            notes: Some("Retorno sem intercorrências".to_owned()),
        },
    )
    .await
    .unwrap();
    assert_eq!(returned.movement_id, movement_id);
    assert_eq!(list_by_colony(&pool, &colony_id).await.unwrap().len(), 1);

    reopen(&pool, &movement_id, "Data de retorno lançada por engano")
        .await
        .unwrap();
    assert!(list_by_colony(&pool, &colony_id).await.unwrap().is_empty());

    let preserved: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM transport_returns WHERE movement_id = ? AND reversed_at IS NOT NULL",
    )
    .bind(&movement_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(preserved, 1);
}

#[tokio::test]
async fn parallel_open_transport_is_blocked_and_return_must_follow_departure() {
    let (pool, colony_id) = seed().await;
    let movement_id = open_transport(&pool, &colony_id).await;

    let second = movements::create(
        &pool,
        CreateMovement {
            colony_id: colony_id.clone(),
            movement_type: "transport".to_owned(),
            moved_at: Some("2026-02-01 09:00:00".to_owned()),
            to_meliponary_id: None,
            to_box_id: None,
            destination: Some("Outro evento".to_owned()),
            document_reference: None,
            notes: None,
        },
    )
    .await;
    assert!(matches!(second, Err(AppError::Database(_))));

    let invalid_return = complete(
        &pool,
        CompleteTransport {
            movement_id,
            returned_at: Some("2026-01-31 18:00:00".to_owned()),
            notes: None,
        },
    )
    .await;
    assert!(matches!(invalid_return, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn reopening_completed_transport_is_blocked_while_another_transport_is_open() {
    let (pool, colony_id) = seed().await;
    let transport_a =
        open_transport_at(&pool, &colony_id, "2026-02-01 08:00:00", "Exposição A").await;
    complete(
        &pool,
        CompleteTransport {
            movement_id: transport_a.clone(),
            returned_at: Some("2026-02-02 18:00:00".to_owned()),
            notes: Some("Retorno A".to_owned()),
        },
    )
    .await
    .unwrap();

    let transport_b =
        open_transport_at(&pool, &colony_id, "2026-02-03 08:00:00", "Exposição B").await;
    let audit_before = movement_audit_count(&pool, &transport_a).await;

    let blocked = reopen(&pool, &transport_a, "Corrigir retorno A").await;
    match blocked {
        Err(AppError::Validation(message)) => {
            assert_eq!(message, OTHER_OPEN_TRANSPORT_MESSAGE);
        }
        other => panic!("reabertura deveria ser bloqueada por validação de domínio: {other:?}"),
    }

    assert_eq!(
        open_transport_ids(&pool, &colony_id).await,
        vec![transport_b.clone()]
    );
    let active_return_a: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM transport_returns
             WHERE movement_id = ? AND reversed_at IS NULL",
    )
    .bind(&transport_a)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_return_a, 1);
    assert_eq!(
        movement_audit_count(&pool, &transport_a).await,
        audit_before
    );

    let direct_sql = sqlx::query(
        "UPDATE transport_returns
             SET reversed_at = '2026-02-03 10:00:00', reversal_reason = 'Bypass direto'
             WHERE movement_id = ? AND reversed_at IS NULL",
    )
    .bind(&transport_a)
    .execute(&pool)
    .await;
    let direct_error = direct_sql.expect_err("trigger SQLite deve bloquear reabertura concorrente");
    assert!(direct_error
        .to_string()
        .contains("A colônia já possui outro transporte temporário aberto."));

    let preserved_active_a: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM transport_returns
             WHERE movement_id = ? AND reversed_at IS NULL",
    )
    .bind(&transport_a)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(preserved_active_a, 1);
    assert_eq!(
        movement_audit_count(&pool, &transport_a).await,
        audit_before
    );

    complete(
        &pool,
        CompleteTransport {
            movement_id: transport_b.clone(),
            returned_at: Some("2026-02-04 18:00:00".to_owned()),
            notes: Some("Retorno B".to_owned()),
        },
    )
    .await
    .unwrap();
    assert!(open_transport_ids(&pool, &colony_id).await.is_empty());

    reopen(&pool, &transport_a, "Corrigir retorno A")
        .await
        .unwrap();
    assert_eq!(
        open_transport_ids(&pool, &colony_id).await,
        vec![transport_a.clone()]
    );

    let return_history_a: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), SUM(CASE WHEN reversed_at IS NOT NULL THEN 1 ELSE 0 END)
             FROM transport_returns WHERE movement_id = ?",
    )
    .bind(&transport_a)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(return_history_a, (1, 1));

    let active_return_b: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM transport_returns
             WHERE movement_id = ? AND reversed_at IS NULL",
    )
    .bind(&transport_b)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_return_b, 1);

    let reopen_audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_records
             WHERE entity_type = 'movement'
               AND entity_id = ?
               AND action = 'reopen_transport'",
    )
    .bind(&transport_a)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(reopen_audits, 1);
}

#[tokio::test]
async fn completed_transport_must_be_reopened_before_administrative_void() {
    let (pool, colony_id) = seed().await;
    let movement_id = open_transport(&pool, &colony_id).await;
    complete(
        &pool,
        CompleteTransport {
            movement_id: movement_id.clone(),
            returned_at: Some("2026-02-02 18:00:00".to_owned()),
            notes: None,
        },
    )
    .await
    .unwrap();

    let blocked = record_corrections::void_transport(
        &pool,
        VoidRecord {
            id: movement_id.clone(),
            reason: "Lançamento incorreto".to_owned(),
        },
    )
    .await;
    assert!(matches!(blocked, Err(AppError::Database(_))));

    reopen(&pool, &movement_id, "Retorno precisa ser corrigido")
        .await
        .unwrap();
    record_corrections::void_transport(
        &pool,
        VoidRecord {
            id: movement_id.clone(),
            reason: "Movimento lançado por engano".to_owned(),
        },
    )
    .await
    .unwrap();

    let voided_at: Option<String> =
        sqlx::query_scalar("SELECT voided_at FROM colony_movements WHERE id = ?")
            .bind(&movement_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(voided_at.is_some());

    let preserved_returns: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM transport_returns WHERE movement_id = ?")
            .bind(&movement_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(preserved_returns, 1);
}
