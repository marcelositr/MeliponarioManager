use super::*;

pub async fn reverse_movement(p: &SqlitePool, input: ReverseRecord) -> Result<(), AppError> {
    let id = req(&input.id, "Movimentação")?;
    let reason = req(&input.reason, "Motivo da reversão")?;
    type MovementReversalRow = (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    );
    let row: Option<MovementReversalRow> = sqlx::query_as("SELECT colony_id,movement_type,moved_at,from_meliponary_id,to_meliponary_id,from_box_id,to_box_id,reversed_at FROM colony_movements WHERE id=? AND voided_at IS NULL").bind(&id).fetch_optional(p).await?;
    let (c, kind, at, from_m, to_m, from_b, to_b, reversed) =
        row.ok_or_else(|| AppError::NotFound("Movimentação não encontrada.".into()))?;
    if reversed.is_some() {
        return Err(AppError::Validation(
            "Esta movimentação já foi revertida.".into(),
        ));
    }
    if kind == "transport" {
        return Err(AppError::Validation(
            "Transporte sem consequência deve ser anulado, não revertido.".into(),
        ));
    }
    let latest:Option<String>=sqlx::query_scalar("SELECT id FROM colony_movements WHERE colony_id=? AND movement_type IN('internal_transfer','external_transfer') AND voided_at IS NULL AND reversed_at IS NULL ORDER BY moved_at DESC,created_at DESC,id DESC LIMIT 1").bind(&c).fetch_optional(p).await?;
    if latest.as_deref() != Some(id.as_str()) {
        return Err(AppError::Validation(
            "Somente a transferência efetiva mais recente pode ser revertida automaticamente."
                .into(),
        ));
    }
    if later_facts(p, &c, &at, None, Some(&id)).await? {
        return Err(AppError::Validation(
            "Existem fatos posteriores à transferência; a reversão automática foi bloqueada."
                .into(),
        ));
    }
    let mut tx = p.begin().await?;
    let reversed_at = now(&mut tx).await?;
    let before = json!({"id":id,"colony_id":c,"movement_type":kind,"moved_at":at,"from_meliponary_id":from_m,"to_meliponary_id":to_m,"from_box_id":from_b,"to_box_id":to_b});
    match kind.as_str() {
        "internal_transfer" => {
            let current_m: String =
                sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id=?")
                    .bind(&c)
                    .fetch_one(&mut *tx)
                    .await?;
            if Some(current_m.as_str()) != to_m.as_deref() {
                return Err(AppError::Validation(
                    "A colônia já não está no destino registrado por esta transferência.".into(),
                ));
            }
            if let Some(tb) = &to_b {
                let occ:Option<(String,String)>=sqlx::query_as("SELECT id,started_at FROM colony_box_occupancies WHERE colony_id=? AND box_id=? AND ended_at IS NULL").bind(&c).bind(tb).fetch_optional(&mut*tx).await?;
                let (oid, start) = occ.ok_or_else(|| {
                    AppError::Validation(
                        "A ocupação criada pela transferência já não é a ocupação atual.".into(),
                    )
                })?;
                if start != at {
                    return Err(AppError::Validation(
                        "A ocupação atual não corresponde exatamente à transferência original."
                            .into(),
                    ));
                }
                sqlx::query("UPDATE colony_box_occupancies SET ended_at=? WHERE id=?")
                    .bind(&reversed_at)
                    .bind(oid)
                    .execute(&mut *tx)
                    .await?;
            } else {
                let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL)").bind(&c).fetch_one(&mut*tx).await?;
                if active {
                    return Err(AppError::Validation("A colônia recebeu uma caixa após a transferência; a reversão automática foi bloqueada.".into()));
                }
            }
            sqlx::query(
                "UPDATE colonies SET meliponary_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?",
            )
            .bind(&from_m)
            .bind(&c)
            .execute(&mut *tx)
            .await?;
            restore_box(&mut tx, &c, from_b.as_deref(), &reversed_at, &reason).await?
        }
        "external_transfer" => {
            let status: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
                .bind(&c)
                .fetch_one(&mut *tx)
                .await?;
            if status != "transferred" {
                return Err(AppError::Validation(
                    "A colônia já não está marcada como transferida.".into(),
                ));
            }
            let previous:Option<String>=sqlx::query_scalar("SELECT new_status FROM colony_lifecycle_records WHERE colony_id=? AND occurred_at<? AND reversed_at IS NULL ORDER BY occurred_at DESC,created_at DESC,id DESC LIMIT 1").bind(&c).bind(&at).fetch_optional(&mut*tx).await?;
            let previous=previous.ok_or_else(||AppError::Validation("O estado anterior à transferência externa não está historicamente comprovado; a reversão automática foi bloqueada.".into()))?;
            if !operational::is_manageable_status(&previous) {
                return Err(AppError::Validation(
                    "O estado anterior não permite restauração automática ao plantel.".into(),
                ));
            }
            sqlx::query("UPDATE colonies SET status=?,meliponary_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(previous).bind(&from_m).bind(&c).execute(&mut*tx).await?;
            restore_box(&mut tx, &c, from_b.as_deref(), &reversed_at, &reason).await?
        }
        _ => {
            return Err(AppError::Validation(
                "Tipo de movimentação sem reversão automática.".into(),
            ))
        }
    };
    sqlx::query("UPDATE colony_movements SET reversed_at=?,reversal_reason=? WHERE id=?")
        .bind(&reversed_at)
        .bind(&reason)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "movement",
        &id,
        "reverse",
        &reason,
        Some(before),
        Some(json!({"reversed_at":reversed_at,"reversal_reason":reason})),
    )
    .await?;
    agenda::reconcile_inspection_tx(&mut tx, &c).await?;
    agenda::reconcile_feeding_tx(&mut tx, &c).await?;
    tx.commit().await?;
    Ok(())
}
