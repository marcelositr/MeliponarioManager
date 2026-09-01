use super::*;

async fn validate_manual_context(
    pool: &SqlitePool,
    meliponary_id: &str,
    colony_id: &Option<String>,
    box_id: &Option<String>,
    task_type: &str,
) -> Result<(), AppError> {
    let meliponary: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM meliponaries WHERE id = ?")
            .bind(meliponary_id)
            .fetch_optional(pool)
            .await?;
    let archived = meliponary
        .ok_or_else(|| AppError::NotFound("Meliponário não encontrado.".to_owned()))?
        .is_some();

    let generic_admin = task_type == "generic" && colony_id.is_none() && box_id.is_none();
    if archived && !generic_admin {
        return Err(AppError::Validation(
            "Não é possível criar compromisso operacional em meliponário arquivado.".to_owned(),
        ));
    }

    if let Some(colony_id) = colony_id {
        let row: Option<(String, String)> =
            sqlx::query_as("SELECT meliponary_id, status FROM colonies WHERE id = ?")
                .bind(colony_id)
                .fetch_optional(pool)
                .await?;
        let (colony_meliponary, status) =
            row.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;
        if colony_meliponary != meliponary_id {
            return Err(AppError::Validation(
                "A colônia não pertence ao meliponário informado.".to_owned(),
            ));
        }
        if task_type != "generic" && !matches!(status.as_str(), "active" | "weak" | "recovering") {
            return Err(AppError::Validation(
                "A colônia não está disponível para novo compromisso operacional.".to_owned(),
            ));
        }
    }

    if let Some(box_id) = box_id {
        let row: Option<(String, String)> =
            sqlx::query_as("SELECT meliponary_id, status FROM boxes WHERE id = ?")
                .bind(box_id)
                .fetch_optional(pool)
                .await?;
        let (box_meliponary, status) =
            row.ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))?;
        if box_meliponary != meliponary_id {
            return Err(AppError::Validation(
                "A caixa não pertence ao meliponário informado.".to_owned(),
            ));
        }
        if task_type != "generic" && status == "retired" {
            return Err(AppError::Validation(
                "Não é possível criar compromisso operacional para caixa aposentada.".to_owned(),
            ));
        }
    }

    match task_type {
        "inspection" | "feeding" if colony_id.is_none() => Err(AppError::Validation(
            "Inspeção e alimentação precisam estar ligadas a uma colônia.".to_owned(),
        )),
        "maintenance" if box_id.is_none() => Err(AppError::Validation(
            "Manutenção precisa estar ligada a uma caixa.".to_owned(),
        )),
        _ => Ok(()),
    }
}

async fn validate_manual_date(pool: &SqlitePool, scheduled_for: &str) -> Result<(), AppError> {
    let oldest_allowed: String =
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime', '-7 days')")
            .fetch_one(pool)
            .await?;
    if scheduled_for < oldest_allowed.as_str() {
        return Err(AppError::Validation(
            "Uma tarefa pendente não pode ser criada mais de 7 dias no passado. Para fatos antigos, use o registro histórico correspondente."
                .to_owned(),
        ));
    }
    Ok(())
}

pub async fn create_manual(
    pool: &SqlitePool,
    input: CreateTask,
) -> Result<ScheduledTask, AppError> {
    let meliponary_id = required(&input.meliponary_id, "Meliponário")?;
    let task_type = required(&input.task_type, "Tipo")?;
    valid_task_type(&task_type)?;
    let title = required(&input.title, "Título")?;
    let scheduled_for = time::normalize(&input.scheduled_for, false)?;
    validate_manual_date(pool, &scheduled_for).await?;
    let priority = optional(&input.priority).unwrap_or_else(|| "normal".to_owned());
    valid_priority(&priority)?;
    let colony_id = optional(&input.colony_id);
    let box_id = optional(&input.box_id);
    validate_manual_context(pool, &meliponary_id, &colony_id, &box_id, &task_type).await?;

    let id = Uuid::new_v4().to_string();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO scheduled_tasks(
           id,meliponary_id,colony_id,box_id,task_type,title,description,scheduled_for,priority
         ) VALUES(?,?,?,?,?,?,?,?,?)",
    )
    .bind(&id)
    .bind(&meliponary_id)
    .bind(&colony_id)
    .bind(&box_id)
    .bind(&task_type)
    .bind(&title)
    .bind(optional(&input.description))
    .bind(&scheduled_for)
    .bind(&priority)
    .execute(&mut *tx)
    .await?;
    let after = task_snapshot_tx(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "scheduled_task",
        &id,
        "create",
        "Criação manual de tarefa na Agenda.",
        None,
        Some(after),
    )
    .await?;
    tx.commit().await?;
    get(pool, &id).await
}

pub async fn reschedule(
    pool: &SqlitePool,
    input: RescheduleTask,
) -> Result<ScheduledTask, AppError> {
    let id = required(&input.id, "Tarefa")?;
    let scheduled_for = time::normalize(&input.scheduled_for, false)?;
    validate_manual_date(pool, &scheduled_for).await?;
    let reason = optional(&input.reason).unwrap_or_else(|| "Reagendamento da Agenda.".to_owned());
    let mut tx = pool.begin().await?;
    let before = task_snapshot_tx(&mut tx, &id).await?;
    type RescheduleRow = (
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
    );
    let current: Option<RescheduleRow> = sqlx::query_as(
        "SELECT meliponary_id,task_type,colony_id,box_id,title,priority,description,status,source_type,source_id
         FROM scheduled_tasks WHERE id=?",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await?;
    let (
        meliponary_id,
        task_type,
        colony_id,
        box_id,
        title,
        priority,
        description,
        status,
        source_type,
        source_id,
    ) = current.ok_or_else(|| AppError::NotFound("Tarefa não encontrada.".to_owned()))?;
    if status != "pending" {
        return Err(AppError::Validation(
            "Somente tarefa pendente pode ser reagendada.".to_owned(),
        ));
    }
    let now = now_tx(&mut tx).await?;
    sqlx::query(
        "UPDATE scheduled_tasks SET status='rescheduled',reschedule_reason=?,updated_at=? WHERE id=?",
    )
    .bind(&reason)
    .bind(&now)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let replacement_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO scheduled_tasks(
           id,meliponary_id,colony_id,box_id,task_type,title,description,scheduled_for,
           priority,source_type,source_id,rescheduled_from_id
         ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&replacement_id)
    .bind(meliponary_id)
    .bind(colony_id)
    .bind(box_id)
    .bind(task_type)
    .bind(title)
    .bind(description)
    .bind(&scheduled_for)
    .bind(priority)
    .bind(source_type)
    .bind(source_id)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = task_snapshot_tx(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "scheduled_task",
        &id,
        "reschedule",
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    get(pool, &replacement_id).await
}

pub async fn cancel(pool: &SqlitePool, input: TaskReason) -> Result<ScheduledTask, AppError> {
    transition_with_reason(pool, input, "cancelled").await
}

pub async fn skip(pool: &SqlitePool, input: TaskReason) -> Result<ScheduledTask, AppError> {
    transition_with_reason(pool, input, "skipped").await
}

async fn transition_with_reason(
    pool: &SqlitePool,
    input: TaskReason,
    target: &'static str,
) -> Result<ScheduledTask, AppError> {
    let id = required(&input.id, "Tarefa")?;
    let reason = required(&input.reason, "Motivo")?;
    let mut tx = pool.begin().await?;
    let before = task_snapshot_tx(&mut tx, &id).await?;
    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_tasks WHERE id=?")
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;
    if status != "pending" {
        return Err(AppError::Validation(
            "Somente tarefa pendente pode receber esta ação.".to_owned(),
        ));
    }
    let now = now_tx(&mut tx).await?;
    if target == "cancelled" {
        sqlx::query(
            "UPDATE scheduled_tasks SET status='cancelled',cancelled_at=?,cancellation_reason=?,updated_at=? WHERE id=?",
        )
        .bind(&now)
        .bind(&reason)
        .bind(&now)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            "UPDATE scheduled_tasks SET status='skipped',skipped_at=?,skip_reason=?,updated_at=? WHERE id=?",
        )
        .bind(&now)
        .bind(&reason)
        .bind(&now)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    }
    let after = task_snapshot_tx(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "scheduled_task",
        &id,
        target,
        &reason,
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    get(pool, &id).await
}

pub async fn complete_generic(pool: &SqlitePool, id: &str) -> Result<ScheduledTask, AppError> {
    let id = required(id, "Tarefa")?;
    let mut tx = pool.begin().await?;
    let before = task_snapshot_tx(&mut tx, &id).await?;
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT task_type,status FROM scheduled_tasks WHERE id=?")
            .bind(&id)
            .fetch_optional(&mut *tx)
            .await?;
    let (task_type, status) =
        row.ok_or_else(|| AppError::NotFound("Tarefa não encontrada.".to_owned()))?;
    if task_type != "generic" || status != "pending" {
        return Err(AppError::Validation(
            "Somente tarefa genérica pendente pode ser concluída manualmente.".to_owned(),
        ));
    }
    let now = now_tx(&mut tx).await?;
    sqlx::query(
        "UPDATE scheduled_tasks SET status='completed',completed_at=?,updated_at=? WHERE id=?",
    )
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = task_snapshot_tx(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "scheduled_task",
        &id,
        "complete",
        "Conclusão manual de tarefa genérica.",
        Some(before),
        Some(after),
    )
    .await?;
    tx.commit().await?;
    get(pool, &id).await
}

pub async fn duplicate(pool: &SqlitePool, input: DuplicateTask) -> Result<ScheduledTask, AppError> {
    let id = required(&input.id, "Tarefa")?;
    let original = get(pool, &id).await?;
    create_manual(
        pool,
        CreateTask {
            meliponary_id: original.meliponary_id,
            colony_id: original.colony_id,
            box_id: original.box_id,
            task_type: original.task_type,
            title: original.title,
            description: original.description,
            scheduled_for: input.scheduled_for,
            priority: Some(original.priority),
        },
    )
    .await
}
