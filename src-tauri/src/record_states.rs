use crate::repository::AppError;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecordAdminState {
    pub entity_type: String,
    pub entity_id: String,
    pub corrected_at: Option<String>,
    pub voided_at: Option<String>,
    pub void_reason: Option<String>,
    pub reversed_at: Option<String>,
    pub reversal_reason: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<RecordAdminState>, AppError> {
    Ok(sqlx::query_as::<_, RecordAdminState>(
        "SELECT entity_type, entity_id, corrected_at, voided_at, void_reason, reversed_at, reversal_reason
         FROM (
            SELECT 'inspection' AS entity_type, id AS entity_id, corrected_at, voided_at, void_reason,
                   NULL AS reversed_at, NULL AS reversal_reason FROM inspections
            UNION ALL
            SELECT 'feeding', id, corrected_at, voided_at, void_reason, NULL, NULL FROM feedings
            UNION ALL
            SELECT 'production', id, corrected_at, voided_at, void_reason, NULL, NULL FROM production_records
            UNION ALL
            SELECT 'box_maintenance', id, corrected_at, voided_at, void_reason, NULL, NULL FROM box_maintenance_records
            UNION ALL
            SELECT 'colony_event', id, corrected_at, voided_at, void_reason, NULL, NULL FROM colony_events
            UNION ALL
            SELECT 'movement', id, corrected_at, voided_at, void_reason, reversed_at, reversal_reason FROM colony_movements
            UNION ALL
            SELECT 'movement_document', id, corrected_at, voided_at, void_reason, NULL, NULL FROM movement_documents
            UNION ALL
            SELECT 'division', id, corrected_at, voided_at, void_reason, NULL, NULL FROM colony_divisions
            UNION ALL
            SELECT 'lifecycle', id, NULL, NULL, NULL, reversed_at, reversal_reason FROM colony_lifecycle_records
            UNION ALL
            SELECT 'box_occupancy', id, corrected_at, NULL, NULL, NULL, NULL FROM colony_box_occupancies
         ) states
         ORDER BY entity_type, entity_id",
    )
    .fetch_all(pool)
    .await?)
}

#[tauri::command]
pub async fn list_record_admin_states(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<RecordAdminState>, String> {
    list(&pool).await.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn exposes_void_and_reversal_state_without_changing_domain_records() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        sqlx::query("INSERT INTO meliponaries(id,name) VALUES('m1','Principal')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO species(id,common_name) VALUES('s1','Jataí')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO colonies(id,meliponary_id,species_id,code) VALUES('c1','m1','s1','JAT-001')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO inspections(id,colony_id,inspected_at,strength,voided_at,void_reason) VALUES('i1','c1','2026-01-01 10:00:00','medium','2026-01-02 10:00:00','Duplicada')")
            .execute(&pool)
            .await
            .unwrap();

        let states = list(&pool).await.unwrap();
        let inspection = states
            .iter()
            .find(|item| item.entity_type == "inspection" && item.entity_id == "i1")
            .unwrap();
        assert!(inspection.voided_at.is_some());
        assert_eq!(inspection.void_reason.as_deref(), Some("Duplicada"));
    }
}
