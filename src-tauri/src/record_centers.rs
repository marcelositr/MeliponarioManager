use crate::{media::InspectionPhoto, repository::AppError, time};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyRecordCenter {
    pub id: String,
    pub code: String,
    pub species_name: String,
    pub meliponary_id: String,
    pub meliponary_name: String,
    pub current_box_code: Option<String>,
    pub status: String,
    pub origin_type: String,
    pub origin_notes: Option<String>,
    pub installed_at: Option<String>,
    pub latest_inspection_at: Option<String>,
    pub latest_strength: Option<String>,
    pub latest_feeding_at: Option<String>,
    pub pending_tasks: i64,
    pub overdue_tasks: i64,
    pub current_alerts: i64,
    pub next_task_title: Option<String>,
    pub next_task_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BoxRecordCenter {
    pub id: String,
    pub code: String,
    pub meliponary_id: String,
    pub meliponary_name: String,
    pub status: String,
    pub current_colony_code: Option<String>,
    pub model: Option<String>,
    pub material: Option<String>,
    pub location_note: Option<String>,
    pub occupancy_records: i64,
    pub maintenance_records: i64,
    pub pending_tasks: i64,
    pub next_maintenance_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MeliponaryRecordCenter {
    pub id: String,
    pub name: String,
    pub responsible_name: Option<String>,
    pub location: Option<String>,
    pub archived_at: Option<String>,
    pub colonies: i64,
    pub boxes: i64,
    pub pending_tasks: i64,
    pub overdue_tasks: i64,
    pub alerts: i64,
    pub recent_production_records: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BoxOccupancyHistory {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub corrected_at: Option<String>,
}

pub async fn colony(pool: &SqlitePool, colony_id: &str) -> Result<ColonyRecordCenter, AppError> {
    let now = time::local_now(pool).await?;
    Ok(sqlx::query_as::<_, ColonyRecordCenter>(
        "WITH latest_inspection AS (
            SELECT inspected_at,strength FROM inspections
            WHERE colony_id=? AND voided_at IS NULL
            ORDER BY inspected_at DESC,created_at DESC,id DESC LIMIT 1
         ), latest_feeding AS (
            SELECT fed_at FROM feedings
            WHERE colony_id=? AND voided_at IS NULL
            ORDER BY fed_at DESC,created_at DESC,id DESC LIMIT 1
         ), next_task AS (
            SELECT title,scheduled_for FROM scheduled_tasks
            WHERE colony_id=? AND status='pending'
            ORDER BY scheduled_for,created_at,id LIMIT 1
         )
         SELECT c.id,c.code,s.common_name species_name,c.meliponary_id,m.name meliponary_name,
                b.code current_box_code,c.status,c.origin_type,c.origin_notes,c.installed_at,
                (SELECT inspected_at FROM latest_inspection) latest_inspection_at,
                (SELECT strength FROM latest_inspection) latest_strength,
                (SELECT fed_at FROM latest_feeding) latest_feeding_at,
                (SELECT COUNT(*) FROM scheduled_tasks t WHERE t.colony_id=c.id AND t.status='pending') pending_tasks,
                (SELECT COUNT(*) FROM scheduled_tasks t WHERE t.colony_id=c.id AND t.status='pending' AND t.scheduled_for<?) overdue_tasks,
                (SELECT COUNT(*) FROM scheduled_tasks t WHERE t.colony_id=c.id AND t.status='pending' AND t.scheduled_for<? AND t.task_type IN('inspection','feeding','maintenance'))
                  + CASE WHEN (SELECT strength FROM latest_inspection)='weak' AND c.status IN('active','weak','recovering') THEN 1 ELSE 0 END current_alerts,
                (SELECT title FROM next_task) next_task_title,
                (SELECT scheduled_for FROM next_task) next_task_at
         FROM colonies c
         JOIN species s ON s.id=c.species_id
         JOIN meliponaries m ON m.id=c.meliponary_id
         LEFT JOIN colony_box_occupancies o ON o.colony_id=c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id=o.box_id
         WHERE c.id=?",
    )
    .bind(colony_id)
    .bind(colony_id)
    .bind(colony_id)
    .bind(&now)
    .bind(&now)
    .bind(colony_id)
    .fetch_one(pool)
    .await?)
}

pub async fn box_center(pool: &SqlitePool, box_id: &str) -> Result<BoxRecordCenter, AppError> {
    Ok(sqlx::query_as::<_, BoxRecordCenter>(
        "SELECT b.id,b.code,b.meliponary_id,m.name meliponary_name,b.status,
                c.code current_colony_code,b.model,b.material,b.location_note,
                (SELECT COUNT(*) FROM colony_box_occupancies o2 WHERE o2.box_id=b.id) occupancy_records,
                (SELECT COUNT(*) FROM box_maintenance_records r WHERE r.box_id=b.id AND r.voided_at IS NULL) maintenance_records,
                (SELECT COUNT(*) FROM scheduled_tasks t WHERE t.box_id=b.id AND t.status='pending') pending_tasks,
                (SELECT scheduled_for FROM scheduled_tasks t
                  WHERE t.box_id=b.id AND t.status='pending' AND t.task_type='maintenance'
                  ORDER BY t.scheduled_for,t.created_at,t.id LIMIT 1) next_maintenance_at
         FROM boxes b
         JOIN meliponaries m ON m.id=b.meliponary_id
         LEFT JOIN colony_box_occupancies o ON o.box_id=b.id AND o.ended_at IS NULL
         LEFT JOIN colonies c ON c.id=o.colony_id
         WHERE b.id=?",
    )
    .bind(box_id)
    .fetch_one(pool)
    .await?)
}

pub async fn meliponary(
    pool: &SqlitePool,
    meliponary_id: &str,
) -> Result<MeliponaryRecordCenter, AppError> {
    let now = time::local_now(pool).await?;
    let seven_days_ago: String =
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', ?, '-7 days')")
            .bind(&now)
            .fetch_one(pool)
            .await?;
    Ok(sqlx::query_as::<_, MeliponaryRecordCenter>(
        "WITH latest_inspection AS (
            SELECT i.colony_id,i.strength,
                   ROW_NUMBER() OVER(PARTITION BY i.colony_id ORDER BY i.inspected_at DESC,i.created_at DESC,i.id DESC) rn
            FROM inspections i WHERE i.voided_at IS NULL
         )
         SELECT m.id,m.name,m.responsible_name,m.location,m.archived_at,
                (SELECT COUNT(*) FROM colonies c WHERE c.meliponary_id=m.id) colonies,
                (SELECT COUNT(*) FROM boxes b WHERE b.meliponary_id=m.id) boxes,
                (SELECT COUNT(*) FROM scheduled_tasks t WHERE t.meliponary_id=m.id AND t.status='pending') pending_tasks,
                (SELECT COUNT(*) FROM scheduled_tasks t WHERE t.meliponary_id=m.id AND t.status='pending' AND t.scheduled_for<?) overdue_tasks,
                (SELECT COUNT(*) FROM scheduled_tasks t WHERE t.meliponary_id=m.id AND t.status='pending' AND t.scheduled_for<? AND t.task_type IN('inspection','feeding','maintenance'))
                  + (SELECT COUNT(*) FROM colonies c JOIN latest_inspection li ON li.colony_id=c.id AND li.rn=1
                     WHERE c.meliponary_id=m.id AND c.status IN('active','weak','recovering') AND li.strength='weak') alerts,
                (SELECT COUNT(*) FROM production_records p JOIN colonies c ON c.id=p.colony_id
                  WHERE c.meliponary_id=m.id AND p.voided_at IS NULL AND p.harvested_at>=?) recent_production_records
         FROM meliponaries m WHERE m.id=?",
    )
    .bind(&now)
    .bind(&now)
    .bind(&seven_days_ago)
    .bind(meliponary_id)
    .fetch_one(pool)
    .await?)
}

pub async fn box_occupancies(
    pool: &SqlitePool,
    box_id: &str,
) -> Result<Vec<BoxOccupancyHistory>, AppError> {
    Ok(sqlx::query_as::<_, BoxOccupancyHistory>(
        "SELECT o.id,o.colony_id,c.code colony_code,o.started_at,o.ended_at,o.reason,o.notes,o.corrected_at
         FROM colony_box_occupancies o
         JOIN colonies c ON c.id=o.colony_id
         WHERE o.box_id=?
         ORDER BY o.started_at DESC,o.id DESC",
    )
    .bind(box_id)
    .fetch_all(pool)
    .await?)
}

pub async fn box_photos(pool: &SqlitePool, box_id: &str) -> Result<Vec<InspectionPhoto>, AppError> {
    Ok(sqlx::query_as::<_, InspectionPhoto>(
        "SELECT p.id,p.inspection_id,i.colony_id,c.code colony_code,p.relative_path,
                p.original_name,p.mime_type,p.byte_size,p.captured_at,p.notes,p.created_at
         FROM inspection_photos p
         JOIN inspections i ON i.id=p.inspection_id
         JOIN colonies c ON c.id=i.colony_id
         WHERE i.box_id=?
         ORDER BY p.captured_at DESC,p.created_at DESC,p.id DESC",
    )
    .bind(box_id)
    .fetch_all(pool)
    .await?)
}

#[tauri::command]
pub async fn get_colony_record_center(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<ColonyRecordCenter, String> {
    colony(&pool, &colony_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_box_record_center(
    pool: State<'_, SqlitePool>,
    box_id: String,
) -> Result<BoxRecordCenter, String> {
    box_center(&pool, &box_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_meliponary_record_center(
    pool: State<'_, SqlitePool>,
    meliponary_id: String,
) -> Result<MeliponaryRecordCenter, String> {
    meliponary(&pool, &meliponary_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_box_occupancies(
    pool: State<'_, SqlitePool>,
    box_id: String,
) -> Result<Vec<BoxOccupancyHistory>, String> {
    box_occupancies(&pool, &box_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_box_context_photos(
    pool: State<'_, SqlitePool>,
    box_id: String,
) -> Result<Vec<InspectionPhoto>, String> {
    box_photos(&pool, &box_id)
        .await
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agenda::{self, CreateTask},
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
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
                responsible_name: Some("Responsável".into()),
                location: Some("Sítio".into()),
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
                model: Some("INPA".into()),
                material: Some("Madeira".into()),
                location_note: Some("Setor A".into()),
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
                origin_notes: Some("Origem conhecida".into()),
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
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();
        (pool, mel.id, colony.id, box_record.id)
    }

    #[tokio::test]
    async fn record_centers_derive_context_without_new_fact_tables() {
        let (pool, meliponary_id, colony_id, box_id) = seeded().await;
        agenda::create_manual(
            &pool,
            CreateTask {
                meliponary_id: meliponary_id.clone(),
                colony_id: Some(colony_id.clone()),
                box_id: None,
                task_type: "inspection".into(),
                title: "Inspecionar JAT-001".into(),
                description: None,
                scheduled_for: time::local_now(&pool).await.unwrap(),
                priority: None,
            },
        )
        .await
        .unwrap();
        let colony_record = colony(&pool, &colony_id).await.unwrap();
        assert_eq!(colony_record.code, "JAT-001");
        assert_eq!(colony_record.current_box_code.as_deref(), Some("CX-001"));
        assert_eq!(colony_record.pending_tasks, 1);
        let box_record = box_center(&pool, &box_id).await.unwrap();
        assert_eq!(box_record.current_colony_code.as_deref(), Some("JAT-001"));
        assert_eq!(box_record.occupancy_records, 1);
        let meliponary_record = meliponary(&pool, &meliponary_id).await.unwrap();
        assert_eq!(meliponary_record.colonies, 1);
        assert_eq!(meliponary_record.boxes, 1);
    }
}
