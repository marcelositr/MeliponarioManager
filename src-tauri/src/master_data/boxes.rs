use super::*;

pub async fn edit_box(pool: &SqlitePool, input: EditBox) -> Result<HiveBox, AppError> {
    let id = required(&input.id, "Caixa")?;
    let code = required(&input.code, "Identificação da caixa")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    let meliponary_id: Option<String> =
        sqlx::query_scalar("SELECT meliponary_id FROM boxes WHERE id = ?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let meliponary_id =
        meliponary_id.ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))?;
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM boxes
            WHERE id <> ? AND meliponary_id = ?
              AND lower(trim(code)) = lower(trim(?))
         )",
    )
    .bind(&id)
    .bind(&meliponary_id)
    .bind(&code)
    .fetch_one(pool)
    .await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe uma caixa com esta identificação no meliponário.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let before = get_box(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE boxes
         SET code = ?, model = ?, material = ?, location_note = ?, notes = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(code)
    .bind(optional(&input.model))
    .bind(optional(&input.material))
    .bind(optional(&input.location_note))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_box(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "box",
        &id,
        "edit",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn delete_box(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Caixa")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool = sqlx::query_scalar(
        "SELECT
            EXISTS(SELECT 1 FROM colony_box_occupancies WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM box_maintenance_records WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM colony_events WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM inspections WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM feedings WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM production_records WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM colony_movements WHERE from_box_id = ? OR to_box_id = ?)
            OR EXISTS(SELECT 1 FROM box_state_records WHERE box_id = ?)",
    )
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if used {
        return Err(AppError::Validation(
            "Esta caixa possui histórico. Aposente-a em vez de excluí-la.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_box(&mut tx, &id).await?;
    sqlx::query("DELETE FROM boxes WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "box",
        &id,
        "delete",
        &reason,
        Some(audit::value(&before)?),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
