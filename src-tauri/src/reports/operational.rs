use super::{
    production, resolve_filter, AgendaMetrics, AgendaReport, AgendaReportRow, CostReport,
    CostReportRow, CountByLabel, MeliponaryReport, OperationalManagement, OperationalMovements,
    OperationalPlantel, OperationalReport, ReportFilter, ResolvedFilter,
};
use crate::repository::AppError;
use sqlx::SqlitePool;

pub(super) async fn operational_report(
    pool: &SqlitePool,
    input: &ReportFilter,
) -> Result<OperationalReport, AppError> {
    let filter = resolve_filter(pool, input).await?;
    operational_report_resolved(pool, &filter).await
}

async fn operational_report_resolved(
    pool: &SqlitePool,
    filter: &ResolvedFilter,
) -> Result<OperationalReport, AppError> {
    let (total_colonies, active_colonies): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END), 0)
         FROM colonies
         WHERE (? IS NULL OR meliponary_id = ?)",
    )
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let active_boxes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM boxes
         WHERE status = 'active' AND (? IS NULL OR meliponary_id = ?)",
    )
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let current_occupancies: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM colony_box_occupancies o
         JOIN boxes b ON b.id = o.box_id
         WHERE o.ended_at IS NULL
           AND (? IS NULL OR b.meliponary_id = ?)",
    )
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let status_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT status, COUNT(*)
         FROM colonies
         WHERE (? IS NULL OR meliponary_id = ?)
         GROUP BY status
         ORDER BY status",
    )
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_all(pool)
    .await?;
    let colony_statuses = status_rows
        .into_iter()
        .map(|(key, count)| CountByLabel {
            label: colony_status_label(&key).to_owned(),
            key,
            count,
        })
        .collect();

    let inspections =
        period_count_by_colony(pool, filter, "inspections", "inspected_at", "voided_at").await?;
    let feedings = period_count_by_colony(pool, filter, "feedings", "fed_at", "voided_at").await?;
    let events =
        period_count_by_colony(pool, filter, "colony_events", "occurred_at", "voided_at").await?;
    let maintenance: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM box_maintenance_records r
         JOIN boxes b ON b.id = r.box_id
         WHERE r.voided_at IS NULL
           AND r.maintained_at >= ? AND r.maintained_at <= ?
           AND (? IS NULL OR b.meliponary_id = ?)",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let transfers: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM colony_movements m
         WHERE m.voided_at IS NULL AND m.reversed_at IS NULL
           AND m.movement_type IN ('internal_transfer', 'external_transfer')
           AND m.moved_at >= ? AND m.moved_at <= ?
           AND (? IS NULL OR m.from_meliponary_id = ? OR m.to_meliponary_id = ?)",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let temporary_started: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM colony_movements m
         WHERE m.voided_at IS NULL AND m.reversed_at IS NULL
           AND m.movement_type = 'transport'
           AND m.moved_at >= ? AND m.moved_at <= ?
           AND (? IS NULL OR m.from_meliponary_id = ?)",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let returns_completed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM transport_returns r
         JOIN colony_movements m ON m.id = r.movement_id
         WHERE r.reversed_at IS NULL
           AND m.voided_at IS NULL AND m.reversed_at IS NULL
           AND r.returned_at >= ? AND r.returned_at <= ?
           AND (? IS NULL OR m.from_meliponary_id = ?)",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let temporary_open_at_end: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM colony_movements m
         WHERE m.movement_type = 'transport'
           AND m.voided_at IS NULL AND m.reversed_at IS NULL
           AND m.moved_at <= ?
           AND (? IS NULL OR m.from_meliponary_id = ?)
           AND NOT EXISTS (
               SELECT 1 FROM transport_returns r
               WHERE r.movement_id = m.id
                 AND r.reversed_at IS NULL
                 AND r.returned_at <= ?
           )",
    )
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .bind(&filter.end_at)
    .fetch_one(pool)
    .await?;

    Ok(OperationalReport {
        context: filter.context.clone(),
        plantel: OperationalPlantel {
            total_colonies,
            active_colonies,
            active_boxes,
            current_occupancies,
            colony_statuses,
        },
        management: OperationalManagement {
            inspections,
            feedings,
            maintenance,
            events,
        },
        production: production::production_summary(pool, filter).await?,
        movements: OperationalMovements {
            transfers,
            temporary_started,
            returns_completed,
            temporary_open_at_end,
        },
        agenda: agenda_metrics(pool, filter).await?,
    })
}

async fn period_count_by_colony(
    pool: &SqlitePool,
    filter: &ResolvedFilter,
    table: &str,
    date_column: &str,
    void_column: &str,
) -> Result<i64, AppError> {
    let allowed = match (table, date_column, void_column) {
        ("inspections", "inspected_at", "voided_at")
        | ("feedings", "fed_at", "voided_at")
        | ("colony_events", "occurred_at", "voided_at") => true,
        _ => false,
    };
    if !allowed {
        return Err(AppError::Validation(
            "Consulta de relatório inválida.".to_owned(),
        ));
    }
    let sql = format!(
        "SELECT COUNT(*) FROM {table} r
         JOIN colonies c ON c.id = r.colony_id
         WHERE r.{void_column} IS NULL
           AND r.{date_column} >= ? AND r.{date_column} <= ?
           AND (? IS NULL OR c.meliponary_id = ?)"
    );
    Ok(sqlx::query_scalar(&sql)
        .bind(&filter.start_at)
        .bind(&filter.end_at)
        .bind(filter.meliponary_id.as_deref())
        .bind(filter.meliponary_id.as_deref())
        .fetch_one(pool)
        .await?)
}

pub(super) async fn cost_report(
    pool: &SqlitePool,
    input: &ReportFilter,
) -> Result<CostReport, AppError> {
    let filter = resolve_filter(pool, input).await?;
    cost_report_resolved(pool, &filter).await
}

async fn cost_report_resolved(
    pool: &SqlitePool,
    filter: &ResolvedFilter,
) -> Result<CostReport, AppError> {
    let rows = sqlx::query_as::<_, CostReportRow>(
        "SELECT r.maintained_at, m.name AS meliponary_name, b.code AS box_code,
                c.code AS colony_code, r.maintenance_type, r.performed_by,
                r.description, r.cost
         FROM box_maintenance_records r
         JOIN boxes b ON b.id = r.box_id
         JOIN meliponaries m ON m.id = b.meliponary_id
         LEFT JOIN colonies c ON c.id = r.colony_id
         WHERE r.voided_at IS NULL AND r.cost IS NOT NULL
           AND r.maintained_at >= ? AND r.maintained_at <= ?
           AND (? IS NULL OR b.meliponary_id = ?)
         ORDER BY r.maintained_at, b.code COLLATE NOCASE, r.id",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_all(pool)
    .await?;
    let total = rows.iter().map(|row| row.cost).sum();
    Ok(CostReport {
        context: filter.context.clone(),
        source_description: "Somente custos realmente registrados em manutenções de caixas.".to_owned(),
        currency_assumption: "A interface atual registra e apresenta o campo de custo em reais (BRL); o banco ainda não possui coluna de moeda.".to_owned(),
        total,
        rows,
    })
}

pub(super) async fn agenda_report(
    pool: &SqlitePool,
    input: &ReportFilter,
) -> Result<AgendaReport, AppError> {
    let filter = resolve_filter(pool, input).await?;
    let rows = agenda_rows(pool, &filter).await?;
    let metrics = agenda_metrics(pool, &filter).await?;
    Ok(AgendaReport {
        context: filter.context,
        metrics,
        rows,
    })
}

pub(super) async fn agenda_rows(
    pool: &SqlitePool,
    filter: &ResolvedFilter,
) -> Result<Vec<AgendaReportRow>, AppError> {
    Ok(sqlx::query_as::<_, AgendaReportRow>(
        "SELECT t.id, t.scheduled_for, m.name AS meliponary_name,
                c.code AS colony_code, b.code AS box_code,
                t.task_type, t.title, t.status, t.completed_at,
                CASE
                    WHEN t.status = 'completed' AND t.completed_at <= t.scheduled_for THEN 'on_time'
                    WHEN t.status = 'completed' AND t.completed_at > t.scheduled_for THEN 'late'
                    ELSE 'not_applicable'
                END AS timing,
                t.rescheduled_from_id
         FROM scheduled_tasks t
         JOIN meliponaries m ON m.id = t.meliponary_id
         LEFT JOIN colonies c ON c.id = t.colony_id
         LEFT JOIN boxes b ON b.id = t.box_id
         WHERE t.scheduled_for >= ? AND t.scheduled_for <= ?
           AND (? IS NULL OR t.meliponary_id = ?)
         ORDER BY t.scheduled_for, t.title COLLATE NOCASE, t.id",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_all(pool)
    .await?)
}

pub(super) async fn agenda_metrics(
    pool: &SqlitePool,
    filter: &ResolvedFilter,
) -> Result<AgendaMetrics, AppError> {
    let created: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scheduled_tasks
         WHERE created_at >= ? AND created_at <= ?
           AND (? IS NULL OR meliponary_id = ?)",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let (scheduled, completed, completed_on_time, completed_late, cancelled, skipped, rescheduled): (
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = sqlx::query_as(
        "SELECT COUNT(*),
                COALESCE(SUM(status = 'completed'), 0),
                COALESCE(SUM(status = 'completed' AND completed_at <= scheduled_for), 0),
                COALESCE(SUM(status = 'completed' AND completed_at > scheduled_for), 0),
                COALESCE(SUM(status = 'cancelled'), 0),
                COALESCE(SUM(status = 'skipped'), 0),
                COALESCE(SUM(status = 'rescheduled'), 0)
         FROM scheduled_tasks
         WHERE scheduled_for >= ? AND scheduled_for <= ?
           AND (? IS NULL OR meliponary_id = ?)",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    let overdue_pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scheduled_tasks
         WHERE status = 'pending'
           AND scheduled_for >= ? AND scheduled_for <= ?
           AND scheduled_for < ?
           AND (? IS NULL OR meliponary_id = ?)",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(&filter.context.generated_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(AgendaMetrics {
        created,
        scheduled,
        completed,
        completed_on_time,
        completed_late,
        cancelled,
        skipped,
        rescheduled,
        overdue_pending,
    })
}

pub(super) async fn meliponary_report(
    pool: &SqlitePool,
    input: &ReportFilter,
) -> Result<MeliponaryReport, AppError> {
    let filter = resolve_filter(pool, input).await?;
    if filter.meliponary_id.is_none() {
        return Err(AppError::Validation(
            "Selecione um meliponário para este relatório.".to_owned(),
        ));
    }
    let operational = operational_report_resolved(pool, &filter).await?;
    let maintenance_cost_total = cost_report_resolved(pool, &filter).await?.total;
    Ok(MeliponaryReport {
        context: filter.context,
        operational,
        maintenance_cost_total,
    })
}

fn colony_status_label(status: &str) -> &str {
    match status {
        "active" => "Ativas",
        "weak" => "Fracas",
        "recovering" => "Em recuperação",
        "transferred" => "Transferidas",
        "lost" => "Perdidas",
        "inactive" => "Inativas",
        _ => status,
    }
}
