use super::*;

async fn get_with_time(pool: &SqlitePool, id: &str, now: &str) -> Result<ScheduledTask, AppError> {
    let today = &now[..10];
    Ok(sqlx::query_as::<_, ScheduledTask>(
        "SELECT t.id, t.meliponary_id, m.name AS meliponary_name,
                t.colony_id, c.code AS colony_code, t.box_id, b.code AS box_code,
                t.task_type, t.title, t.description, t.scheduled_for, t.status,
                t.priority, t.source_type, t.source_id, t.completed_at,
                t.completed_by_type, t.completed_by_id, t.cancelled_at,
                t.cancellation_reason, t.skipped_at, t.skip_reason,
                t.rescheduled_from_id, t.reschedule_reason, t.created_at, t.updated_at,
                CASE WHEN t.status='pending' AND t.scheduled_for < ? THEN 1 ELSE 0 END AS overdue,
                CASE WHEN t.status='pending' AND substr(t.scheduled_for,1,10) = ? THEN 1 ELSE 0 END AS today
         FROM scheduled_tasks t
         JOIN meliponaries m ON m.id=t.meliponary_id
         LEFT JOIN colonies c ON c.id=t.colony_id
         LEFT JOIN boxes b ON b.id=t.box_id
         WHERE t.id=?",
    )
    .bind(now)
    .bind(today)
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<ScheduledTask, AppError> {
    let id = required(id, "Tarefa")?;
    let now = time::local_now(pool).await?;
    get_with_time(pool, &id, &now).await
}

pub async fn list(pool: &SqlitePool, query: TaskQuery) -> Result<Vec<ScheduledTask>, AppError> {
    let view = optional(&query.view).unwrap_or_else(|| "pending".to_owned());
    if !VIEWS.contains(&view.as_str()) {
        return Err(AppError::Validation(
            "Visualização da Agenda inválida.".to_owned(),
        ));
    }
    if let Some(value) = query.task_type.as_deref() {
        valid_task_type(value)?;
    }
    if let Some(value) = query.priority.as_deref() {
        valid_priority(value)?;
    }

    let now = time::local_now(pool).await?;
    let today = now[..10].to_owned();
    let tomorrow: String = sqlx::query_scalar("SELECT strftime('%Y-%m-%d 00:00:00', ?, '+1 day')")
        .bind(&now)
        .fetch_one(pool)
        .await?;
    let seven_days: String =
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', ?, '+7 days')")
            .bind(&now)
            .fetch_one(pool)
            .await?;
    let search = optional(&query.search).map(|value| format!("%{value}%"));

    Ok(sqlx::query_as::<_, ScheduledTask>(
        "SELECT t.id, t.meliponary_id, m.name AS meliponary_name,
                t.colony_id, c.code AS colony_code, t.box_id, b.code AS box_code,
                t.task_type, t.title, t.description, t.scheduled_for, t.status,
                t.priority, t.source_type, t.source_id, t.completed_at,
                t.completed_by_type, t.completed_by_id, t.cancelled_at,
                t.cancellation_reason, t.skipped_at, t.skip_reason,
                t.rescheduled_from_id, t.reschedule_reason, t.created_at, t.updated_at,
                CASE WHEN t.status='pending' AND t.scheduled_for < ? THEN 1 ELSE 0 END AS overdue,
                CASE WHEN t.status='pending' AND substr(t.scheduled_for,1,10) = ? THEN 1 ELSE 0 END AS today
         FROM scheduled_tasks t
         JOIN meliponaries m ON m.id=t.meliponary_id
         LEFT JOIN colonies c ON c.id=t.colony_id
         LEFT JOIN boxes b ON b.id=t.box_id
         WHERE
           (? IS NULL OR t.meliponary_id = ?)
           AND (? IS NULL OR t.colony_id = ?)
           AND (? IS NULL OR t.box_id = ?)
           AND (? IS NULL OR t.task_type = ?)
           AND (? IS NULL OR t.priority = ?)
           AND (? IS NULL OR t.title LIKE ? OR c.code LIKE ? OR b.code LIKE ?)
           AND (
             ? = 'all'
             OR (? = 'pending' AND t.status='pending')
             OR (? = 'overdue' AND t.status='pending' AND t.scheduled_for < ?)
             OR (? = 'today' AND t.status='pending' AND substr(t.scheduled_for,1,10) = ?)
             OR (? = 'upcoming' AND t.status='pending' AND t.scheduled_for >= ? AND t.scheduled_for <= ?)
             OR (? = 'completed' AND t.status='completed')
             OR (? = 'cancelled' AND t.status='cancelled')
             OR (? = 'rescheduled' AND t.status='rescheduled')
             OR (? = 'skipped' AND t.status='skipped')
           )
         ORDER BY
           CASE WHEN t.status='pending' AND t.scheduled_for < ? THEN 0
                WHEN t.status='pending' AND substr(t.scheduled_for,1,10)=? THEN 1
                WHEN t.status='pending' THEN 2 ELSE 3 END,
           t.scheduled_for, t.created_at, t.id",
    )
    .bind(&now)
    .bind(&today)
    .bind(&query.meliponary_id)
    .bind(&query.meliponary_id)
    .bind(&query.colony_id)
    .bind(&query.colony_id)
    .bind(&query.box_id)
    .bind(&query.box_id)
    .bind(&query.task_type)
    .bind(&query.task_type)
    .bind(&query.priority)
    .bind(&query.priority)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .bind(&view)
    .bind(&view)
    .bind(&view)
    .bind(&now)
    .bind(&view)
    .bind(&today)
    .bind(&view)
    .bind(&tomorrow)
    .bind(&seven_days)
    .bind(&view)
    .bind(&view)
    .bind(&view)
    .bind(&view)
    .bind(&now)
    .bind(&today)
    .fetch_all(pool)
    .await?)
}

pub(super) async fn summary_at(
    pool: &SqlitePool,
    meliponary_id: Option<&str>,
    now: &str,
) -> Result<AgendaSummary, AppError> {
    let tomorrow: String = sqlx::query_scalar("SELECT strftime('%Y-%m-%d 00:00:00', ?, '+1 day')")
        .bind(now)
        .fetch_one(pool)
        .await?;
    let horizon: String = sqlx::query_scalar("SELECT strftime('%Y-%m-%d 00:00:00', ?, '+8 days')")
        .bind(now)
        .fetch_one(pool)
        .await?;
    Ok(sqlx::query_as::<_, AgendaSummary>(
        "SELECT
           COALESCE(SUM(CASE WHEN status='pending' AND scheduled_for < ? THEN 1 ELSE 0 END),0) overdue,
           COALESCE(SUM(CASE WHEN status='pending' AND scheduled_for >= ? AND scheduled_for < ? THEN 1 ELSE 0 END),0) today,
           COALESCE(SUM(CASE WHEN status='pending' AND scheduled_for >= ? AND scheduled_for < ? THEN 1 ELSE 0 END),0) next_seven_days,
           COALESCE(SUM(CASE WHEN status='pending' AND scheduled_for >= ? THEN 1 ELSE 0 END),0) future
         FROM scheduled_tasks
         WHERE (? IS NULL OR meliponary_id=?)",
    )
    .bind(now)
    .bind(now)
    .bind(&tomorrow)
    .bind(&tomorrow)
    .bind(&horizon)
    .bind(&horizon)
    .bind(meliponary_id)
    .bind(meliponary_id)
    .fetch_one(pool)
    .await?)
}

pub async fn summary(
    pool: &SqlitePool,
    meliponary_id: Option<&str>,
) -> Result<AgendaSummary, AppError> {
    let now = time::local_now(pool).await?;
    summary_at(pool, meliponary_id, &now).await
}
