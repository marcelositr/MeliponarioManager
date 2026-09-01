use super::*;

pub async fn edit_colony(pool: &SqlitePool, input: EditColony) -> Result<Colony, AppError> {
    let id = required(&input.id, "Colônia")?;
    let code = required(&input.code, "Identificação da colônia")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    let current: Option<(String, String)> =
        sqlx::query_as("SELECT meliponary_id, code FROM colonies WHERE id = ?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let (meliponary_id, current_code) =
        current.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;
    if crate::identity::text_key(&current_code) != crate::identity::text_key(&code) {
        crate::identity::ensure_colony_code_available(pool, &meliponary_id, &code, Some(&id))
            .await?;
    }

    let mut tx = pool.begin().await?;
    let before = get_colony(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE colonies
         SET code = ?, origin_notes = ?, notes = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(code)
    .bind(optional(&input.origin_notes))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_colony(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "colony",
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

pub async fn delete_colony(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Colônia")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool = sqlx::query_scalar(
        "SELECT
            EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM inspections WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM feedings WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM production_records WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_events WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_divisions
                      WHERE parent_colony_id = ? OR daughter_colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_movements WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM box_maintenance_records WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colonies WHERE mother_colony_id = ?)
            OR EXISTS(SELECT 1 FROM colonies WHERE id = ? AND mother_colony_id IS NOT NULL)",
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
    .bind(&id)
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if used {
        return Err(AppError::Validation(
            "Esta colônia possui histórico ou vínculo genealógico e não pode ser excluída. Use o ciclo de vida adequado.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_colony(&mut tx, &id).await?;
    sqlx::query("DELETE FROM colonies WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "colony",
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
