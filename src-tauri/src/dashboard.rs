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
pub async fn overview(p: &SqlitePool) -> Result<DashboardOverview, AppError> {
    let colony_statuses=sqlx::query_as::<_,DashboardCount>("SELECT CASE WHEN status IN('weak','recovering') THEN 'active' ELSE status END label,COUNT(*) count FROM colonies GROUP BY CASE WHEN status IN('weak','recovering') THEN 'active' ELSE status END ORDER BY count DESC,label").fetch_all(p).await?;
    let species_distribution=sqlx::query_as::<_,DashboardCount>("SELECT s.common_name label,COUNT(c.id) count FROM species s JOIN colonies c ON c.species_id=s.id GROUP BY s.id,s.common_name ORDER BY count DESC,s.common_name COLLATE NOCASE").fetch_all(p).await?;
    let inspection_strengths=sqlx::query_as::<_,DashboardCount>("WITH latest AS(SELECT i.colony_id,i.strength,ROW_NUMBER()OVER(PARTITION BY i.colony_id ORDER BY i.inspected_at DESC,i.created_at DESC,i.id DESC)rn FROM inspections i WHERE i.voided_at IS NULL)SELECT COALESCE(l.strength,'unknown')label,COUNT(*)count FROM colonies c LEFT JOIN latest l ON l.colony_id=c.id AND l.rn=1 WHERE c.status IN('active','weak','recovering') GROUP BY COALESCE(l.strength,'unknown') ORDER BY count DESC,label").fetch_all(p).await?;
    let occupied_boxes =
        sqlx::query_scalar("SELECT COUNT(*) FROM colony_box_occupancies WHERE ended_at IS NULL")
            .fetch_one(p)
            .await?;
    let free_boxes=sqlx::query_scalar("SELECT COUNT(*) FROM boxes b WHERE b.status='active' AND NOT EXISTS(SELECT 1 FROM colony_box_occupancies o WHERE o.box_id=b.id AND o.ended_at IS NULL)").fetch_one(p).await?;
    let recent_production=sqlx::query_as::<_,RecentProduction>("SELECT c.code colony_code,p.product_type,p.quantity,p.unit,p.harvested_at FROM production_records p JOIN colonies c ON c.id=p.colony_id WHERE p.voided_at IS NULL ORDER BY p.harvested_at DESC,p.created_at DESC LIMIT 5").fetch_all(p).await?;
    let recent_movements=sqlx::query_as::<_,RecentMovement>("SELECT c.code colony_code,m.movement_type,m.moved_at,COALESCE(tm.name,m.destination)destination FROM colony_movements m JOIN colonies c ON c.id=m.colony_id LEFT JOIN meliponaries tm ON tm.id=m.to_meliponary_id WHERE m.voided_at IS NULL AND m.reversed_at IS NULL ORDER BY m.moved_at DESC,m.created_at DESC LIMIT 5").fetch_all(p).await?;
    Ok(DashboardOverview {
        colony_statuses,
        species_distribution,
        inspection_strengths,
        occupied_boxes,
        free_boxes,
        alerts: alerts::list(p).await?,
        recent_production,
        recent_movements,
    })
}
#[tauri::command]
pub async fn get_dashboard_overview(
    pool: State<'_, SqlitePool>,
) -> Result<DashboardOverview, String> {
    overview(&pool).await.map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        inspections::{self, CreateInspection},
        record_corrections::{self, VoidRecord},
        repository,
    };
    use sqlx::sqlite::SqlitePoolOptions;
    async fn p() -> SqlitePool {
        let p = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&p).await.unwrap();
        p
    }
    async fn seed(p: &SqlitePool) -> (String, String) {
        let m = repository::create_meliponary(
            p,
            CreateMeliponary {
                name: "Principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let s = repository::create_species(
            p,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let b = repository::create_box(
            p,
            CreateHiveBox {
                meliponary_id: m.id.clone(),
                code: "CX-001".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let c = repository::create_colony(
            p,
            CreateColony {
                meliponary_id: m.id,
                species_id: s.id,
                code: "JAT-001".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: None,
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::place_colony(
            p,
            PlaceColony {
                colony_id: c.id.clone(),
                box_id: b.id.clone(),
                started_at: None,
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        (c.id, b.id)
    }
    #[tokio::test]
    async fn overview_derives_current_plantel_state() {
        let p = p().await;
        let (c, _) = seed(&p).await;
        inspections::create(
            &p,
            CreateInspection {
                colony_id: c.clone(),
                inspected_at: None,
                strength: Some("weak".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("UPDATE colonies SET status='weak' WHERE id=?")
            .bind(&c)
            .execute(&p)
            .await
            .unwrap();
        let r = overview(&p).await.unwrap();
        assert_eq!(r.occupied_boxes, 1);
        assert_eq!(r.free_boxes, 0);
        assert!(r
            .colony_statuses
            .iter()
            .any(|x| x.label == "active" && x.count == 1));
        assert!(r
            .inspection_strengths
            .iter()
            .any(|x| x.label == "weak" && x.count == 1));
        assert!(r.alerts.iter().any(|x| x.alert_type == "weak_colony"));
    }
    #[tokio::test]
    async fn voided_inspection_and_production_are_ignored_operationally() {
        let p = p().await;
        let (c, _) = seed(&p).await;
        let i = inspections::create(
            &p,
            CreateInspection {
                colony_id: c.clone(),
                inspected_at: Some("2026-01-01 10:00:00".into()),
                strength: Some("weak".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: None,
            },
        )
        .await
        .unwrap();
        record_corrections::void_inspection(
            &p,
            VoidRecord {
                id: i.id,
                reason: "Erro".into(),
            },
        )
        .await
        .unwrap();
        sqlx::query("INSERT INTO production_records(id,colony_id,harvested_at,product_type,quantity,unit,voided_at,void_reason)VALUES('p1',?,'2026-01-01 10:00:00','honey',1,'ml','2026-01-02 10:00:00','Erro')").bind(&c).execute(&p).await.unwrap();
        let r = overview(&p).await.unwrap();
        assert!(!r.inspection_strengths.iter().any(|x| x.label == "weak"));
        assert!(r.recent_production.is_empty());
    }
}
