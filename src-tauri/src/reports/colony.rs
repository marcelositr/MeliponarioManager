use super::{resolve_filter, ColonyHistoryRow, ColonyReport, ColonyReportIdentity, ColonyReportInput};
use crate::repository::AppError;
use sqlx::{FromRow, SqlitePool};
use std::collections::HashSet;

#[derive(Debug, FromRow)]
struct AuditRow {
    id: String,
    entity_id: String,
    changed_at: String,
    action: String,
    reason: String,
}

pub(super) async fn colony_report(
    pool: &SqlitePool,
    input: &ColonyReportInput,
) -> Result<ColonyReport, AppError> {
    let filter = resolve_filter(pool, &input.filter).await?;
    let colony_id = input.colony_id.trim();
    if colony_id.is_empty() {
        return Err(AppError::Validation(
            "Selecione uma colônia para este relatório.".to_owned(),
        ));
    }

    let colony_meliponary_id: String =
        sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id = ?")
            .bind(colony_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;
    if filter
        .meliponary_id
        .as_deref()
        .is_some_and(|id| id != colony_meliponary_id)
    {
        return Err(AppError::Validation(
            "A colônia não pertence ao meliponário selecionado.".to_owned(),
        ));
    }

    let identity = sqlx::query_as::<_, ColonyReportIdentity>(
        "SELECT c.id AS colony_id, c.code AS colony_code,
                m.name AS meliponary_name, s.common_name AS species_name,
                s.scientific_name, c.origin_type, c.origin_notes, c.installed_at,
                c.status, b.code AS current_box_code, mother.code AS mother_colony_code
         FROM colonies c
         JOIN meliponaries m ON m.id = c.meliponary_id
         JOIN species s ON s.id = c.species_id
         LEFT JOIN colony_box_occupancies o ON o.colony_id = c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id = o.box_id
         LEFT JOIN colonies mother ON mother.id = c.mother_colony_id
         WHERE c.id = ?",
    )
    .bind(colony_id)
    .fetch_one(pool)
    .await?;

    let mut timeline = history_rows(pool, colony_id, &filter.start_at, &filter.end_at).await?;
    if !input.include_audit {
        timeline.retain(|row| !matches!(row.state.as_str(), "voided" | "reversed"));
    } else {
        append_audit_rows(pool, colony_id, &filter.start_at, &filter.end_at, &mut timeline).await?;
    }
    timeline.sort_by(|a, b| {
        a.occurred_at
            .cmp(&b.occurred_at)
            .then_with(|| a.category.cmp(&b.category))
            .then_with(|| a.source_id.cmp(&b.source_id))
    });

    let photo_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM inspection_photos p
         JOIN inspections i ON i.id = p.inspection_id
         WHERE i.colony_id = ? AND i.voided_at IS NULL",
    )
    .bind(colony_id)
    .fetch_one(pool)
    .await?;

    Ok(ColonyReport {
        context: filter.context,
        identity,
        include_audit: input.include_audit,
        photo_count,
        timeline,
    })
}

async fn history_rows(
    pool: &SqlitePool,
    colony_id: &str,
    start_at: &str,
    end_at: &str,
) -> Result<Vec<ColonyHistoryRow>, AppError> {
    Ok(sqlx::query_as::<_, ColonyHistoryRow>(
        "SELECT source_id, occurred_at, category, title, details, state
         FROM (
            SELECT o.id source_id, o.started_at occurred_at, 'Ocupação' category,
                   COALESCE(NULLIF(TRIM(o.reason), ''), 'Colônia colocada em caixa') title,
                   CASE WHEN o.notes IS NULL THEN 'Caixa ' || b.code
                        ELSE 'Caixa ' || b.code || ' · ' || o.notes END details,
                   CASE WHEN o.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END state
            FROM colony_box_occupancies o JOIN boxes b ON b.id=o.box_id WHERE o.colony_id=?
            UNION ALL
            SELECT i.id,i.inspected_at,'Inspeção','Inspeção',COALESCE(i.observations,i.actions_taken),
                   CASE WHEN i.voided_at IS NOT NULL THEN 'voided' WHEN i.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END
            FROM inspections i WHERE i.colony_id=?
            UNION ALL
            SELECT f.id,f.fed_at,'Alimentação','Alimentação: '||f.food_type,COALESCE(f.response_notes,f.notes),
                   CASE WHEN f.voided_at IS NOT NULL THEN 'voided' WHEN f.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END
            FROM feedings f WHERE f.colony_id=?
            UNION ALL
            SELECT p.id,p.harvested_at,'Produção',
                   'Produção: '||CASE p.product_type WHEN 'honey' THEN 'Mel' WHEN 'pollen' THEN 'Pólen' WHEN 'propolis' THEN 'Própolis' WHEN 'wax' THEN 'Cera' WHEN 'cerumen' THEN 'Cerume' ELSE 'Outro produto' END,
                   printf('%g %s%s',p.quantity,p.unit,CASE WHEN p.notes IS NOT NULL THEN ' · '||p.notes ELSE '' END),
                   CASE WHEN p.voided_at IS NOT NULL THEN 'voided' WHEN p.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END
            FROM production_records p WHERE p.colony_id=?
            UNION ALL
            SELECT e.id,e.occurred_at,'Evento',COALESCE(e.title,'Evento operacional'),e.details,
                   CASE WHEN e.voided_at IS NOT NULL THEN 'voided' WHEN e.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END
            FROM colony_events e WHERE e.colony_id=?
            UNION ALL
            SELECT r.id,r.maintained_at,'Manutenção','Manutenção de caixa',r.description,
                   CASE WHEN r.voided_at IS NOT NULL THEN 'voided' WHEN r.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END
            FROM box_maintenance_records r WHERE r.colony_id=?
            UNION ALL
            SELECT d.id,d.performed_at,'Divisão','Divisão de colônia',d.notes,
                   CASE WHEN d.voided_at IS NOT NULL THEN 'voided' WHEN d.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END
            FROM colony_divisions d WHERE d.parent_colony_id=? OR d.daughter_colony_id=?
            UNION ALL
            SELECT m.id,m.moved_at,'Movimentação',
                   CASE m.movement_type WHEN 'internal_transfer' THEN 'Transferência interna' WHEN 'external_transfer' THEN 'Transferência externa' ELSE 'Transporte temporário' END,
                   COALESCE(dest.name,m.destination,m.notes),
                   CASE WHEN m.reversed_at IS NOT NULL THEN 'reversed' WHEN m.voided_at IS NOT NULL THEN 'voided' WHEN m.corrected_at IS NOT NULL THEN 'corrected' ELSE 'effective' END
            FROM colony_movements m LEFT JOIN meliponaries dest ON dest.id=m.to_meliponary_id WHERE m.colony_id=?
            UNION ALL
            SELECT l.id,l.occurred_at,'Ciclo de vida','Ciclo de vida: '||l.previous_status||' → '||l.new_status,COALESCE(l.reason,l.notes),
                   CASE WHEN l.reversed_at IS NOT NULL THEN 'reversed' ELSE 'effective' END
            FROM colony_lifecycle_records l WHERE l.colony_id=?
            UNION ALL
            SELECT tr.id,tr.returned_at,'Transporte','Retorno de transporte temporário',tr.notes,
                   CASE WHEN tr.reversed_at IS NOT NULL THEN 'reversed' ELSE 'effective' END
            FROM transport_returns tr JOIN colony_movements m ON m.id=tr.movement_id WHERE m.colony_id=?
            UNION ALL
            SELECT t.id,t.scheduled_for,'Agenda','Agenda: '||t.title,
                   CASE t.status WHEN 'completed' THEN 'Concluída' WHEN 'cancelled' THEN 'Cancelada' WHEN 'skipped' THEN 'Ignorada' WHEN 'rescheduled' THEN 'Reagendada' ELSE 'Pendente' END,
                   'effective'
            FROM scheduled_tasks t WHERE t.colony_id=?
         ) history
         WHERE occurred_at>=? AND occurred_at<=?
         ORDER BY occurred_at,category,source_id",
    )
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(start_at)
    .bind(end_at)
    .fetch_all(pool)
    .await?)
}

async fn append_audit_rows(
    pool: &SqlitePool,
    colony_id: &str,
    start_at: &str,
    end_at: &str,
    timeline: &mut Vec<ColonyHistoryRow>,
) -> Result<(), AppError> {
    let source_ids: HashSet<String> = timeline.iter().map(|row| row.source_id.clone()).collect();
    let audits = sqlx::query_as::<_, AuditRow>(
        "SELECT id,entity_id,changed_at,action,reason FROM audit_records
         WHERE changed_at>=? AND changed_at<=? ORDER BY changed_at,id",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_all(pool)
    .await?;
    timeline.extend(
        audits
            .into_iter()
            .filter(|row| row.entity_id == colony_id || source_ids.contains(&row.entity_id))
            .map(|row| ColonyHistoryRow {
                source_id: row.id,
                occurred_at: row.changed_at,
                category: "Auditoria".to_owned(),
                title: audit_action_label(&row.action).to_owned(),
                details: Some(row.reason),
                state: "audit".to_owned(),
            }),
    );
    Ok(())
}

fn audit_action_label(action: &str) -> &str {
    match action {
        "correct" => "Correção administrativa",
        "void" => "Anulação administrativa",
        "reverse" => "Reversão administrativa",
        "complete_transport" => "Conclusão de transporte",
        "reopen_transport" => "Reabertura de transporte",
        _ => "Alteração auditada",
    }
}
