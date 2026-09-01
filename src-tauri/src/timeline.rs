use crate::{
    history::{self, TimelineEntry},
    lifecycle, maintenance,
    repository::AppError,
};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;

#[derive(Debug, FromRow)]
struct DecorationRow {
    source_type: String,
    source_id: String,
    corrected_at: Option<String>,
    voided_at: Option<String>,
    void_reason: Option<String>,
    reversed_at: Option<String>,
    reversal_reason: Option<String>,
}

type Decorations = HashMap<String, HashMap<String, DecorationRow>>;

pub async fn by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<TimelineEntry>, AppError> {
    let mut entries = history::timeline_by_colony(pool, colony_id).await?;
    entries.extend(maintenance::timeline_entries_by_colony(pool, colony_id).await?);
    entries.extend(lifecycle::timeline_entries(pool, colony_id).await?);
    entries.extend(
        sqlx::query_as::<_, TimelineEntry>(
            "SELECT
                'division' AS source_type,
                d.id AS source_id,
                d.performed_at AS occurred_at,
                'Divisão de colônia' AS title,
                d.notes AS details,
                b.code AS box_code,
                'info' AS severity
             FROM colony_divisions d
             LEFT JOIN boxes b ON b.id = d.source_box_id
             WHERE d.parent_colony_id = ? OR d.daughter_colony_id = ?",
        )
        .bind(colony_id)
        .bind(colony_id)
        .fetch_all(pool)
        .await?,
    );

    let decorations = load_decorations(pool, colony_id).await?;
    for entry in &mut entries {
        if let Some(decoration) = decorations
            .get(entry.source_type.as_str())
            .and_then(|by_id| by_id.get(entry.source_id.as_str()))
        {
            apply_decoration(entry, decoration);
        }
    }

    entries.sort_by(|a, b| {
        b.occurred_at
            .cmp(&a.occurred_at)
            .then_with(|| b.source_id.cmp(&a.source_id))
    });
    Ok(entries)
}

async fn load_decorations(pool: &SqlitePool, colony_id: &str) -> Result<Decorations, AppError> {
    let rows = sqlx::query_as::<_, DecorationRow>(
        "SELECT 'inspection' AS source_type, id AS source_id,
                corrected_at, voided_at, void_reason,
                NULL AS reversed_at, NULL AS reversal_reason
         FROM inspections
         WHERE colony_id = ?

         UNION ALL

         SELECT 'feeding', id,
                corrected_at, voided_at, void_reason,
                NULL, NULL
         FROM feedings
         WHERE colony_id = ?

         UNION ALL

         SELECT 'production', id,
                corrected_at, voided_at, void_reason,
                NULL, NULL
         FROM production_records
         WHERE colony_id = ?

         UNION ALL

         SELECT 'event', id,
                corrected_at, voided_at, void_reason,
                NULL, NULL
         FROM colony_events
         WHERE colony_id = ?

         UNION ALL

         SELECT 'box_maintenance', id,
                corrected_at, voided_at, void_reason,
                NULL, NULL
         FROM box_maintenance_records
         WHERE colony_id = ?

         UNION ALL

         SELECT 'division', id,
                corrected_at, voided_at, void_reason,
                NULL, NULL
         FROM colony_divisions
         WHERE parent_colony_id = ? OR daughter_colony_id = ?

         UNION ALL

         SELECT 'movement', id,
                corrected_at, voided_at, void_reason,
                reversed_at, reversal_reason
         FROM colony_movements
         WHERE colony_id = ?

         UNION ALL

         SELECT 'lifecycle', id,
                NULL, NULL, NULL,
                reversed_at, reversal_reason
         FROM colony_lifecycle_records
         WHERE colony_id = ?

         UNION ALL

         SELECT 'box_occupancy', id,
                corrected_at, NULL, NULL,
                NULL, NULL
         FROM colony_box_occupancies
         WHERE colony_id = ?",
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
    .fetch_all(pool)
    .await?;

    let mut decorations = Decorations::new();
    for row in rows {
        decorations
            .entry(row.source_type.clone())
            .or_default()
            .insert(row.source_id.clone(), row);
    }
    Ok(decorations)
}

fn apply_decoration(entry: &mut TimelineEntry, decoration: &DecorationRow) {
    match entry.source_type.as_str() {
        "inspection" => apply_voidable(
            entry,
            decoration,
            "Inspeção corrigida",
            "Registro anulado · Inspeção",
        ),
        "feeding" => apply_voidable(
            entry,
            decoration,
            "Alimentação corrigida",
            "Registro anulado · Alimentação",
        ),
        "production" => apply_voidable(
            entry,
            decoration,
            "Produção corrigida",
            "Registro anulado · Produção",
        ),
        "event" => apply_voidable(
            entry,
            decoration,
            "Evento corrigido",
            "Registro anulado · Evento",
        ),
        "box_maintenance" => apply_voidable(
            entry,
            decoration,
            "Manutenção corrigida",
            "Registro anulado · Manutenção",
        ),
        "division" => apply_voidable(
            entry,
            decoration,
            "Divisão corrigida",
            "Registro anulado · Divisão",
        ),
        "movement" => {
            if decoration.reversed_at.is_some() {
                entry.title = "Movimentação revertida".into();
                entry.details = decoration.reversal_reason.clone();
                entry.severity = "attention".into();
            } else {
                apply_voidable(
                    entry,
                    decoration,
                    "Movimentação corrigida",
                    "Registro anulado · Movimentação",
                );
            }
        }
        "lifecycle" if decoration.reversed_at.is_some() => {
            entry.title = "Ciclo de vida revertido".into();
            entry.details = decoration.reversal_reason.clone();
            entry.severity = "attention".into();
        }
        "box_occupancy" if decoration.corrected_at.is_some() => {
            entry.title = "Ocupação de caixa corrigida".into();
        }
        _ => {}
    }
}

fn apply_voidable(
    entry: &mut TimelineEntry,
    decoration: &DecorationRow,
    corrected_title: &str,
    voided_title: &str,
) {
    if decoration.voided_at.is_some() {
        entry.title = voided_title.to_owned();
        entry.details = decoration.void_reason.clone();
        entry.severity = "attention".into();
    } else if decoration.corrected_at.is_some() {
        entry.title = corrected_title.to_owned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(source_type: &str) -> TimelineEntry {
        TimelineEntry {
            source_type: source_type.into(),
            source_id: "source-1".into(),
            occurred_at: "2026-01-01 10:00:00".into(),
            title: "Original".into(),
            details: Some("Detalhe original".into()),
            box_code: None,
            severity: "info".into(),
        }
    }

    fn decoration(source_type: &str) -> DecorationRow {
        DecorationRow {
            source_type: source_type.into(),
            source_id: "source-1".into(),
            corrected_at: None,
            voided_at: None,
            void_reason: None,
            reversed_at: None,
            reversal_reason: None,
        }
    }

    #[test]
    fn voided_decoration_replaces_details_and_marks_attention() {
        let mut entry = entry("production");
        let mut decoration = decoration("production");
        decoration.corrected_at = Some("2026-01-02 10:00:00".into());
        decoration.voided_at = Some("2026-01-03 10:00:00".into());
        decoration.void_reason = Some("Lançamento incorreto".into());

        apply_decoration(&mut entry, &decoration);

        assert_eq!(entry.title, "Registro anulado · Produção");
        assert_eq!(entry.details.as_deref(), Some("Lançamento incorreto"));
        assert_eq!(entry.severity, "attention");
    }

    #[test]
    fn movement_reversal_has_precedence_over_void_and_correction() {
        let mut entry = entry("movement");
        let mut decoration = decoration("movement");
        decoration.corrected_at = Some("2026-01-02 10:00:00".into());
        decoration.voided_at = Some("2026-01-03 10:00:00".into());
        decoration.void_reason = Some("Motivo de anulação".into());
        decoration.reversed_at = Some("2026-01-04 10:00:00".into());
        decoration.reversal_reason = Some("Motivo da reversão".into());

        apply_decoration(&mut entry, &decoration);

        assert_eq!(entry.title, "Movimentação revertida");
        assert_eq!(entry.details.as_deref(), Some("Motivo da reversão"));
        assert_eq!(entry.severity, "attention");
    }

    #[test]
    fn correction_changes_title_without_overwriting_details() {
        let mut entry = entry("inspection");
        let mut decoration = decoration("inspection");
        decoration.corrected_at = Some("2026-01-02 10:00:00".into());

        apply_decoration(&mut entry, &decoration);

        assert_eq!(entry.title, "Inspeção corrigida");
        assert_eq!(entry.details.as_deref(), Some("Detalhe original"));
        assert_eq!(entry.severity, "info");
    }
}
