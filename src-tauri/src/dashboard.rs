use crate::{alerts, repository::AppError};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCount {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecentProduction {
    pub colony_code: String,
    pub product_type: String,
    pub quantity: f64,
    pub unit: String,
    pub harvested_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecentMovement {
    pub colony_code: String,
    pub movement_type: String,
    pub moved_at: String,
    pub destination: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardOverview {
    pub colony_statuses: Vec<DashboardCount>,
    pub species_distribution: Vec<DashboardCount>,
    pub inspection_strengths: Vec<DashboardCount>,
    pub occupied_boxes: i64,
    pub free_boxes: i64,
    pub alerts: Vec<alerts::Alert>,
    pub recent_production: Vec<RecentProduction>,
    pub recent_movements: Vec<RecentMovement>,
}

pub async fn overview(pool: &SqlitePool) -> Result<DashboardOverview, AppError> {
    let colony_statuses = sqlx::query_as::<_, DashboardCount>(
        "SELECT status AS label, COUNT(*) AS count
         FROM colonies
         GROUP BY status
         ORDER BY count DESC, status COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;

    let species_distribution = sqlx::query_as::<_, DashboardCount>(
        "SELECT s.common_name AS label, COUNT(c.id) AS count
         FROM species s
         JOIN colonies c ON c.species_id = s.id
         GROUP BY s.id, s.common_name
         ORDER BY count DESC, s.common_name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;

    let inspection_strengths = sqlx::query_as::<_, DashboardCount>(
        "WITH latest AS (
            SELECT i.colony_id, i.strength,
                   ROW_NUMBER() OVER (
                       PARTITION BY i.colony_id
                       ORDER BY i.inspected_at DESC, i.created_at DESC, i.id DESC
                   ) AS rn
            FROM inspections i
         )
         SELECT COALESCE(latest.strength, 'unknown') AS label, COUNT(*) AS count
         FROM colonies c
         LEFT JOIN latest ON latest.colony_id = c.id AND latest.rn = 1
         WHERE c.status IN ('active', 'weak', 'recovering')
         GROUP BY COALESCE(latest.strength, 'unknown')
         ORDER BY count DESC, label",
    )
    .fetch_all(pool)
    .await?;

    let occupied_boxes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM colony_box_occupancies WHERE ended_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    let free_boxes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM boxes b
         WHERE b.status = 'active'
           AND NOT EXISTS (
               SELECT 1 FROM colony_box_occupancies o
               WHERE o.box_id = b.id AND o.ended_at IS NULL
           )",
    )
    .fetch_one(pool)
    .await?;

    let recent_production = sqlx::query_as::<_, RecentProduction>(
        "SELECT c.code AS colony_code, p.product_type, p.quantity, p.unit, p.harvested_at
         FROM production_records p
         JOIN colonies c ON c.id = p.colony_id
         ORDER BY p.harvested_at DESC, p.created_at DESC
         LIMIT 5",
    )
    .fetch_all(pool)
    .await?;

    let recent_movements = sqlx::query_as::<_, RecentMovement>(
        "SELECT c.code AS colony_code,
                m.movement_type,
                m.moved_at,
                COALESCE(tm.name, m.destination) AS destination
         FROM colony_movements m
         JOIN colonies c ON c.id = m.colony_id
         LEFT JOIN meliponaries tm ON tm.id = m.to_meliponary_id
         ORDER BY m.moved_at DESC, m.created_at DESC
         LIMIT 5",
    )
    .fetch_all(pool)
    .await?;

    Ok(DashboardOverview {
        colony_statuses,
        species_distribution,
        inspection_strengths,
        occupied_boxes,
        free_boxes,
        alerts: alerts::list(pool).await?,
        recent_production,
        recent_movements,
    })
}

#[tauri::command]
pub async fn get_dashboard_overview(
    pool: State<'_, SqlitePool>,
) -> Result<DashboardOverview, String> {
    overview(&pool).await.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        inspections::{self, CreateInspection},
        repository,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn overview_derives_current_plantel_state() {
        let pool = test_pool().await;
        let meliponary = repository::create_meliponary(
            &pool,
            CreateMeliponary { name: "Principal".into(), responsible_name: None, location: None, notes: None },
        ).await.unwrap();
        let species = repository::create_species(
            &pool,
            CreateSpecies { common_name: "Jataí".into(), scientific_name: None, genus: None, notes: None },
        ).await.unwrap();
        let hive_box = repository::create_box(
            &pool,
            CreateHiveBox { meliponary_id: meliponary.id.clone(), code: "CX-001".into(), model: None, material: None, location_note: None, notes: None },
        ).await.unwrap();
        let colony = repository::create_colony(
            &pool,
            CreateColony { meliponary_id: meliponary.id, species_id: species.id, code: "JAT-001".into(), origin_type: None, origin_notes: None, installed_at: None, mother_colony_id: None, notes: None },
        ).await.unwrap();
        repository::place_colony(
            &pool,
            PlaceColony { colony_id: colony.id.clone(), box_id: hive_box.id, started_at: None, reason: None, notes: None },
        ).await.unwrap();
        inspections::create(
            &pool,
            CreateInspection { colony_id: colony.id, inspected_at: None, strength: Some("weak".into()), queen_present: None, laying_status: None, food_reserves: None, brood_status: None, pests_notes: None, observations: None, actions_taken: None, next_inspection_at: None },
        ).await.unwrap();

        let result = overview(&pool).await.unwrap();
        assert_eq!(result.occupied_boxes, 1);
        assert_eq!(result.free_boxes, 0);
        assert!(result.species_distribution.iter().any(|item| item.label == "Jataí" && item.count == 1));
        assert!(result.inspection_strengths.iter().any(|item| item.label == "weak" && item.count == 1));
        assert!(result.alerts.iter().any(|item| item.alert_type == "weak_colony"));
    }
}
