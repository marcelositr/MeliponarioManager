use super::*;

pub async fn correct_inspection(
    pool: &SqlitePool,
    input: CorrectInspection,
) -> Result<(), AppError> {
    let id = required(&input.id, "Inspeção")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let strength = required(&input.strength, "Força")?;
    if !STRENGTHS.contains(&strength.as_str()) {
        return Err(AppError::Validation(
            "Força da colônia inválida.".to_owned(),
        ));
    }
    let inspected_at = time::normalize(&input.inspected_at, false)?;
    let next = time::normalize_optional(&input.next_inspection_at, false)?;
    time::ensure_not_before(
        &next,
        &inspected_at,
        "A próxima inspeção não pode ser anterior à inspeção registrada.",
    )?;

    let colony_id: Option<String> =
        sqlx::query_scalar("SELECT colony_id FROM inspections WHERE id = ? AND voided_at IS NULL")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let colony_id = colony_id
        .ok_or_else(|| AppError::Validation("Inspeção não encontrada ou já anulada.".to_owned()))?;
    operational::ensure_colony_available_at(pool, &colony_id, &inspected_at).await?;
    let box_id = historical_box_for_colony(pool, &colony_id, &inspected_at).await?;

    let mut tx = pool.begin().await?;
    let before = snapshot_tx(&mut tx,
        "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'inspected_at',inspected_at,'strength',strength,'queen_present',queen_present,'laying_status',laying_status,'food_reserves',food_reserves,'brood_status',brood_status,'pests_notes',pests_notes,'observations',observations,'actions_taken',actions_taken,'next_inspection_at',next_inspection_at,'voided_at',voided_at) FROM inspections WHERE id = ?",
        &id, "Inspeção não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query(
        "UPDATE inspections SET box_id=?, inspected_at=?, strength=?, queen_present=?,
            laying_status=?, food_reserves=?, brood_status=?, pests_notes=?, observations=?,
            actions_taken=?, next_inspection_at=?, corrected_at=? WHERE id=? AND voided_at IS NULL",
    )
    .bind(box_id)
    .bind(&inspected_at)
    .bind(strength)
    .bind(input.queen_present)
    .bind(optional(&input.laying_status))
    .bind(optional(&input.food_reserves))
    .bind(optional(&input.brood_status))
    .bind(optional(&input.pests_notes))
    .bind(optional(&input.observations))
    .bind(optional(&input.actions_taken))
    .bind(next)
    .bind(corrected_at)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = snapshot_tx(&mut tx,
        "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'inspected_at',inspected_at,'strength',strength,'queen_present',queen_present,'laying_status',laying_status,'food_reserves',food_reserves,'brood_status',brood_status,'pests_notes',pests_notes,'observations',observations,'actions_taken',actions_taken,'next_inspection_at',next_inspection_at,'voided_at',voided_at) FROM inspections WHERE id = ?",
        &id, "Inspeção não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "inspection",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    agenda::reconcile_inspection_tx(&mut tx, &colony_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_inspection(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "inspection", "inspections",
        "SELECT json_object('id',id,'colony_id',colony_id,'inspected_at',inspected_at,'strength',strength,'next_inspection_at',next_inspection_at,'voided_at',voided_at,'void_reason',void_reason) FROM inspections WHERE id = ?")
        .await
}

pub async fn correct_feeding(pool: &SqlitePool, input: CorrectFeeding) -> Result<(), AppError> {
    let id = required(&input.id, "Alimentação")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let food_type = required(&input.food_type, "Tipo de alimento")?;
    let fed_at = time::normalize(&input.fed_at, false)?;
    let next = time::normalize_optional(&input.next_feeding_at, false)?;
    time::ensure_not_before(
        &next,
        &fed_at,
        "A próxima alimentação não pode ser anterior à alimentação registrada.",
    )?;
    let unit = optional(&input.unit);
    match (input.quantity, unit.as_ref()) {
        (None, None) => {}
        (Some(value), Some(_)) if value.is_finite() && value > 0.0 => {}
        (Some(_), None) => {
            return Err(AppError::Validation(
                "Informe a unidade quando houver quantidade.".to_owned(),
            ))
        }
        (None, Some(_)) => {
            return Err(AppError::Validation(
                "Informe a quantidade quando houver unidade.".to_owned(),
            ))
        }
        _ => {
            return Err(AppError::Validation(
                "A quantidade precisa ser maior que zero.".to_owned(),
            ))
        }
    }
    let colony_id: Option<String> =
        sqlx::query_scalar("SELECT colony_id FROM feedings WHERE id=? AND voided_at IS NULL")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let colony_id = colony_id.ok_or_else(|| {
        AppError::Validation("Alimentação não encontrada ou já anulada.".to_owned())
    })?;
    operational::ensure_colony_available_at(pool, &colony_id, &fed_at).await?;
    let box_id = historical_box_for_colony(pool, &colony_id, &fed_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'fed_at',fed_at,'food_type',food_type,'quantity',quantity,'unit',unit,'response_notes',response_notes,'notes',notes,'next_feeding_at',next_feeding_at,'voided_at',voided_at) FROM feedings WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Alimentação não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE feedings SET box_id=?, fed_at=?, food_type=?, quantity=?, unit=?, response_notes=?, notes=?, next_feeding_at=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(box_id).bind(&fed_at).bind(food_type).bind(input.quantity).bind(unit)
        .bind(optional(&input.response_notes)).bind(optional(&input.notes)).bind(next)
        .bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Alimentação não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "feeding",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    agenda::reconcile_feeding_tx(&mut tx, &colony_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn void_feeding(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "feeding", "feedings",
        "SELECT json_object('id',id,'colony_id',colony_id,'fed_at',fed_at,'food_type',food_type,'next_feeding_at',next_feeding_at,'voided_at',voided_at,'void_reason',void_reason) FROM feedings WHERE id=?")
        .await
}

pub async fn correct_production(
    pool: &SqlitePool,
    input: CorrectProduction,
) -> Result<(), AppError> {
    let id = required(&input.id, "Produção")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let product_type = required(&input.product_type, "Produto")?;
    if !PRODUCT_TYPES.contains(&product_type.as_str()) {
        return Err(AppError::Validation("Tipo de produto inválido.".to_owned()));
    }
    if !input.quantity.is_finite() || input.quantity <= 0.0 {
        return Err(AppError::Validation(
            "A quantidade precisa ser maior que zero.".to_owned(),
        ));
    }
    let unit = required(&input.unit, "Unidade")?;
    let harvested_at = time::normalize(&input.harvested_at, false)?;
    let colony_id: Option<String> = sqlx::query_scalar(
        "SELECT colony_id FROM production_records WHERE id=? AND voided_at IS NULL",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?;
    let colony_id = colony_id
        .ok_or_else(|| AppError::Validation("Produção não encontrada ou já anulada.".to_owned()))?;
    operational::ensure_colony_available_at(pool, &colony_id, &harvested_at).await?;
    let box_id = historical_box_for_colony(pool, &colony_id, &harvested_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'harvested_at',harvested_at,'product_type',product_type,'quantity',quantity,'unit',unit,'purpose',purpose,'notes',notes,'voided_at',voided_at) FROM production_records WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Produção não encontrada.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE production_records SET box_id=?, harvested_at=?, product_type=?, quantity=?, unit=?, purpose=?, notes=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(box_id).bind(&harvested_at).bind(product_type).bind(input.quantity).bind(unit)
        .bind(optional(&input.purpose)).bind(optional(&input.notes)).bind(corrected_at).bind(&id)
        .execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Produção não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "production",
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

pub async fn void_production(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "production", "production_records",
        "SELECT json_object('id',id,'colony_id',colony_id,'harvested_at',harvested_at,'product_type',product_type,'quantity',quantity,'unit',unit,'voided_at',voided_at,'void_reason',void_reason) FROM production_records WHERE id=?")
        .await
}

pub async fn correct_maintenance(
    pool: &SqlitePool,
    input: CorrectMaintenance,
) -> Result<(), AppError> {
    let id = required(&input.id, "Manutenção")?;
    let box_id = required(&input.box_id, "Caixa")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    let maintenance_type = required(&input.maintenance_type, "Tipo de manutenção")?;
    if !MAINTENANCE_TYPES.contains(&maintenance_type.as_str()) {
        return Err(AppError::Validation(
            "Tipo de manutenção inválido.".to_owned(),
        ));
    }
    if input
        .cost
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(AppError::Validation(
            "O custo precisa ser válido e não negativo.".to_owned(),
        ));
    }
    let maintained_at = time::normalize(&input.maintained_at, false)?;
    let next = time::normalize_optional(&input.next_maintenance_at, false)?;
    time::ensure_not_before(
        &next,
        &maintained_at,
        "A próxima manutenção não pode ser anterior à manutenção registrada.",
    )?;
    let box_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM boxes WHERE id=?)")
        .bind(&box_id)
        .fetch_one(pool)
        .await?;
    if !box_exists {
        return Err(AppError::NotFound("Caixa não encontrada.".to_owned()));
    }
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM box_maintenance_records WHERE id=? AND voided_at IS NULL)",
    )
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Err(AppError::Validation(
            "Manutenção não encontrada ou já anulada.".to_owned(),
        ));
    }
    let colony_id = colony_in_box_at(pool, &box_id, &maintained_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'box_id',box_id,'colony_id',colony_id,'maintained_at',maintained_at,'maintenance_type',maintenance_type,'description',description,'performed_by',performed_by,'cost',cost,'next_maintenance_at',next_maintenance_at,'voided_at',voided_at) FROM box_maintenance_records WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Manutenção não encontrada.").await?;
    let old_box_id: String = sqlx::query_scalar(
        "SELECT box_id FROM box_maintenance_records WHERE id=? AND voided_at IS NULL",
    )
    .bind(&id)
    .fetch_one(&mut *tx)
    .await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE box_maintenance_records SET box_id=?, colony_id=?, maintained_at=?, maintenance_type=?, description=?, performed_by=?, cost=?, next_maintenance_at=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(&box_id).bind(colony_id).bind(&maintained_at).bind(maintenance_type)
        .bind(optional(&input.description)).bind(optional(&input.performed_by)).bind(input.cost)
        .bind(next).bind(corrected_at).bind(&id).execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Manutenção não encontrada.").await?;
    audit::record_tx(
        &mut tx,
        "box_maintenance",
        &id,
        "correct",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    agenda::reconcile_maintenance_tx(&mut tx, &old_box_id).await?;
    if box_id != old_box_id {
        agenda::reconcile_maintenance_tx(&mut tx, &box_id).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn void_maintenance(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "box_maintenance", "box_maintenance_records",
        "SELECT json_object('id',id,'box_id',box_id,'colony_id',colony_id,'maintained_at',maintained_at,'maintenance_type',maintenance_type,'next_maintenance_at',next_maintenance_at,'voided_at',voided_at,'void_reason',void_reason) FROM box_maintenance_records WHERE id=?")
        .await
}

pub async fn correct_event(pool: &SqlitePool, input: CorrectEvent) -> Result<(), AppError> {
    let id = required(&input.id, "Evento")?;
    let event_type = required(&input.event_type, "Tipo do evento")?;
    let severity = required(&input.severity, "Importância")?;
    let reason = required(&input.reason, "Motivo da correção")?;
    if !EVENT_TYPES.contains(&event_type.as_str()) {
        return Err(AppError::Validation("Tipo de evento inválido.".to_owned()));
    }
    if !SEVERITIES.contains(&severity.as_str()) {
        return Err(AppError::Validation("Nível do evento inválido.".to_owned()));
    }
    let occurred_at = time::normalize(&input.occurred_at, false)?;
    let now = time::local_now(pool).await?;
    if occurred_at > now {
        return Err(AppError::Validation("Eventos manuais representam fatos ocorridos e não podem ser registrados no futuro. Use a Agenda em etapa própria.".to_owned()));
    }
    let colony_id: Option<String> =
        sqlx::query_scalar("SELECT colony_id FROM colony_events WHERE id=? AND voided_at IS NULL")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let colony_id = colony_id
        .ok_or_else(|| AppError::Validation("Evento não encontrado ou já anulado.".to_owned()))?;
    let box_id = historical_box_for_colony(pool, &colony_id, &occurred_at).await?;
    let mut tx = pool.begin().await?;
    let snapshot_sql = "SELECT json_object('id',id,'colony_id',colony_id,'box_id',box_id,'event_type',event_type,'occurred_at',occurred_at,'title',title,'details',details,'severity',severity,'voided_at',voided_at) FROM colony_events WHERE id=?";
    let before = snapshot_tx(&mut tx, snapshot_sql, &id, "Evento não encontrado.").await?;
    let corrected_at = now_tx(&mut tx).await?;
    sqlx::query("UPDATE colony_events SET box_id=?, event_type=?, occurred_at=?, title=?, details=?, severity=?, corrected_at=? WHERE id=? AND voided_at IS NULL")
        .bind(box_id).bind(event_type).bind(&occurred_at).bind(optional(&input.title))
        .bind(optional(&input.details)).bind(severity).bind(corrected_at).bind(&id)
        .execute(&mut *tx).await?;
    let after = snapshot_tx(&mut tx, snapshot_sql, &id, "Evento não encontrado.").await?;
    audit::record_tx(
        &mut tx,
        "colony_event",
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

pub async fn void_event(pool: &SqlitePool, input: VoidRecord) -> Result<(), AppError> {
    void_fact(pool, input, "colony_event", "colony_events",
        "SELECT json_object('id',id,'colony_id',colony_id,'event_type',event_type,'occurred_at',occurred_at,'title',title,'severity',severity,'voided_at',voided_at,'void_reason',void_reason) FROM colony_events WHERE id=?")
        .await
}
