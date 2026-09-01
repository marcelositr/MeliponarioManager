use super::*;

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
