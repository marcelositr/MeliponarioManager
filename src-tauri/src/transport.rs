use crate::{audit, repository::AppError, time};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, SqlitePool};
use tauri::State;
use uuid::Uuid;

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

pub async fn ensure_can_open(pool: &SqlitePool, colony_id: &str) -> Result<(), AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    let open: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM colony_movements m
            WHERE m.colony_id = ?
              AND m.movement_type = 'transport'
              AND m.voided_at IS NULL
              AND m.reversed_at IS NULL
              AND NOT EXISTS (
                  SELECT 1
                  FROM transport_returns r
                  WHERE r.movement_id = m.id
                    AND r.reversed_at IS NULL
              )
        )",
    )
    .bind(colony_id)
    .fetch_one(pool)
    .await?;

    if open {
        Err(AppError::Validation(
            "Esta colônia já possui um transporte temporário aberto. Registre o retorno antes de iniciar outro."
                .to_owned(),
        ))
    } else {
        Ok(())
    }
}

pub async fn ensure_can_void(pool: &SqlitePool, movement_id: &str) -> Result<(), AppError> {
    let movement_id = required(movement_id, "Movimentação")?;
    let active_return: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM transport_returns
            WHERE movement_id = ? AND reversed_at IS NULL
        )",
    )
    .bind(movement_id)
    .fetch_one(pool)
    .await?;

    if active_return {
        Err(AppError::Validation(
            "Este transporte já foi concluído. Reabra o transporte antes de anular o movimento original."
                .to_owned(),
        ))
    } else {
        Ok(())
    }
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

    let movement: Option<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT colony_id, movement_type, moved_at, voided_at, reversed_at
         FROM colony_movements
         WHERE id = ?",
    )
    .bind(&movement_id)
    .fetch_optional(pool)
    .await?;
    let (_colony_id, movement_type, moved_at, voided_at, reversed_at) = movement
        .ok_or_else(|| AppError::NotFound("Transporte não encontrado.".to_owned()))?;

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

pub async fn reopen(
    pool: &SqlitePool,
    movement_id: &str,
    reason: &str,
) -> Result<(), AppError> {
    let movement_id = required(movement_id, "Transporte")?;
    let reason = required(reason, "Motivo da reabertura")?;

    let movement_type: Option<String> =
        sqlx::query_scalar("SELECT movement_type FROM colony_movements WHERE id = ?")
            .bind(&movement_id)
            .fetch_optional(pool)
            .await?;
    match movement_type.as_deref() {
        Some("transport") => {}
        Some(_) => {
            return Err(AppError::Validation(
                "Somente transporte temporário pode ser reaberto por este fluxo.".to_owned(),
            ))
        }
        None => return Err(AppError::NotFound("Transporte não encontrado.".to_owned())),
    }

    let active = get_active_return(pool, &movement_id)
        .await?
        .ok_or_else(|| AppError::Validation("Este transporte já está aberto.".to_owned()))?;

    let mut tx = pool.begin().await?;
    let reversed_at: String = sqlx::query_scalar(
        "SELECT strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')",
    )
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
    .await?;

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
    use crate::movements::{self, CreateMovement};
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
        sqlx::query("INSERT INTO boxes(id,meliponary_id,code,status) VALUES('b1','m1','CX-1','active')")
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

    async fn open_transport(pool: &SqlitePool, colony_id: &str) -> String {
        movements::create(
            pool,
            CreateMovement {
                colony_id: colony_id.to_owned(),
                movement_type: "transport".to_owned(),
                moved_at: Some("2026-02-01 08:00:00".to_owned()),
                to_meliponary_id: None,
                to_box_id: None,
                destination: Some("Exposição".to_owned()),
                document_reference: None,
                notes: None,
            },
        )
        .await
        .unwrap()
        .id
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

        assert!(matches!(
            ensure_can_open(&pool, &colony_id).await,
            Err(AppError::Validation(_))
        ));

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
    async fn completed_transport_cannot_be_voided_before_reopening() {
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

        assert!(matches!(
            ensure_can_void(&pool, &movement_id).await,
            Err(AppError::Validation(_))
        ));
    }
}
