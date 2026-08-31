use super::*;

pub async fn correct_movement_details(
    pool: &SqlitePool,
    input: CorrectMovementDetails,
) -> Result<(), AppError> {
    let id = required(&input.id, "Movimentação")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let movement: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT movement_type, voided_at, reversed_at FROM colony_movements WHERE id=?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let (movement_type, voided_at, reversed_at) =
        movement.ok_or_else(|| AppError::NotFound("Movimentação não encontrada.".to_owned()))?;
    if voided_at.is_some() || reversed_at.is_some() {
        return Err(AppError::Validation(
            "Movimentação anulada ou revertida não pode ser editada.".to_owned(),
        ));
    }
    let destination = optional(&input.destination);
    if movement_type != "internal_transfer" && destination.is_none() {
        return Err(AppError::Validation(
            "Informe o destino da movimentação.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'movement_type',movement_type,'moved_at',moved_at,'from_meliponary_id',from_meliponary_id,'to_meliponary_id',to_meliponary_id,'from_box_id',from_box_id,'to_box_id',to_box_id,'destination',destination,'notes',notes,'voided_at',voided_at,'reversed_at',reversed_at) FROM colony_movements WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Movimentação não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_movements SET destination=?, notes=?, corrected_at=? WHERE id=?")
        .bind(if movement_type == "internal_transfer" {
            None::<String>
        } else {
            destination
        })
        .bind(optional(&input.notes))
        .bind(corrected_at)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Movimentação não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "movement",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_transport(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    let id = required(&input.id, "Movimentação")?;
    let kind: Option<String> =
        sqlx::query_scalar("SELECT movement_type FROM colony_movements WHERE id=?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    match kind.as_deref() {
        Some("transport") => void_fact(pool, VoidRecord { id, reason: input.reason }, "movement", "colony_movements",
            "SELECT json_object('id',id,'movement_type',movement_type,'moved_at',moved_at,'destination',destination,'notes',notes,'voided_at',voided_at,'void_reason',void_reason,'reversed_at',reversed_at) FROM colony_movements WHERE id=?").await,
        Some(_) => Err(AppError::Validation("Transferências têm consequências e precisam ser revertidas pelo fluxo específico.".to_owned())),
        None => Err(AppError::NotFound("Movimentação não encontrada.".to_owned())),
    }
}

pub async fn update_movement_document(
    pool: &SqlitePool,
    input: UpdateMovementDocument,
) -> Result<(), AppError> {
    let id = required(&input.id, "Documento")?;
    let document_type = required(&input.document_type, "Tipo")?;
    let reference = required(&input.reference_number, "Referência")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    if !DOCUMENT_TYPES.contains(&document_type.as_str()) {
        return Err(AppError::Validation(
            "Tipo de documento inválido.".to_owned(),
        ));
    }
    let issued_at = time::normalize_optional(&input.issued_at, false)?;
    let valid_until = time::normalize_optional(&input.valid_until, false)?;
    if let (Some(issued), Some(valid)) = (&issued_at, &valid_until) {
        if valid < issued {
            return Err(AppError::Validation(
                "A validade não pode ser anterior à emissão.".to_owned(),
            ));
        }
    }
    let movement_id: Option<String> = sqlx::query_scalar(
        "SELECT movement_id FROM movement_documents WHERE id=? AND voided_at IS NULL",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let movement_id = movement_id.ok_or_else(|| {
        AppError::Validation("Documento não encontrado ou invalidado.".to_owned())
    })?;
    let duplicate: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM movement_documents WHERE id<>? AND movement_id=? AND document_type=? AND reference_number=? AND voided_at IS NULL)")
        .bind(&id).bind(&movement_id).bind(&document_type).bind(&reference).fetch_one(pool).await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe um documento válido com esta referência na movimentação.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'movement_id',movement_id,'document_type',document_type,'reference_number',reference_number,'source_system',source_system,'issuer',issuer,'issued_at',issued_at,'valid_until',valid_until,'file_path',file_path,'notes',notes,'voided_at',voided_at) FROM movement_documents WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Documento não encontrado.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE movement_documents SET document_type=?, reference_number=?, source_system=?, issuer=?, issued_at=?, valid_until=?, file_path=?, notes=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(document_type).bind(reference).bind(optional(&input.source_system)).bind(optional(&input.issuer))
        .bind(issued_at).bind(valid_until).bind(optional(&input.file_path)).bind(optional(&input.notes))
        .bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Documento não encontrado.").await?;
    audit::record_tx(
        &mut tx,
        "movement_document",
        &id,
        "edit",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_movement_document(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "movement_document", "movement_documents",
        "SELECT json_object('id',id,'movement_id',movement_id,'document_type',document_type,'reference_number',reference_number,'voided_at',voided_at,'void_reason',void_reason) FROM movement_documents WHERE id=?")
        .await
}
