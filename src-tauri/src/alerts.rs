use crate::{repository::AppError, time};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub alert_key: String,
    pub meliponary_id: String,
    pub colony_id: Option<String>,
    pub colony_code: Option<String>,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub task_id: Option<String>,
    pub alert_type: String,
    pub severity: String,
    pub due_at: Option<String>,
    pub title: String,
    pub details: Option<String>,
    pub recommended_action: String,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Alert>, AppError> {
    list_for(pool, None).await
}

pub async fn list_for(
    pool: &SqlitePool,
    meliponary_id: Option<&str>,
) -> Result<Vec<Alert>, AppError> {
    let now = time::local_now(pool).await?;
    Ok(sqlx::query_as::<_, Alert>(
        "WITH latest_inspection AS (
            SELECT i.colony_id,i.strength,
                   ROW_NUMBER() OVER (
                     PARTITION BY i.colony_id
                     ORDER BY i.inspected_at DESC,i.created_at DESC,i.id DESC
                   ) rn
            FROM inspections i
            WHERE i.voided_at IS NULL
         ), task_alerts AS (
            SELECT 'task:'||t.id alert_key,
                   t.meliponary_id,
                   t.colony_id,
                   c.code colony_code,
                   t.box_id,
                   b.code box_code,
                   t.id task_id,
                   CASE t.task_type
                     WHEN 'inspection' THEN 'inspection_due'
                     WHEN 'feeding' THEN 'feeding_due'
                     WHEN 'maintenance' THEN 'maintenance_due'
                   END alert_type,
                   CASE WHEN t.priority='critical' THEN 'critical' ELSE 'attention' END severity,
                   t.scheduled_for due_at,
                   CASE t.task_type
                     WHEN 'inspection' THEN 'Inspeção atrasada'
                     WHEN 'feeding' THEN 'Alimentação atrasada'
                     WHEN 'maintenance' THEN 'Manutenção atrasada'
                   END title,
                   t.description details,
                   CASE t.task_type
                     WHEN 'inspection' THEN 'register_inspection'
                     WHEN 'feeding' THEN 'register_feeding'
                     WHEN 'maintenance' THEN 'register_maintenance'
                   END recommended_action
            FROM scheduled_tasks t
            JOIN meliponaries tm ON tm.id=t.meliponary_id AND tm.archived_at IS NULL
            LEFT JOIN colonies c ON c.id=t.colony_id
            LEFT JOIN boxes b ON b.id=t.box_id
            WHERE t.status='pending'
              AND t.task_type IN('inspection','feeding','maintenance')
              AND t.scheduled_for < ?
              AND (? IS NULL OR t.meliponary_id=?)
         ), weak_alerts AS (
            SELECT 'weak:'||c.id alert_key,
                   c.meliponary_id,
                   c.id colony_id,
                   c.code colony_code,
                   o.box_id,
                   b.code box_code,
                   NULL task_id,
                   'weak_colony' alert_type,
                   'attention' severity,
                   NULL due_at,
                   'Colônia fraca' title,
                   'A última inspeção válida classificou a colônia como fraca.' details,
                   'register_inspection' recommended_action
            FROM colonies c
            JOIN meliponaries cm ON cm.id=c.meliponary_id AND cm.archived_at IS NULL
            JOIN latest_inspection li ON li.colony_id=c.id AND li.rn=1
            LEFT JOIN colony_box_occupancies o ON o.colony_id=c.id AND o.ended_at IS NULL
            LEFT JOIN boxes b ON b.id=o.box_id
            WHERE c.status IN('active','weak','recovering')
              AND li.strength='weak'
              AND (? IS NULL OR c.meliponary_id=?)
         ), all_alerts AS (
            SELECT alert_key,meliponary_id,colony_id,colony_code,box_id,box_code,task_id,
                   alert_type,severity,due_at,title,details,recommended_action
            FROM task_alerts
            UNION ALL
            SELECT alert_key,meliponary_id,colony_id,colony_code,box_id,box_code,task_id,
                   alert_type,severity,due_at,title,details,recommended_action
            FROM weak_alerts
         )
         SELECT alert_key,meliponary_id,colony_id,colony_code,box_id,box_code,task_id,
                alert_type,severity,due_at,title,details,recommended_action
         FROM all_alerts
         ORDER BY CASE severity WHEN 'critical' THEN 0 ELSE 1 END,
                  COALESCE(due_at,'9999-12-31 23:59:59'),alert_key",
    )
    .bind(&now)
    .bind(meliponary_id)
    .bind(meliponary_id)
    .bind(meliponary_id)
    .bind(meliponary_id)
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
        agenda::{self, CreateTask},
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        inspections::{self, CreateInspection},
        master_data::{self, EntityAction},
        record_corrections::{self, VoidRecord},
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

    #[tokio::test]
    async fn overdue_task_is_the_single_due_alert_source() {
        let (pool, meliponary_id, colony_id, _) = seeded().await;
        inspections::create(
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
                next_inspection_at: Some("2026-01-11 10:00:00".into()),
            },
        )
        .await
        .unwrap();
        assert!(list(&pool)
            .await
            .unwrap()
            .iter()
            .all(|item| item.alert_type != "inspection_due"));
        agenda::reconcile_inspection(&pool, &colony_id)
            .await
            .unwrap();
        let alerts = list(&pool).await.unwrap();
        assert_eq!(
            alerts
                .iter()
                .filter(|item| item.alert_type == "inspection_due")
                .count(),
            1
        );
        assert_eq!(
            alerts
                .iter()
                .find(|item| item.alert_type == "inspection_due")
                .unwrap()
                .meliponary_id,
            meliponary_id
        );
    }

    #[tokio::test]
    async fn completed_or_cancelled_task_does_not_alert() {
        let (pool, meliponary_id, colony_id, _) = seeded().await;
        let old = "2026-01-10 10:00:00".to_owned();
        let completed = agenda::create_manual(
            &pool,
            CreateTask {
                meliponary_id: meliponary_id.clone(),
                colony_id: Some(colony_id.clone()),
                box_id: None,
                task_type: "inspection".into(),
                title: "Inspecionar".into(),
                description: None,
                scheduled_for: old.clone(),
                priority: None,
            },
        )
        .await;
        assert!(
            completed.is_err(),
            "manual past guard should reject distant history"
        );
        sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,colony_id,task_type,title,scheduled_for,status,completed_at) VALUES('t1',?,?,'inspection','Inspecionar',?,'completed',?)")
            .bind(&meliponary_id).bind(&colony_id).bind(&old).bind(&old).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,colony_id,task_type,title,scheduled_for,status,cancelled_at,cancellation_reason) VALUES('t2',?,?,'inspection','Inspecionar',?,'cancelled',?,'Sem necessidade')")
            .bind(&meliponary_id).bind(&colony_id).bind(&old).bind(&old).execute(&pool).await.unwrap();
        assert!(list(&pool)
            .await
            .unwrap()
            .iter()
            .all(|item| item.task_id.is_none()));
    }

    #[tokio::test]
    async fn weak_colony_remains_derived_from_latest_valid_inspection() {
        let (pool, _, colony_id, _) = seeded().await;
        let old = inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-01-10 10:00:00".into()),
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
        inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-02-10 10:00:00".into()),
                strength: Some("strong".into()),
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
        assert!(list(&pool)
            .await
            .unwrap()
            .iter()
            .all(|item| item.alert_type != "weak_colony"));
        record_corrections::void_inspection(
            &pool,
            VoidRecord {
                id: old.id,
                reason: "Registro antigo inválido".into(),
            },
        )
        .await
        .unwrap();
        assert!(list(&pool)
            .await
            .unwrap()
            .iter()
            .all(|item| item.alert_type != "weak_colony"));
    }

    #[tokio::test]
    async fn archived_meliponary_is_excluded_from_operational_alerts_without_losing_history() {
        let (pool, meliponary_id, colony_id, _) = seeded().await;
        inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-01-10 10:00:00".into()),
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
        sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,colony_id,task_type,title,scheduled_for) VALUES('manual-overdue',?,?,'inspection','Conferir colônia','2026-01-11 10:00:00')")
            .bind(&meliponary_id)
            .bind(&colony_id)
            .execute(&pool)
            .await
            .unwrap();

        let before = list(&pool).await.unwrap();
        assert!(before.iter().any(|item| item.alert_type == "weak_colony"));
        assert!(before
            .iter()
            .any(|item| item.task_id.as_deref() == Some("manual-overdue")));

        master_data::archive_meliponary(
            &pool,
            EntityAction {
                id: meliponary_id.clone(),
                reason: "Fora de operação".into(),
            },
        )
        .await
        .unwrap();

        assert!(list(&pool).await.unwrap().is_empty());
        assert!(list_for(&pool, Some(&meliponary_id))
            .await
            .unwrap()
            .is_empty());
        let kept_inspections: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM inspections WHERE colony_id=?")
                .bind(&colony_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(kept_inspections, 1);
        let kept_task_status: String =
            sqlx::query_scalar("SELECT status FROM scheduled_tasks WHERE id='manual-overdue'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(kept_task_status, "pending");

        master_data::reactivate_meliponary(
            &pool,
            EntityAction {
                id: meliponary_id.clone(),
                reason: "Retorno à operação".into(),
            },
        )
        .await
        .unwrap();
        let after = list_for(&pool, Some(&meliponary_id)).await.unwrap();
        assert!(after.iter().any(|item| item.alert_type == "weak_colony"));
        assert!(after
            .iter()
            .any(|item| item.task_id.as_deref() == Some("manual-overdue")));
    }

    #[tokio::test]
    async fn active_meliponary_filter_scopes_alerts() {
        let (pool, meliponary_id, colony_id, _) = seeded().await;
        sqlx::query("INSERT INTO scheduled_tasks(id,meliponary_id,colony_id,task_type,title,scheduled_for) VALUES('t1',?,?,'inspection','Inspecionar','2026-01-01 10:00:00')")
            .bind(&meliponary_id).bind(&colony_id).execute(&pool).await.unwrap();
        assert_eq!(
            list_for(&pool, Some(&meliponary_id)).await.unwrap().len(),
            1
        );
        assert!(list_for(&pool, Some("missing")).await.unwrap().is_empty());
    }
}
