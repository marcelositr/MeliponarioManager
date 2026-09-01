use super::*;

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
    source_baseline: String,
}

async fn reconcile_derived_tx(
    tx: &mut Transaction<'_, Sqlite>,
    colony_scope: Option<&str>,
    box_scope: Option<&str>,
    task_type: &'static str,
    desired: Option<DerivedTask>,
) -> Result<(), AppError> {
    let pending = sqlx::query_as::<_, PendingDerived>(
        "WITH RECURSIVE lineage(id,root_scheduled_for) AS (
            SELECT id,scheduled_for FROM scheduled_tasks WHERE rescheduled_from_id IS NULL
            UNION ALL
            SELECT child.id,parent.root_scheduled_for
            FROM scheduled_tasks child
            JOIN lineage parent ON child.rescheduled_from_id=parent.id
         )
         SELECT t.id,t.source_id,
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
    .fetch_all(&mut **tx)
    .await?;
    let now = now_tx(tx).await?;
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
                .execute(&mut **tx)
                .await?;
                kept = true;
            }
            Some(next) if current.source_id == next.source_id && !kept => {
                sqlx::query(
                    "UPDATE scheduled_tasks SET status='rescheduled',reschedule_reason='Data futura alterada no fato de origem.',updated_at=? WHERE id=?",
                )
                .bind(&now)
                .bind(&current.id)
                .execute(&mut **tx)
                .await?;
            }
            _ => {
                sqlx::query(
                    "UPDATE scheduled_tasks SET status='cancelled',cancelled_at=?,cancellation_reason='Compromisso substituído ou invalidado pelo fato de origem.',updated_at=? WHERE id=?",
                )
                .bind(&now)
                .bind(&now)
                .bind(&current.id)
                .execute(&mut **tx)
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
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

async fn ensure_colony_exists_tx(
    tx: &mut Transaction<'_, Sqlite>,
    colony_id: &str,
) -> Result<(), AppError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id=?)")
        .bind(colony_id)
        .fetch_one(&mut **tx)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }
    Ok(())
}

pub(crate) async fn reconcile_inspection_tx(
    tx: &mut Transaction<'_, Sqlite>,
    colony_id: &str,
) -> Result<(), AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    ensure_colony_exists_tx(tx, &colony_id).await?;
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
        "SELECT i.id,i.next_inspection_at,o.box_id,c.meliponary_id,c.code,m.archived_at,c.status
         FROM inspections i
         JOIN colonies c ON c.id=i.colony_id
         JOIN meliponaries m ON m.id=c.meliponary_id
         LEFT JOIN colony_box_occupancies o ON o.colony_id=c.id AND o.ended_at IS NULL
         WHERE i.colony_id=? AND i.voided_at IS NULL
         ORDER BY i.inspected_at DESC,i.created_at DESC,i.id DESC LIMIT 1",
    )
    .bind(&colony_id)
    .fetch_optional(&mut **tx)
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
    reconcile_derived_tx(tx, Some(&colony_id), None, "inspection", desired).await
}

pub async fn reconcile_inspection(pool: &SqlitePool, colony_id: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    reconcile_inspection_tx(&mut tx, colony_id).await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) async fn reconcile_feeding_tx(
    tx: &mut Transaction<'_, Sqlite>,
    colony_id: &str,
) -> Result<(), AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    ensure_colony_exists_tx(tx, &colony_id).await?;
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
        "SELECT f.id,f.next_feeding_at,o.box_id,c.meliponary_id,c.code,m.archived_at,c.status
         FROM feedings f
         JOIN colonies c ON c.id=f.colony_id
         JOIN meliponaries m ON m.id=c.meliponary_id
         LEFT JOIN colony_box_occupancies o ON o.colony_id=c.id AND o.ended_at IS NULL
         WHERE f.colony_id=? AND f.voided_at IS NULL
         ORDER BY f.fed_at DESC,f.created_at DESC,f.id DESC LIMIT 1",
    )
    .bind(&colony_id)
    .fetch_optional(&mut **tx)
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
    reconcile_derived_tx(tx, Some(&colony_id), None, "feeding", desired).await
}

pub async fn reconcile_feeding(pool: &SqlitePool, colony_id: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    reconcile_feeding_tx(&mut tx, colony_id).await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) async fn reconcile_maintenance_tx(
    tx: &mut Transaction<'_, Sqlite>,
    box_id: &str,
) -> Result<(), AppError> {
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
    .fetch_optional(&mut **tx)
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
    reconcile_derived_tx(tx, None, Some(&box_id), "maintenance", desired).await
}

pub async fn reconcile_maintenance(pool: &SqlitePool, box_id: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    reconcile_maintenance_tx(&mut tx, box_id).await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) async fn reconcile_meliponary_tx(
    tx: &mut Transaction<'_, Sqlite>,
    meliponary_id: &str,
) -> Result<(), AppError> {
    let meliponary_id = required(meliponary_id, "Meliponário")?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meliponaries WHERE id=?)")
            .bind(&meliponary_id)
            .fetch_one(&mut **tx)
            .await?;
    if !exists {
        return Err(AppError::NotFound(
            "Meliponário não encontrado.".to_owned(),
        ));
    }

    let colonies: Vec<String> =
        sqlx::query_scalar("SELECT id FROM colonies WHERE meliponary_id=? ORDER BY id")
            .bind(&meliponary_id)
            .fetch_all(&mut **tx)
            .await?;
    for colony_id in colonies {
        reconcile_inspection_tx(tx, &colony_id).await?;
        reconcile_feeding_tx(tx, &colony_id).await?;
    }

    let boxes: Vec<String> =
        sqlx::query_scalar("SELECT id FROM boxes WHERE meliponary_id=? ORDER BY id")
            .bind(&meliponary_id)
            .fetch_all(&mut **tx)
            .await?;
    for box_id in boxes {
        reconcile_maintenance_tx(tx, &box_id).await?;
    }
    Ok(())
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
