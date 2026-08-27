use crate::repository::AppError;
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
    Ok(sqlx::query_as::<_, Alert>(
        "WITH latest_inspection AS (
            SELECT
                i.*,
                ROW_NUMBER() OVER (
                    PARTITION BY i.colony_id
                    ORDER BY i.inspected_at DESC, i.created_at DESC, i.id DESC
                ) AS rn
            FROM inspections i
         ),
         latest_feeding AS (
            SELECT
                f.*,
                ROW_NUMBER() OVER (
                    PARTITION BY f.colony_id
                    ORDER BY f.fed_at DESC, f.created_at DESC, f.id DESC
                ) AS rn
            FROM feedings f
         )
         SELECT alert_key, colony_id, colony_code, alert_type, severity, due_at, title, details
         FROM (
            SELECT
                'inspection_due:' || li.id AS alert_key,
                c.id AS colony_id,
                c.code AS colony_code,
                'inspection_due' AS alert_type,
                'attention' AS severity,
                li.next_inspection_at AS due_at,
                'Inspeção pendente' AS title,
                printf('Revisão prevista para %s.', li.next_inspection_at) AS details
            FROM latest_inspection li
            JOIN colonies c ON c.id = li.colony_id
            WHERE li.rn = 1
              AND li.next_inspection_at IS NOT NULL
              AND datetime(li.next_inspection_at) <= CURRENT_TIMESTAMP
              AND c.status IN ('active', 'weak', 'recovering')

            UNION ALL

            SELECT
                'feeding_due:' || lf.id AS alert_key,
                c.id AS colony_id,
                c.code AS colony_code,
                'feeding_due' AS alert_type,
                'attention' AS severity,
                lf.next_feeding_at AS due_at,
                'Alimentação pendente' AS title,
                printf(
                    'Próxima alimentação prevista para %s. Último alimento: %s.',
                    lf.next_feeding_at,
                    lf.food_type
                ) AS details
            FROM latest_feeding lf
            JOIN colonies c ON c.id = lf.colony_id
            WHERE lf.rn = 1
              AND lf.next_feeding_at IS NOT NULL
              AND datetime(lf.next_feeding_at) <= CURRENT_TIMESTAMP
              AND c.status IN ('active', 'weak', 'recovering')

            UNION ALL

            SELECT
                'weak_colony:' || c.id AS alert_key,
                c.id AS colony_id,
                c.code AS colony_code,
                'weak_colony' AS alert_type,
                'attention' AS severity,
                NULL AS due_at,
                'Colônia fraca' AS title,
                CASE
                    WHEN li.strength = 'weak' THEN 'A última inspeção classificou a colônia como fraca.'
                    ELSE 'A colônia está marcada como fraca no plantel.'
                END AS details
            FROM colonies c
            LEFT JOIN latest_inspection li
              ON li.colony_id = c.id AND li.rn = 1
            WHERE c.status IN ('active', 'weak', 'recovering')
              AND (c.status = 'weak' OR li.strength = 'weak')
         ) alerts
         ORDER BY
            CASE alerts.severity
                WHEN 'critical' THEN 0
                WHEN 'attention' THEN 1
                ELSE 2
            END,
            COALESCE(alerts.due_at, '9999-12-31 23:59:59'),
            alerts.colony_code COLLATE NOCASE,
            alerts.alert_type",
    )
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

    async fn seed(pool: &SqlitePool) -> String {
        let meliponary = repository::create_meliponary(
            pool,
            CreateMeliponary {
                name: "Meliponário principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let species = repository::create_species(
            pool,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let hive_box = repository::create_box(
            pool,
            CreateHiveBox {
                meliponary_id: meliponary.id.clone(),
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
            pool,
            CreateColony {
                meliponary_id: meliponary.id,
                species_id: species.id,
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
            pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: hive_box.id,
                started_at: Some("2000-01-01 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        colony.id
    }

    #[tokio::test]
    async fn overdue_inspection_creates_alert() {
        let pool = test_pool().await;
        let colony_id = seed(&pool).await;

        inspections::create(
            &pool,
            CreateInspection {
                colony_id,
                inspected_at: Some("2000-01-10 10:00:00".into()),
                strength: Some("medium".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2000-01-20 10:00:00".into()),
            },
        )
        .await
        .unwrap();

        let alerts = list(&pool).await.unwrap();
        assert!(alerts
            .iter()
            .any(|alert| alert.alert_type == "inspection_due"));
    }

    #[tokio::test]
    async fn newer_inspection_supersedes_old_schedule() {
        let pool = test_pool().await;
        let colony_id = seed(&pool).await;

        inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2000-01-10 10:00:00".into()),
                strength: Some("medium".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2000-01-20 10:00:00".into()),
            },
        )
        .await
        .unwrap();

        inspections::create(
            &pool,
            CreateInspection {
                colony_id,
                inspected_at: Some("2026-01-10 10:00:00".into()),
                strength: Some("strong".into()),
                queen_present: None,
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2999-01-20 10:00:00".into()),
            },
        )
        .await
        .unwrap();

        let alerts = list(&pool).await.unwrap();
        assert!(!alerts
            .iter()
            .any(|alert| alert.alert_type == "inspection_due"));
    }

    #[tokio::test]
    async fn overdue_feeding_creates_alert() {
        let pool = test_pool().await;
        let colony_id = seed(&pool).await;

        feeding::create(
            &pool,
            CreateFeeding {
                colony_id,
                fed_at: Some("2000-02-01 12:00:00".into()),
                food_type: "Xarope 1:1".into(),
                quantity: Some(50.0),
                unit: Some("ml".into()),
                response_notes: None,
                notes: None,
                next_feeding_at: Some("2000-02-08 12:00:00".into()),
            },
        )
        .await
        .unwrap();

        let alerts = list(&pool).await.unwrap();
        assert!(alerts.iter().any(|alert| alert.alert_type == "feeding_due"));
    }

    #[tokio::test]
    async fn latest_weak_inspection_creates_single_weak_alert() {
        let pool = test_pool().await;
        let colony_id = seed(&pool).await;

        inspections::create(
            &pool,
            CreateInspection {
                colony_id,
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

        let alerts = list(&pool).await.unwrap();
        assert_eq!(
            alerts
                .iter()
                .filter(|alert| alert.alert_type == "weak_colony")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn inactive_colony_is_not_alerted() {
        let pool = test_pool().await;
        let colony_id = seed(&pool).await;

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
                next_inspection_at: Some("2000-01-20 10:00:00".into()),
            },
        )
        .await
        .unwrap();

        sqlx::query("UPDATE colonies SET status = 'inactive' WHERE id = ?")
            .bind(&colony_id)
            .execute(&pool)
            .await
            .unwrap();

        let alerts = list(&pool).await.unwrap();
        assert!(alerts.is_empty());
    }
}
