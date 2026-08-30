use crate::{audit, repository::AppError, time};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, SqlitePool};
use tauri::State;
use uuid::Uuid;

const OTHER_OPEN_TRANSPORT_MESSAGE: &str = "Esta colônia já possui outro transporte temporário aberto. Conclua ou anule o transporte atual antes de reabrir este.";

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TransportReturn {
    pub id: String,
    pub movement_id: String,
    pub returned_at: String,
    pub notes: Option<String>,
    pub reversed_at: Option<String>,
    pub reversal_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteTransport {
    pub movement_id: String,
    pub returned_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReopenTransport {
    pub movement_id: String,
    pub reason: String,
}

fn required(value: &str, field: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} é obrigatório.")));
    }
    Ok(value.to_owned())
}

fn optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn message(error: AppError) -> String {
    error.to_string()
}

fn reopen_write_error(error: sqlx::Error) -> AppError {
    if error
        .to_string()
        .contains("A colônia já possui outro transporte temporário aberto.")
    {
        AppError::Validation(OTHER_OPEN_TRANSPORT_MESSAGE.to_owned())
    } else {
        AppError::Database(error)
    }
}

pub async fn has_open_transport_for_colony(
    pool: &SqlitePool,
    colony_id: &str,
    exclude_movement_id: Option<&str>,
) -> Result<bool, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM colony_movements m
            WHERE m.colony_id = ?
              AND m.movement_type = 'transport'
              AND m.voided_at IS NULL
              AND m.reversed_at IS NULL
              AND (? IS NULL OR m.id <> ?)
              AND NOT EXISTS (
                  SELECT 1
                  FROM transport_returns r
                  WHERE r.movement_id = m.id
                    AND r.reversed_at IS NULL
              )
        )",
    )
    .bind(&colony_id)
    .bind(exclude_movement_id)
    .bind(exclude_movement_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

pub async fn complete(
    pool: &SqlitePool,
    input: CompleteTransport,
) -> Result<TransportReturn, AppError> {
    let movement_id = required(&input.movement_id, "Transporte")?;
    let returned_at = required(
        input.returned_at.as_deref().unwrap_or_default(),
        "Data de retorno",
    )?;
    let notes = optional(&input.notes);

    let movement: Option<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT movement_type, moved_at, voided_at, reversed_at
         FROM colony_movements
         WHERE id = ?",
    )
    .bind(&movement_id)
    .fetch_optional(pool)
    .await?;
    let (movement_type, moved_at, voided_at, reversed_at) =
        movement.ok_or_else(|| AppError::NotFound("Transporte não encontrado.".to_owned()))?;

    if movement_type != "transport" {
        return Err(AppError::Validation(
            "Somente transportes temporários possuem retorno operacional.".to_owned(),
        ));
    }
    if voided_at.is_some() || reversed_at.is_some() {
        return Err(AppError::Validation(
            "Transporte anulado ou revertido não pode receber retorno.".to_owned(),
        ));
    }
    if returned_at < moved_at {
        return Err(AppError::Validation(
            "A data de retorno não pode ser anterior ao início do transporte.".to_owned(),
        ));
    }

    let already_completed: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM transport_returns
            WHERE movement_id = ? AND reversed_at IS NULL
        )",
    )
    .bind(&movement_id)
    .fetch_one(pool)
    .await?;
    if already_completed {
        return Err(AppError::Validation(
            "Este transporte temporário já possui retorno registrado.".to_owned(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO transport_returns (id, movement_id, returned_at, notes)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&movement_id)
    .bind(&returned_at)
    .bind(notes.clone())
    .execute(&mut *tx)
    .await?;

    audit::record_tx(
        &mut tx,
        "movement",
        &movement_id,
        "complete_transport",
        "Retorno do transporte temporário",
        Some(json!({ "transport_status": "open" })),
        Some(json!({
            "transport_status": "completed",
            "transport_return_id": id,
            "returned_at": returned_at,
            "return_notes": notes,
        })),
    )
    .await?;
    tx.commit().await?;

    get_active_return(pool, &movement_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Retorno do transporte não encontrado.".to_owned()))
}

pub async fn reopen(pool: &SqlitePool, movement_id: &str, reason: &str) -> Result<(), AppError> {
    let movement_id = required(movement_id, "Transporte")?;
    let reason = required(reason, "Motivo da reabertura")?;

    let movement: Option<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT movement_type, colony_id, voided_at, reversed_at
         FROM colony_movements
         WHERE id = ?",
    )
    .bind(&movement_id)
    .fetch_optional(pool)
    .await?;
    let (movement_type, colony_id, voided_at, reversed_at) =
        movement.ok_or_else(|| AppError::NotFound("Transporte não encontrado.".to_owned()))?;

    if movement_type != "transport" {
        return Err(AppError::Validation(
            "Somente transporte temporário pode ser reaberto por este fluxo.".to_owned(),
        ));
    }
    if voided_at.is_some() || reversed_at.is_some() {
        return Err(AppError::Validation(
            "Transporte anulado ou revertido não pode ser reaberto.".to_owned(),
        ));
    }

    let active = get_active_return(pool, &movement_id)
        .await?
        .ok_or_else(|| AppError::Validation("Este transporte já está aberto.".to_owned()))?;

    if has_open_transport_for_colony(pool, &colony_id, Some(&movement_id)).await? {
        return Err(AppError::Validation(
            OTHER_OPEN_TRANSPORT_MESSAGE.to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let reversed_at: String =
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')")
            .fetch_one(&mut *tx)
            .await?;

    sqlx::query(
        "UPDATE transport_returns
         SET reversed_at = ?, reversal_reason = ?
         WHERE id = ? AND reversed_at IS NULL",
    )
    .bind(&reversed_at)
    .bind(&reason)
    .bind(&active.id)
    .execute(&mut *tx)
    .await
    .map_err(reopen_write_error)?;

    audit::record_tx(
        &mut tx,
        "movement",
        &movement_id,
        "reopen_transport",
        &reason,
        Some(json!({
            "transport_status": "completed",
            "transport_return_id": active.id,
            "returned_at": active.returned_at,
            "return_notes": active.notes,
        })),
        Some(json!({
            "transport_status": "open",
            "return_reversed_at": reversed_at,
        })),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<TransportReturn>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    Ok(sqlx::query_as::<_, TransportReturn>(
        "SELECT r.id, r.movement_id, r.returned_at, r.notes,
                r.reversed_at, r.reversal_reason, r.created_at
         FROM transport_returns r
         JOIN colony_movements m ON m.id = r.movement_id
         WHERE m.colony_id = ? AND r.reversed_at IS NULL
         ORDER BY r.returned_at DESC, r.created_at DESC, r.id DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

async fn get_active_return(
    pool: &SqlitePool,
    movement_id: &str,
) -> Result<Option<TransportReturn>, AppError> {
    Ok(sqlx::query_as::<_, TransportReturn>(
        "SELECT id, movement_id, returned_at, notes,
                reversed_at, reversal_reason, created_at
         FROM transport_returns
         WHERE movement_id = ? AND reversed_at IS NULL
         ORDER BY returned_at DESC, created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(movement_id)
    .fetch_optional(pool)
    .await?)
}

#[tauri::command]
pub async fn complete_transport(
    pool: State<'_, SqlitePool>,
    mut input: CompleteTransport,
) -> Result<TransportReturn, String> {
    input.returned_at = Some(
        time::normalize_or_now(&pool, &input.returned_at, false)
            .await
            .map_err(message)?,
    );
    complete(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_transport_returns(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<TransportReturn>, String> {
    list_by_colony(&pool, &colony_id).await.map_err(message)
}

#[tauri::command]
pub async fn reopen_transport(
    pool: State<'_, SqlitePool>,
    input: ReopenTransport,
) -> Result<(), String> {
    reopen(&pool, &input.movement_id, &input.reason)
        .await
        .map_err(message)
}

#[cfg(test)]
mod tests {
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
        let direct_error =
            direct_sql.expect_err("trigger SQLite deve bloquear reabertura concorrente");
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
}
