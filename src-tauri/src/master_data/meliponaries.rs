use super::*;

pub async fn edit_meliponary(
    pool: &SqlitePool,
    input: EditMeliponary,
) -> Result<Meliponary, AppError> {
    let id = required(&input.id, "Meliponário")?;
    let name = required(&input.name, "Nome do meliponário")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    let current_name: Option<String> = sqlx::query_scalar("SELECT name FROM meliponaries WHERE id = ?")
        .bind(&id)
        .fetch_optional(pool)
        .await?;
    let current_name = current_name
        .ok_or_else(|| AppError::NotFound("Meliponário não encontrado.".to_owned()))?;
    if crate::identity::text_key(&current_name) != crate::identity::text_key(&name) {
        crate::identity::ensure_meliponary_name_available(pool, &name, Some(&id)).await?;
    }

    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE meliponaries
         SET name = ?, responsible_name = ?, location = ?, notes = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(name)
    .bind(optional(&input.responsible_name))
    .bind(optional(&input.location))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_meliponary(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
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

pub async fn archive_meliponary(
    pool: &SqlitePool,
    input: EntityAction,
) -> Result<Meliponary, AppError> {
    let id = required(&input.id, "Meliponário")?;
    let reason = required(&input.reason, "Motivo do arquivamento")?;
    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    if before.archived_at.is_some() {
        return Err(AppError::Validation(
            "O meliponário já está arquivado.".to_owned(),
        ));
    }
    let archived_at = local_now(&mut tx).await?;
    sqlx::query(
        "UPDATE meliponaries
         SET archived_at = ?, archive_reason = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(archived_at)
    .bind(&reason)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_meliponary(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
        &id,
        "archive",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    agenda::reconcile_meliponary_tx(&mut tx, &id).await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn reactivate_meliponary(
    pool: &SqlitePool,
    input: EntityAction,
) -> Result<Meliponary, AppError> {
    let id = required(&input.id, "Meliponário")?;
    let reason = required(&input.reason, "Motivo da reativação")?;
    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    if before.archived_at.is_none() {
        return Err(AppError::Validation(
            "O meliponário já está ativo.".to_owned(),
        ));
    }
    sqlx::query(
        "UPDATE meliponaries
         SET archived_at = NULL, archive_reason = NULL, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_meliponary(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
        &id,
        "reactivate",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    agenda::reconcile_meliponary_tx(&mut tx, &id).await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn delete_meliponary(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Meliponário")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool = sqlx::query_scalar(
        "SELECT
            EXISTS(SELECT 1 FROM boxes WHERE meliponary_id = ?)
            OR EXISTS(SELECT 1 FROM colonies WHERE meliponary_id = ?)
            OR EXISTS(SELECT 1 FROM colony_movements
                      WHERE from_meliponary_id = ? OR to_meliponary_id = ?)",
    )
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if used {
        return Err(AppError::Validation(
            "Este meliponário já foi utilizado. Arquive-o em vez de excluí-lo.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    sqlx::query("DELETE FROM meliponaries WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
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
