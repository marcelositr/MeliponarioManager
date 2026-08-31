use super::*;

pub async fn correct_division(pool: &SqlitePool, input: CorrectDivision) -> Result<(), AppError> {
    let id = required(&input.id, "Divisão")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'parent_colony_id',parent_colony_id,'daughter_colony_id',daughter_colony_id,'source_box_id',source_box_id,'performed_at',performed_at,'result',result,'notes',notes,'voided_at',voided_at) FROM colony_divisions WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    let already_void: Option<String> =
        sqlx::query_scalar("SELECT voided_at FROM colony_divisions WHERE id=?")
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;
    if already_void.is_some() {
        return Err(AppError::Validation(
            "Divisão anulada não pode ser corrigida.".to_owned(),
        ));
    }
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_divisions SET notes=?, corrected_at=? WHERE id=?")
        .bind(optional(&input.notes))
        .bind(corrected_at)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "division",
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

pub async fn void_division(pool: &SqlitePool, input: VoidDivision) -> Result<(), AppError> {
    let id = required(&input.id, "Divisão")?;
    let reason = required(&input.reason, "Motivo da anulação")?;
    let row: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("SELECT daughter_colony_id, voided_at FROM colony_divisions WHERE id=?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let (daughter_id, voided_at) =
        row.ok_or_else(|| AppError::NotFound("Divisão não encontrada.".to_owned()))?;
    if voided_at.is_some() {
        return Err(AppError::Validation(
            "A divisão já está anulada.".to_owned(),
        ));
    }

    let disposition = optional(&input.daughter_disposition);
    if let Some(daughter_id) = &daughter_id {
        let consequences: bool = sqlx::query_scalar(
            "SELECT
                EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM inspections WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM feedings WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM production_records WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_events WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_movements WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM box_maintenance_records WHERE colony_id=?)
                OR EXISTS(SELECT 1 FROM colony_divisions WHERE id<>? AND (parent_colony_id=? OR daughter_colony_id=?))
                OR EXISTS(SELECT 1 FROM colonies WHERE mother_colony_id=?)",
        )
        .bind(daughter_id).bind(daughter_id).bind(daughter_id).bind(daughter_id)
        .bind(daughter_id).bind(daughter_id).bind(daughter_id).bind(daughter_id)
        .bind(&id).bind(daughter_id).bind(daughter_id).bind(daughter_id)
        .fetch_one(pool).await?;
        if consequences {
            return Err(AppError::Validation(
                "A divisão criou uma filha que já possui consequências históricas. A anulação automática foi bloqueada para não reescrever o passado."
                    .to_owned(),
            ));
        }
        if !matches!(disposition.as_deref(), Some("keep") | Some("deactivate")) {
            return Err(AppError::Validation(
                "Informe explicitamente o destino da filha: keep para preservá-la ativa ou deactivate para inativá-la."
                    .to_owned(),
            ));
        }
    }

    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'parent_colony_id',parent_colony_id,'daughter_colony_id',daughter_colony_id,'performed_at',performed_at,'result',result,'notes',notes,'voided_at',voided_at,'void_reason',void_reason) FROM colony_divisions WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    let voided_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_divisions SET voided_at=?, void_reason=? WHERE id=?")
        .bind(&voided_at)
        .bind(&reason)
        .bind(&id)
        .execute(&mut *tx)
        .await?;

    if let (Some(daughter_id), Some("deactivate")) = (&daughter_id, disposition.as_deref()) {
        let previous: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
            .bind(daughter_id)
            .fetch_one(&mut *tx)
            .await?;
        if matches!(previous.as_str(), "active" | "weak" | "recovering") {
            sqlx::query(
                "UPDATE colonies SET status='inactive', updated_at=CURRENT_TIMESTAMP WHERE id=?",
            )
            .bind(daughter_id)
            .execute(&mut *tx)
            .await?;
            let lifecycle_id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO colony_lifecycle_records (id,colony_id,action,occurred_at,previous_status,new_status,reason,notes) VALUES (?,?,'deactivate',?,?,'inactive',?,'Gerado explicitamente durante a anulação de uma divisão sem consequências posteriores.')")
                .bind(lifecycle_id).bind(daughter_id).bind(&voided_at).bind(previous)
                .bind(format!("Anulação da divisão {id}: {reason}"))
                .execute(&mut *tx).await?;
        }
    }

    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Divisão não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "division",
        &id,
        "void",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn correct_occupancy(pool: &SqlitePool, input: CorrectOccupancy) -> Result<(), AppError> {
    let id = required(&input.id, "Ocupação")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let started_at = time::normalize(&input.started_at, false)?;
    let ended_at = time::normalize_optional(&input.ended_at, false)?;
    if ended_at
        .as_deref()
        .is_some_and(|end| end < started_at.as_str())
    {
        return Err(AppError::Validation(
            "O fim da ocupação não pode ser anterior ao início.".to_owned(),
        ));
    }
    let row: Option<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT colony_id, box_id, started_at, ended_at FROM colony_box_occupancies WHERE id=?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let (colony_id, box_id, old_start, old_end) =
        row.ok_or_else(|| AppError::NotFound("Ocupação não encontrada.".to_owned()))?;
    if old_end.is_none() != ended_at.is_none() {
        return Err(AppError::Validation(
            "A correção histórica não pode abrir ou encerrar uma ocupação. Use o fluxo operacional correspondente."
                .to_owned(),
        ));
    }

    let state_at: Option<String> = sqlx::query_scalar(
        "SELECT new_status FROM box_state_records WHERE box_id=? AND occurred_at<=?
         ORDER BY occurred_at DESC, created_at DESC, id DESC LIMIT 1",
    )
    .bind(&box_id)
    .bind(&started_at)
    .fetch_optional(pool)
    .await?;
    if state_at.as_deref().unwrap_or("active") != "active" {
        return Err(AppError::Validation(
            "A caixa não estava ativa no novo início da ocupação.".to_owned(),
        ));
    }

    let new_end_for_overlap = ended_at.as_deref().unwrap_or("9999-12-31 23:59:59");
    let colony_overlap: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM colony_box_occupancies
            WHERE id<>? AND colony_id=? AND started_at<=?
              AND COALESCE(ended_at,'9999-12-31 23:59:59')>=?)",
    )
    .bind(&id)
    .bind(&colony_id)
    .bind(new_end_for_overlap)
    .bind(&started_at)
    .fetch_one(pool)
    .await?;
    if colony_overlap {
        return Err(AppError::Validation(
            "A correção faria a colônia ocupar duas caixas no mesmo intervalo.".to_owned(),
        ));
    }
    let box_overlap: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM colony_box_occupancies
            WHERE id<>? AND box_id=? AND started_at<=?
              AND COALESCE(ended_at,'9999-12-31 23:59:59')>=?)",
    )
    .bind(&id)
    .bind(&box_id)
    .bind(new_end_for_overlap)
    .bind(&started_at)
    .fetch_one(pool)
    .await?;
    if box_overlap {
        return Err(AppError::Validation(
            "A correção faria a caixa abrigar duas colônias no mesmo intervalo.".to_owned(),
        ));
    }

    let fact_outside: bool = sqlx::query_scalar(
        "WITH facts(ts) AS (
            SELECT inspected_at FROM inspections WHERE colony_id=? AND box_id=?
            UNION ALL SELECT fed_at FROM feedings WHERE colony_id=? AND box_id=?
            UNION ALL SELECT harvested_at FROM production_records WHERE colony_id=? AND box_id=?
            UNION ALL SELECT occurred_at FROM colony_events WHERE colony_id=? AND box_id=?
            UNION ALL SELECT occurred_at FROM colony_lifecycle_records WHERE colony_id=? AND box_id=?
            UNION ALL SELECT maintained_at FROM box_maintenance_records WHERE colony_id=? AND box_id=?
            UNION ALL SELECT moved_at FROM colony_movements WHERE colony_id=? AND (from_box_id=? OR to_box_id=?)
         )
         SELECT EXISTS(
            SELECT 1 FROM facts
            WHERE ts>=? AND (? IS NULL OR ts<=?)
              AND (ts<? OR (? IS NOT NULL AND ts>?))
         )",
    )
    .bind(&colony_id).bind(&box_id).bind(&colony_id).bind(&box_id)
    .bind(&colony_id).bind(&box_id).bind(&colony_id).bind(&box_id)
    .bind(&colony_id).bind(&box_id).bind(&colony_id).bind(&box_id)
    .bind(&colony_id).bind(&box_id).bind(&box_id)
    .bind(&old_start).bind(&old_end).bind(&old_end)
    .bind(&started_at).bind(&ended_at).bind(&ended_at)
    .fetch_one(pool).await?;
    if fact_outside {
        return Err(AppError::Validation(
            "A correção removeria do intervalo fatos que já usam esta ocupação como contexto histórico. Corrija primeiro os fatos dependentes."
                .to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'started_at',started_at,'ended_at',ended_at,'reason',reason,'notes',notes,'corrected_at',corrected_at) FROM colony_box_occupancies WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Ocupação não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_box_occupancies SET started_at=?, ended_at=?, reason=?, notes=?, corrected_at=? WHERE id=?")
        .bind(&started_at).bind(&ended_at).bind(optional(&input.occupancy_reason)).bind(optional(&input.notes))
        .bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Ocupação não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "box_occupancy",
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
