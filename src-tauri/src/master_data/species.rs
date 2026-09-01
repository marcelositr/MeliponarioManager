use super::*;

pub async fn edit_species(pool: &SqlitePool, input: EditSpecies) -> Result<Species, AppError> {
    let id = required(&input.id, "Espécie")?;
    let common_name = required(&input.common_name, "Nome popular")?;
    let scientific_name = optional(&input.scientific_name);
    let reason = required(&input.reason, "Motivo da edição")?;
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM species
            WHERE id <> ?
              AND lower(trim(common_name)) = lower(trim(?))
              AND lower(trim(COALESCE(scientific_name, ''))) = lower(trim(COALESCE(?, '')))
         )",
    )
    .bind(&id)
    .bind(&common_name)
    .bind(&scientific_name)
    .fetch_one(pool)
    .await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe uma espécie com estes nomes cadastrais.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE species
         SET common_name = ?, scientific_name = ?, genus = ?, notes = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(common_name)
    .bind(scientific_name)
    .bind(optional(&input.genus))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_species(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "species",
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

pub async fn archive_species(pool: &SqlitePool, input: EntityAction) -> Result<Species, AppError> {
    let id = required(&input.id, "Espécie")?;
    let reason = required(&input.reason, "Motivo do arquivamento")?;
    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    if before.archived_at.is_some() {
        return Err(AppError::Validation(
            "A espécie já está arquivada.".to_owned(),
        ));
    }
    let archived_at = local_now(&mut tx).await?;
    sqlx::query(
        "UPDATE species
         SET archived_at = ?, archive_reason = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(archived_at)
    .bind(&reason)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_species(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "species",
        &id,
        "archive",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn reactivate_species(
    pool: &SqlitePool,
    input: EntityAction,
) -> Result<Species, AppError> {
    let id = required(&input.id, "Espécie")?;
    let reason = required(&input.reason, "Motivo da reativação")?;
    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    if before.archived_at.is_none() {
        return Err(AppError::Validation("A espécie já está ativa.".to_owned()));
    }
    sqlx::query(
        "UPDATE species
         SET archived_at = NULL, archive_reason = NULL, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_species(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "species",
        &id,
        "reactivate",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn delete_species(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Espécie")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE species_id = ?)")
            .bind(&id)
            .fetch_one(pool)
            .await?;
    if used {
        return Err(AppError::Validation(
            "Esta espécie já foi utilizada. Arquive-a em vez de excluí-la.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    sqlx::query("DELETE FROM species WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "species",
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
