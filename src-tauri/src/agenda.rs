use crate::{audit, operational, repository::AppError, time};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

const TASK_TYPES: &[&str] = &["inspection", "feeding", "maintenance", "generic"];
const PRIORITIES: &[&str] = &["normal", "attention", "critical"];
const VIEWS: &[&str] = &[
    "all",
    "pending",
    "overdue",
    "today",
    "upcoming",
    "completed",
    "cancelled",
    "rescheduled",
    "skipped",
];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: String,
    pub meliponary_id: String,
    pub meliponary_name: String,
    pub colony_id: Option<String>,
    pub colony_code: Option<String>,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub task_type: String,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_for: String,
    pub status: String,
    pub priority: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub completed_at: Option<String>,
    pub completed_by_type: Option<String>,
    pub completed_by_id: Option<String>,
    pub cancelled_at: Option<String>,
    pub cancellation_reason: Option<String>,
    pub skipped_at: Option<String>,
    pub skip_reason: Option<String>,
    pub rescheduled_from_id: Option<String>,
    pub reschedule_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub overdue: bool,
    pub today: bool,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgendaSummary {
    pub overdue: i64,
    pub today: i64,
    pub next_seven_days: i64,
    pub future: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTask {
    pub meliponary_id: String,
    pub colony_id: Option<String>,
    pub box_id: Option<String>,
    pub task_type: String,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_for: String,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskQuery {
    pub view: Option<String>,
    pub meliponary_id: Option<String>,
    pub colony_id: Option<String>,
    pub box_id: Option<String>,
    pub task_type: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RescheduleTask {
    pub id: String,
    pub scheduled_for: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReason {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateTask {
    pub id: String,
    pub scheduled_for: String,
}

#[derive(Debug, Clone)]
struct DerivedTask {
    source_type: &'static str,
    source_id: String,
    meliponary_id: String,
    colony_id: Option<String>,
    box_id: Option<String>,
    task_type: &'static str,
    title: String,
    scheduled_for: String,
}

#[derive(Debug, Clone, FromRow)]
struct PendingDerived {
    id: String,
    source_id: String,
    scheduled_for: String,
    source_baseline: String,
}

fn required(value: &str, field: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} é obrigatório.")));
    }
    Ok(value.to_owned())
}

fn optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn valid_task_type(value: &str) -> Result<(), AppError> {
    if TASK_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation("Tipo de tarefa inválido.".to_owned()))
    }
}

fn valid_priority(value: &str) -> Result<(), AppError> {
    if PRIORITIES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation("Prioridade inválida.".to_owned()))
    }
}

async fn now_tx(tx: &mut Transaction<'_, Sqlite>) -> Result<String, AppError> {
    Ok(
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')")
            .fetch_one(&mut **tx)
            .await?,
    )
}

async fn task_snapshot_tx(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<Value, AppError> {
    let raw: Option<String> = sqlx::query_scalar(
        "SELECT json_object(
            'id', id, 'meliponary_id', meliponary_id, 'colony_id', colony_id,
            'box_id', box_id, 'task_type', task_type, 'title', title,
            'description', description, 'scheduled_for', scheduled_for,
            'status', status, 'priority', priority, 'source_type', source_type,
            'source_id', source_id, 'completed_at', completed_at,
            'completed_by_type', completed_by_type, 'completed_by_id', completed_by_id,
            'cancelled_at', cancelled_at, 'cancellation_reason', cancellation_reason,
            'skipped_at', skipped_at, 'skip_reason', skip_reason,
            'rescheduled_from_id', rescheduled_from_id, 'reschedule_reason', reschedule_reason
         ) FROM scheduled_tasks WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    let raw = raw.ok_or_else(|| AppError::NotFound("Tarefa não encontrada.".to_owned()))?;
    serde_json::from_str(&raw).map_err(|error| {
        AppError::Validation(format!(
            "Não foi possível preparar a auditoria da Agenda: {error}"
        ))
    })
}

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

async fn summary_at(
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

async fn reconcile_derived(
    pool: &SqlitePool,
    colony_scope: Option<&str>,
    box_scope: Option<&str>,
    task_type: &'static str,
    desired: Option<DerivedTask>,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let pending = sqlx::query_as::<_, PendingDerived>(
        "WITH RECURSIVE lineage(id,root_scheduled_for) AS (
            SELECT id,scheduled_for FROM scheduled_tasks WHERE rescheduled_from_id IS NULL
            UNION ALL
            SELECT child.id,parent.root_scheduled_for
            FROM scheduled_tasks child
            JOIN lineage parent ON child.rescheduled_from_id=parent.id
         )
         SELECT t.id,t.source_id,t.scheduled_for,
                COALESCE(lineage.root_scheduled_for,t.scheduled_for) source_baseline
         FROM scheduled_tasks t
         LEFT JOIN lineage ON lineage.id=t.id
         WHERE t.status='pending' AND t.task_type=? AND t.source_type=?
           AND ((? IS NOT NULL AND t.colony_id=?) OR (? IS NOT NULL AND t.box_id=?))
         ORDER BY t.created_at,t.id",
    )
    .bind(task_type)
    .bind(task_type)
    .bind(colony_scope)
    .bind(colony_scope)
    .bind(box_scope)
    .bind(box_scope)
    .fetch_all(&mut *tx)
    .await?;
    let now = now_tx(&mut tx).await?;
    let mut kept = false;

    for current in pending {
        match desired.as_ref() {
            Some(next)
                if current.source_id == next.source_id
                    && current.source_baseline == next.scheduled_for
                    && !kept =>
            {
                sqlx::query(
                    "UPDATE scheduled_tasks SET meliponary_id=?,colony_id=?,box_id=?,title=?,updated_at=? WHERE id=?",
                )
                .bind(&next.meliponary_id)
                .bind(&next.colony_id)
                .bind(&next.box_id)
                .bind(&next.title)
                .bind(&now)
                .bind(&current.id)
                .execute(&mut *tx)
                .await?;
                kept = true;
            }
            Some(next) if current.source_id == next.source_id && !kept => {
                sqlx::query(
                    "UPDATE scheduled_tasks SET status='rescheduled',reschedule_reason='Data futura alterada no fato de origem.',updated_at=? WHERE id=?",
                )
                .bind(&now)
                .bind(&current.id)
                .execute(&mut *tx)
                .await?;
            }
            _ => {
                sqlx::query(
                    "UPDATE scheduled_tasks SET status='cancelled',cancelled_at=?,cancellation_reason='Compromisso substituído ou invalidado pelo fato de origem.',updated_at=? WHERE id=?",
                )
                .bind(&now)
                .bind(&now)
                .bind(&current.id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    if let Some(next) = desired {
        if !kept {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO scheduled_tasks(
                   id,meliponary_id,colony_id,box_id,task_type,title,scheduled_for,
                   priority,source_type,source_id
                 ) VALUES(?,?,?,?,?,?,?,'normal',?,?)",
            )
            .bind(id)
            .bind(next.meliponary_id)
            .bind(next.colony_id)
            .bind(next.box_id)
            .bind(next.task_type)
            .bind(next.title)
            .bind(next.scheduled_for)
            .bind(next.source_type)
            .bind(next.source_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

pub async fn reconcile_inspection(pool: &SqlitePool, colony_id: &str) -> Result<(), AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    operational::ensure_colony_exists(pool, &colony_id).await?;
    type Latest = (
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<String>,
        String,
    );
    let latest: Option<Latest> = sqlx::query_as(
        "SELECT i.id,i.next_inspection_at,i.box_id,c.meliponary_id,c.code,m.archived_at,c.status
         FROM inspections i
         JOIN colonies c ON c.id=i.colony_id
         JOIN meliponaries m ON m.id=c.meliponary_id
         WHERE i.colony_id=? AND i.voided_at IS NULL
         ORDER BY i.inspected_at DESC,i.created_at DESC,i.id DESC LIMIT 1",
    )
    .bind(&colony_id)
    .fetch_optional(pool)
    .await?;
    let desired = latest.and_then(
        |(source_id, next, box_id, meliponary_id, code, archived_at, status)| {
            if archived_at.is_some() || !matches!(status.as_str(), "active" | "weak" | "recovering")
            {
                return None;
            }
            next.map(|scheduled_for| DerivedTask {
                source_type: "inspection",
                source_id,
                meliponary_id,
                colony_id: Some(colony_id.clone()),
                box_id,
                task_type: "inspection",
                title: format!("Inspecionar {code}"),
                scheduled_for,
            })
        },
    );
    reconcile_derived(pool, Some(&colony_id), None, "inspection", desired).await
}

pub async fn reconcile_feeding(pool: &SqlitePool, colony_id: &str) -> Result<(), AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    operational::ensure_colony_exists(pool, &colony_id).await?;
    type Latest = (
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<String>,
        String,
    );
    let latest: Option<Latest> = sqlx::query_as(
        "SELECT f.id,f.next_feeding_at,f.box_id,c.meliponary_id,c.code,m.archived_at,c.status
         FROM feedings f
         JOIN colonies c ON c.id=f.colony_id
         JOIN meliponaries m ON m.id=c.meliponary_id
         WHERE f.colony_id=? AND f.voided_at IS NULL
         ORDER BY f.fed_at DESC,f.created_at DESC,f.id DESC LIMIT 1",
    )
    .bind(&colony_id)
    .fetch_optional(pool)
    .await?;
    let desired = latest.and_then(
        |(source_id, next, box_id, meliponary_id, code, archived_at, status)| {
            if archived_at.is_some() || !matches!(status.as_str(), "active" | "weak" | "recovering")
            {
                return None;
            }
            next.map(|scheduled_for| DerivedTask {
                source_type: "feeding",
                source_id,
                meliponary_id,
                colony_id: Some(colony_id.clone()),
                box_id,
                task_type: "feeding",
                title: format!("Alimentar {code}"),
                scheduled_for,
            })
        },
    );
    reconcile_derived(pool, Some(&colony_id), None, "feeding", desired).await
}

pub async fn reconcile_maintenance(pool: &SqlitePool, box_id: &str) -> Result<(), AppError> {
    let box_id = required(box_id, "Caixa")?;
    type Latest = (
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<String>,
        String,
    );
    let latest: Option<Latest> = sqlx::query_as(
        "SELECT r.id,r.next_maintenance_at,r.colony_id,b.meliponary_id,b.code,m.archived_at,b.status
         FROM box_maintenance_records r
         JOIN boxes b ON b.id=r.box_id
         JOIN meliponaries m ON m.id=b.meliponary_id
         WHERE r.box_id=? AND r.voided_at IS NULL
         ORDER BY r.maintained_at DESC,r.created_at DESC,r.id DESC LIMIT 1",
    )
    .bind(&box_id)
    .fetch_optional(pool)
    .await?;
    let desired = latest.and_then(
        |(source_id, next, colony_id, meliponary_id, code, archived_at, status)| {
            if archived_at.is_some() || status == "retired" {
                return None;
            }
            next.map(|scheduled_for| DerivedTask {
                source_type: "maintenance",
                source_id,
                meliponary_id,
                colony_id,
                box_id: Some(box_id.clone()),
                task_type: "maintenance",
                title: format!("Revisar caixa {code}"),
                scheduled_for,
            })
        },
    );
    reconcile_derived(pool, None, Some(&box_id), "maintenance", desired).await
}

pub async fn reconcile_all(pool: &SqlitePool) -> Result<(), AppError> {
    let colonies: Vec<String> = sqlx::query_scalar("SELECT id FROM colonies")
        .fetch_all(pool)
        .await?;
    for colony_id in colonies {
        reconcile_inspection(pool, &colony_id).await?;
        reconcile_feeding(pool, &colony_id).await?;
    }
    let boxes: Vec<String> = sqlx::query_scalar("SELECT id FROM boxes")
        .fetch_all(pool)
        .await?;
    for box_id in boxes {
        reconcile_maintenance(pool, &box_id).await?;
    }
    Ok(())
}

pub async fn mark_completed_by_fact_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: &str,
    expected_type: &str,
    fact_type: &str,
    fact_id: &str,
) -> Result<(), AppError> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT task_type,status FROM scheduled_tasks WHERE id=?")
            .bind(task_id)
            .fetch_optional(&mut **tx)
            .await?;
    let (task_type, status) =
        row.ok_or_else(|| AppError::NotFound("Tarefa não encontrada.".to_owned()))?;
    if task_type != expected_type || status != "pending" {
        return Err(AppError::Validation(
            "A tarefa não está disponível para esta execução.".to_owned(),
        ));
    }
    let now = now_tx(tx).await?;
    sqlx::query(
        "UPDATE scheduled_tasks SET status='completed',completed_at=?,completed_by_type=?,completed_by_id=?,updated_at=? WHERE id=?",
    )
    .bind(&now)
    .bind(fact_type)
    .bind(fact_id)
    .bind(&now)
    .bind(task_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        feeding::{self, CreateFeeding},
        inspections::{self, CreateInspection},
        maintenance::{self, CreateBoxMaintenance},
        repository,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    async fn seeded() -> (SqlitePool, String, String, String) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        let mel = repository::create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let species = repository::create_species(
            &pool,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let box_record = repository::create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: mel.id.clone(),
                code: "CX-001".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let colony = repository::create_colony(
            &pool,
            CreateColony {
                meliponary_id: mel.id.clone(),
                species_id: species.id,
                code: "JAT-001".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: Some("2026-01-01 09:00:00".into()),
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::place_colony(
            &pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: box_record.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        (pool, mel.id, colony.id, box_record.id)
    }

    async fn future(pool: &SqlitePool, modifier: &str) -> String {
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S','now','localtime',?)")
            .bind(modifier)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn pending_for(
        pool: &SqlitePool,
        colony_id: Option<String>,
        box_id: Option<String>,
        task_type: &str,
    ) -> Vec<ScheduledTask> {
        list(
            pool,
            TaskQuery {
                view: Some("pending".into()),
                colony_id,
                box_id,
                task_type: Some(task_type.into()),
                ..Default::default()
            },
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn manual_task_supports_views_and_generic_completion() {
        let (pool, meliponary_id, _, _) = seeded().await;
        let now = time::local_now(&pool).await.unwrap();
        let task = create_manual(
            &pool,
            CreateTask {
                meliponary_id,
                colony_id: None,
                box_id: None,
                task_type: "generic".into(),
                title: "Conferir autorização".into(),
                description: None,
                scheduled_for: now,
                priority: Some("attention".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            list(
                &pool,
                TaskQuery {
                    view: Some("pending".into()),
                    ..Default::default()
                }
            )
            .await
            .unwrap()
            .len(),
            1
        );
        complete_generic(&pool, &task.id).await.unwrap();
        assert_eq!(
            list(
                &pool,
                TaskQuery {
                    view: Some("completed".into()),
                    ..Default::default()
                }
            )
            .await
            .unwrap()
            .len(),
            1
        );
    }

    #[tokio::test]
    async fn reschedule_preserves_lineage_and_original() {
        let (pool, meliponary_id, _, _) = seeded().await;
        let now = time::local_now(&pool).await.unwrap();
        let task = create_manual(
            &pool,
            CreateTask {
                meliponary_id,
                colony_id: None,
                box_id: None,
                task_type: "generic".into(),
                title: "Limpar área".into(),
                description: None,
                scheduled_for: now.clone(),
                priority: None,
            },
        )
        .await
        .unwrap();
        let next = reschedule(
            &pool,
            RescheduleTask {
                id: task.id.clone(),
                scheduled_for: now,
                reason: Some("Chuva".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(get(&pool, &task.id).await.unwrap().status, "rescheduled");
        assert_eq!(next.status, "pending");
        assert_eq!(next.rescheduled_from_id.as_deref(), Some(task.id.as_str()));
    }

    #[tokio::test]
    async fn cancellation_and_skip_require_reason() {
        let (pool, meliponary_id, _, _) = seeded().await;
        let now = time::local_now(&pool).await.unwrap();
        let one = create_manual(
            &pool,
            CreateTask {
                meliponary_id: meliponary_id.clone(),
                colony_id: None,
                box_id: None,
                task_type: "generic".into(),
                title: "Uma".into(),
                description: None,
                scheduled_for: now.clone(),
                priority: None,
            },
        )
        .await
        .unwrap();
        assert!(cancel(
            &pool,
            TaskReason {
                id: one.id.clone(),
                reason: " ".into()
            }
        )
        .await
        .is_err());
        assert_eq!(
            cancel(
                &pool,
                TaskReason {
                    id: one.id,
                    reason: "Sem necessidade".into()
                }
            )
            .await
            .unwrap()
            .status,
            "cancelled"
        );
        let two = create_manual(
            &pool,
            CreateTask {
                meliponary_id,
                colony_id: None,
                box_id: None,
                task_type: "generic".into(),
                title: "Duas".into(),
                description: None,
                scheduled_for: now,
                priority: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            skip(
                &pool,
                TaskReason {
                    id: two.id,
                    reason: "Decisão consciente".into()
                }
            )
            .await
            .unwrap()
            .status,
            "skipped"
        );
    }

    #[tokio::test]
    async fn inspection_next_date_reconciles_without_duplicate() {
        let (pool, _, colony_id, _) = seeded().await;
        let first = inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-01-10 10:00:00".into()),
                strength: Some("medium".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2026-01-17 10:00:00".into()),
            },
        )
        .await
        .unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        let pending = pending_for(&pool, Some(colony_id.clone()), None, "inspection").await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].source_id.as_deref(), Some(first.id.as_str()));
        inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-01-20 10:00:00".into()),
                strength: Some("strong".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2026-01-27 10:00:00".into()),
            },
        )
        .await
        .unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        let pending = pending_for(&pool, Some(colony_id), None, "inspection").await;
        assert_eq!(pending.len(), 1);
        let cancelled = list(
            &pool,
            TaskQuery {
                view: Some("cancelled".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(cancelled.len(), 1);
    }

    #[tokio::test]
    async fn manual_reschedule_of_derived_inspection_survives_reconcile_and_source_changes_win() {
        let (pool, _, colony_id, _) = seeded().await;
        let source_date = future(&pool, "+2 days").await;
        let manual_date = future(&pool, "+4 days").await;
        let corrected_date = future(&pool, "+6 days").await;
        let source = inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some(time::local_now(&pool).await.unwrap()),
                strength: Some("medium".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some(source_date.clone()),
            },
        )
        .await
        .unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        let original = pending_for(&pool, Some(colony_id.clone()), None, "inspection")
            .await
            .remove(0);
        let manual = reschedule(
            &pool,
            RescheduleTask {
                id: original.id,
                scheduled_for: manual_date.clone(),
                reason: Some("Ajuste operacional".into()),
            },
        )
        .await
        .unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        let pending = pending_for(&pool, Some(colony_id.clone()), None, "inspection").await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, manual.id);
        assert_eq!(pending[0].scheduled_for, manual_date);
        assert_eq!(pending[0].source_id.as_deref(), Some(source.id.as_str()));

        sqlx::query("UPDATE inspections SET next_inspection_at=? WHERE id=?")
            .bind(&corrected_date)
            .bind(&source.id)
            .execute(&pool)
            .await
            .unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        let pending = pending_for(&pool, Some(colony_id.clone()), None, "inspection").await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].scheduled_for, corrected_date);
        assert_ne!(pending[0].id, manual.id);

        sqlx::query("UPDATE inspections SET voided_at=CURRENT_TIMESTAMP,void_reason='teste' WHERE id=?")
            .bind(&source.id)
            .execute(&pool)
            .await
            .unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        assert!(pending_for(&pool, Some(colony_id), None, "inspection")
            .await
            .is_empty());
    }

    #[tokio::test]
    async fn manual_reschedule_of_derived_feeding_survives_plain_reconcile() {
        let (pool, _, colony_id, _) = seeded().await;
        let source_date = future(&pool, "+2 days").await;
        let manual_date = future(&pool, "+4 days").await;
        feeding::create(
            &pool,
            CreateFeeding {
                colony_id: colony_id.clone(),
                fed_at: Some(time::local_now(&pool).await.unwrap()),
                food_type: "Xarope".into(),
                quantity: None,
                unit: None,
                response_notes: None,
                notes: None,
                next_feeding_at: Some(source_date),
            },
        )
        .await
        .unwrap();
        reconcile_feeding(&pool, &colony_id).await.unwrap();
        let original = pending_for(&pool, Some(colony_id.clone()), None, "feeding")
            .await
            .remove(0);
        let manual = reschedule(
            &pool,
            RescheduleTask {
                id: original.id,
                scheduled_for: manual_date.clone(),
                reason: Some("Ajuste operacional".into()),
            },
        )
        .await
        .unwrap();
        reconcile_feeding(&pool, &colony_id).await.unwrap();
        let pending = pending_for(&pool, Some(colony_id), None, "feeding").await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, manual.id);
        assert_eq!(pending[0].scheduled_for, manual_date);
    }

    #[tokio::test]
    async fn manual_reschedule_of_derived_maintenance_survives_plain_reconcile() {
        let (pool, _, _, box_id) = seeded().await;
        let source_date = future(&pool, "+2 days").await;
        let manual_date = future(&pool, "+4 days").await;
        maintenance::create(
            &pool,
            CreateBoxMaintenance {
                box_id: box_id.clone(),
                maintained_at: Some(time::local_now(&pool).await.unwrap()),
                maintenance_type: "cleaning".into(),
                description: None,
                performed_by: None,
                cost: None,
                next_maintenance_at: Some(source_date),
            },
        )
        .await
        .unwrap();
        reconcile_maintenance(&pool, &box_id).await.unwrap();
        let original = pending_for(&pool, None, Some(box_id.clone()), "maintenance")
            .await
            .remove(0);
        let manual = reschedule(
            &pool,
            RescheduleTask {
                id: original.id,
                scheduled_for: manual_date.clone(),
                reason: Some("Ajuste operacional".into()),
            },
        )
        .await
        .unwrap();
        reconcile_maintenance(&pool, &box_id).await.unwrap();
        let pending = pending_for(&pool, None, Some(box_id), "maintenance").await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, manual.id);
        assert_eq!(pending[0].scheduled_for, manual_date);
    }

    #[tokio::test]
    async fn reconcile_all_repairs_missing_derived_task_idempotently() {
        let (pool, _, colony_id, _) = seeded().await;
        let source_date = future(&pool, "+2 days").await;
        inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some(time::local_now(&pool).await.unwrap()),
                strength: Some("medium".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some(source_date),
            },
        )
        .await
        .unwrap();
        reconcile_inspection(&pool, &colony_id).await.unwrap();
        sqlx::query("DELETE FROM scheduled_tasks WHERE task_type='inspection' AND colony_id=? AND status='pending'")
            .bind(&colony_id)
            .execute(&pool)
            .await
            .unwrap();
        assert!(pending_for(&pool, Some(colony_id.clone()), None, "inspection")
            .await
            .is_empty());
        reconcile_all(&pool).await.unwrap();
        reconcile_all(&pool).await.unwrap();
        assert_eq!(
            pending_for(&pool, Some(colony_id), None, "inspection")
                .await
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn executive_summary_buckets_do_not_overlap() {
        let (pool, meliponary_id, _, _) = seeded().await;
        for (id, scheduled_for) in [
            ("overdue", "2026-09-10 11:00:00"),
            ("today", "2026-09-10 13:00:00"),
            ("next", "2026-09-11 09:00:00"),
            ("future", "2026-09-18 00:00:00"),
        ] {
            sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,task_type,title,scheduled_for) VALUES(?,?,'generic',?,?)")
                .bind(id)
                .bind(&meliponary_id)
                .bind(id)
                .bind(scheduled_for)
                .execute(&pool)
                .await
                .unwrap();
        }
        let summary = summary_at(&pool, Some(&meliponary_id), "2026-09-10 12:00:00")
            .await
            .unwrap();
        assert_eq!(summary.overdue, 1);
        assert_eq!(summary.today, 1);
        assert_eq!(summary.next_seven_days, 1);
        assert_eq!(summary.future, 1);
        assert_eq!(summary_at(&pool, Some("missing"), "2026-09-10 12:00:00").await.unwrap().overdue, 0);
    }
}
