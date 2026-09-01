use super::*;

pub async fn reverse_lifecycle(p: &SqlitePool, input: ReverseRecord) -> Result<(), AppError> {
    let id = req(&input.id, "Transição")?;
    let reason = req(&input.reason, "Motivo da reversão")?;
    type LifecycleReversalRow = (
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    );
    let row: Option<LifecycleReversalRow> = sqlx::query_as("SELECT colony_id,action,occurred_at,previous_status,new_status,box_id,reversed_at FROM colony_lifecycle_records WHERE id=?").bind(&id).fetch_optional(p).await?;
    let (c, action, at, previous, new_status, box_id, reversed) =
        row.ok_or_else(|| AppError::NotFound("Transição não encontrada.".into()))?;
    if reversed.is_some() {
        return Err(AppError::Validation(
            "Esta transição já foi revertida.".into(),
        ));
    }
    let latest:Option<String>=sqlx::query_scalar("SELECT id FROM colony_lifecycle_records WHERE colony_id=? AND reversed_at IS NULL ORDER BY occurred_at DESC,created_at DESC,id DESC LIMIT 1").bind(&c).fetch_optional(p).await?;
    if latest.as_deref() != Some(id.as_str()) {
        return Err(AppError::Validation(
            "Somente a transição efetiva mais recente pode ser revertida automaticamente.".into(),
        ));
    }
    if later_facts(p, &c, &at, Some(&id), None).await? {
        return Err(AppError::Validation(
            "Existem fatos posteriores incompatíveis com a reversão. Corrija esses fatos primeiro."
                .into(),
        ));
    }
    let current: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
        .bind(&c)
        .fetch_one(p)
        .await?;
    if current != new_status {
        return Err(AppError::Validation(
            "O estado atual da colônia já não corresponde ao resultado desta transição.".into(),
        ));
    }
    let mut tx = p.begin().await?;
    let reversed_at = now(&mut tx).await?;
    let before = json!({"id":id,"colony_id":c,"action":action,"occurred_at":at,"previous_status":previous,"new_status":new_status,"box_id":box_id});
    match action.as_str() {
        "loss" | "deactivate" => {
            sqlx::query("UPDATE colonies SET status=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                .bind(&previous)
                .bind(&c)
                .execute(&mut *tx)
                .await?;
            restore_box(&mut tx, &c, box_id.as_deref(), &reversed_at, &reason).await?
        }
        "reactivate" => {
            let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL)").bind(&c).fetch_one(&mut*tx).await?;
            if active {
                return Err(AppError::Validation("A colônia recebeu uma caixa após a reativação; a reversão automática foi bloqueada.".into()));
            }
            sqlx::query("UPDATE colonies SET status=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                .bind(&previous)
                .bind(&c)
                .execute(&mut *tx)
                .await?;
        }
        _ => {
            return Err(AppError::Validation(
                "Ação de ciclo de vida sem reversão automática.".into(),
            ))
        }
    };
    sqlx::query("UPDATE colony_lifecycle_records SET reversed_at=?,reversal_reason=? WHERE id=?")
        .bind(&reversed_at)
        .bind(&reason)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "lifecycle",
        &id,
        "reverse",
        &reason,
        Some(before),
        Some(json!({"reversed_at":reversed_at,"reversal_reason":reason})),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
