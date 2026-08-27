use crate::{repository::AppError, time};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub alert_key: String,
    pub colony_id: String,
    pub colony_code: String,
    pub alert_type: String,
    pub severity: String,
    pub due_at: Option<String>,
    pub title: String,
    pub details: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Alert>, AppError> {
    let now = time::local_now(pool).await?;
    Ok(sqlx::query_as::<_, Alert>(
        "WITH latest_inspection AS (
            SELECT i.*, ROW_NUMBER() OVER (
                PARTITION BY i.colony_id
                ORDER BY i.inspected_at DESC, i.created_at DESC, i.id DESC
            ) AS rn FROM inspections i
         ),
         latest_feeding AS (
            SELECT f.*, ROW_NUMBER() OVER (
                PARTITION BY f.colony_id
                ORDER BY f.fed_at DESC, f.created_at DESC, f.id DESC
            ) AS rn FROM feedings f
         )
         SELECT alert_key, colony_id, colony_code, alert_type, severity, due_at, title, details
         FROM (
            SELECT 'inspection_due:' || li.id AS alert_key, c.id AS colony_id,
                c.code AS colony_code, 'inspection_due' AS alert_type,
                'attention' AS severity, li.next_inspection_at AS due_at,
                'Inspeção pendente' AS title,
                printf('Revisão prevista para %s.', li.next_inspection_at) AS details
            FROM latest_inspection li JOIN colonies c ON c.id = li.colony_id
            WHERE li.rn = 1 AND li.next_inspection_at IS NOT NULL
              AND li.next_inspection_at <= ?
              AND c.status IN ('active', 'weak', 'recovering')
            UNION ALL
            SELECT 'feeding_due:' || lf.id, c.id, c.code, 'feeding_due', 'attention',
                lf.next_feeding_at, 'Alimentação pendente',
                printf('Próxima alimentação prevista para %s. Último alimento: %s.',
                    lf.next_feeding_at, lf.food_type)
            FROM latest_feeding lf JOIN colonies c ON c.id = lf.colony_id
            WHERE lf.rn = 1 AND lf.next_feeding_at IS NOT NULL
              AND lf.next_feeding_at <= ?
              AND c.status IN ('active', 'weak', 'recovering')
            UNION ALL
            SELECT 'weak_colony:' || c.id, c.id, c.code, 'weak_colony', 'attention',
                NULL, 'Colônia fraca',
                'A última inspeção classificou a colônia como fraca.'
            FROM colonies c
            JOIN latest_inspection li ON li.colony_id = c.id AND li.rn = 1
            WHERE c.status IN ('active', 'weak', 'recovering') AND li.strength = 'weak'
         ) alerts
         ORDER BY CASE alerts.severity WHEN 'critical' THEN 0 WHEN 'attention' THEN 1 ELSE 2 END,
            COALESCE(alerts.due_at, '9999-12-31 23:59:59'),
            alerts.colony_code COLLATE NOCASE, alerts.alert_type",
    )
    .bind(&now)
    .bind(&now)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(list(pool).await?.len() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        feeding::{self, CreateFeeding},
        inspections::{self, CreateInspection}, repository,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new().max_connections(1)
            .connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        pool
    }

    async fn seed(pool: &SqlitePool) -> String {
        let m = repository::create_meliponary(pool, CreateMeliponary {
            name: "Principal".into(), responsible_name: None, location: None, notes: None,
        }).await.unwrap();
        let s = repository::create_species(pool, CreateSpecies {
            common_name: "Jataí".into(), scientific_name: None, genus: None, notes: None,
        }).await.unwrap();
        let b = repository::create_box(pool, CreateHiveBox {
            meliponary_id: m.id.clone(), code: "CX-001".into(), model: None,
            material: None, location_note: None, notes: None,
        }).await.unwrap();
        let c = repository::create_colony(pool, CreateColony {
            meliponary_id: m.id, species_id: s.id, code: "JAT-001".into(),
            origin_type: None, origin_notes: None,
            installed_at: Some("2000-01-01 09:00:00".into()), mother_colony_id: None, notes: None,
        }).await.unwrap();
        repository::place_colony(pool, PlaceColony {
            colony_id: c.id.clone(), box_id: b.id, started_at: Some("2000-01-01 09:00:00".into()),
            reason: None, notes: None,
        }).await.unwrap();
        c.id
    }

    #[tokio::test]
    async fn overdue_items_use_same_local_reference() {
        let pool = test_pool().await;
        let id = seed(&pool).await;
        inspections::create(&pool, CreateInspection {
            colony_id: id.clone(), inspected_at: Some("2000-01-10 10:00:00".into()),
            strength: Some("medium".into()), queen_present: None, laying_status: None,
            food_reserves: None, brood_status: None, pests_notes: None, observations: None,
            actions_taken: None, next_inspection_at: Some("2000-01-20 10:00:00".into()),
        }).await.unwrap();
        feeding::create(&pool, CreateFeeding {
            colony_id: id, fed_at: Some("2000-02-01 12:00:00".into()), food_type: "Xarope".into(),
            quantity: Some(50.0), unit: Some("ml".into()), response_notes: None, notes: None,
            next_feeding_at: Some("2000-02-08 12:00:00".into()),
        }).await.unwrap();
        let alerts = list(&pool).await.unwrap();
        assert!(alerts.iter().any(|a| a.alert_type == "inspection_due"));
        assert!(alerts.iter().any(|a| a.alert_type == "feeding_due"));
    }

    #[tokio::test]
    async fn weakness_comes_from_latest_inspection_not_legacy_status() {
        let pool = test_pool().await;
        let id = seed(&pool).await;
        sqlx::query("UPDATE colonies SET status = 'weak' WHERE id = ?")
            .bind(&id).execute(&pool).await.unwrap();
        assert!(!list(&pool).await.unwrap().iter().any(|a| a.alert_type == "weak_colony"));
        inspections::create(&pool, CreateInspection {
            colony_id: id, inspected_at: Some("2026-01-10 10:00:00".into()),
            strength: Some("weak".into()), queen_present: None, laying_status: None,
            food_reserves: None, brood_status: None, pests_notes: None, observations: None,
            actions_taken: None, next_inspection_at: None,
        }).await.unwrap();
        assert!(list(&pool).await.unwrap().iter().any(|a| a.alert_type == "weak_colony"));
    }

    #[tokio::test]
    async fn inactive_colony_is_not_alerted() {
        let pool = test_pool().await;
        let id = seed(&pool).await;
        inspections::create(&pool, CreateInspection {
            colony_id: id.clone(), inspected_at: Some("2026-01-10 10:00:00".into()),
            strength: Some("weak".into()), queen_present: None, laying_status: None,
            food_reserves: None, brood_status: None, pests_notes: None, observations: None,
            actions_taken: None, next_inspection_at: Some("2000-01-20 10:00:00".into()),
        }).await.unwrap();
        sqlx::query("UPDATE colonies SET status = 'inactive' WHERE id = ?")
            .bind(&id).execute(&pool).await.unwrap();
        assert!(list(&pool).await.unwrap().is_empty());
    }
}
