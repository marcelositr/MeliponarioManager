use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

const EVENT_TYPES: &[&str] = &[
    "swarming",
    "abandonment",
    "queen_loss",
    "attack",
    "pest",
    "recovery",
    "maintenance",
    "observation",
    "other",
];
const SEVERITIES: &[&str] = &["info", "attention", "critical"];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyEvent {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub box_id: Option<String>,
    pub box_code: Option<String>,
    pub event_type: String,
    pub occurred_at: String,
    pub title: Option<String>,
    pub details: Option<String>,
    pub severity: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateColonyEvent {
    pub colony_id: String,
    pub event_type: String,
    pub occurred_at: Option<String>,
    pub title: Option<String>,
    pub details: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub source_type: String,
    pub source_id: String,
    pub occurred_at: String,
    pub title: String,
    pub details: Option<String>,
    pub box_code: Option<String>,
    pub severity: String,
}

fn required(value: &str, field: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} é obrigatório.")));
    }
    Ok(value.to_owned())
}

fn optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

async fn colony_exists(pool: &SqlitePool, colony_id: &str) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)",
    )
    .bind(colony_id)
    .fetch_one(pool)
    .await?)
}

async fn box_at(
    pool: &SqlitePool,
    colony_id: &str,
    occurred_at: &str,
) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT box_id
         FROM colony_box_occupancies
         WHERE colony_id = ?
           AND started_at <= ?
           AND (ended_at IS NULL OR ended_at >= ?)
         ORDER BY started_at DESC
         LIMIT 1",
    )
    .bind(colony_id)
    .bind(occurred_at)
    .bind(occurred_at)
    .fetch_optional(pool)
    .await?)
}

async fn get(pool: &SqlitePool, id: &str) -> Result<ColonyEvent, AppError> {
    Ok(sqlx::query_as::<_, ColonyEvent>(
        "SELECT e.id, e.colony_id, c.code AS colony_code, e.box_id, b.code AS box_code,
                e.event_type, e.occurred_at, e.title, e.details, e.severity, e.created_at
         FROM colony_events e
         JOIN colonies c ON c.id = e.colony_id
         LEFT JOIN boxes b ON b.id = e.box_id
         WHERE e.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn create(
    pool: &SqlitePool,
    input: CreateColonyEvent,
) -> Result<ColonyEvent, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let event_type = required(&input.event_type, "Tipo do evento")?;
    if !EVENT_TYPES.contains(&event_type.as_str()) {
        return Err(AppError::Validation("Tipo de evento inválido.".to_owned()));
    }

    let severity = optional(&input.severity).unwrap_or_else(|| "info".to_owned());
    if !SEVERITIES.contains(&severity.as_str()) {
        return Err(AppError::Validation(
            "Nível do evento inválido. Use info, attention ou critical.".to_owned(),
        ));
    }

    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    let occurred_at = match optional(&input.occurred_at) {
        Some(value) => value,
        None => sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
            .fetch_one(pool)
            .await?,
    };
    let box_id = box_at(pool, &colony_id, &occurred_at).await?;
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO colony_events (
            id, colony_id, box_id, event_type, occurred_at, title, details, severity
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&colony_id)
    .bind(box_id)
    .bind(event_type)
    .bind(occurred_at)
    .bind(optional(&input.title))
    .bind(optional(&input.details))
    .bind(severity)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn list_events_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<ColonyEvent>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, ColonyEvent>(
        "SELECT e.id, e.colony_id, c.code AS colony_code, e.box_id, b.code AS box_code,
                e.event_type, e.occurred_at, e.title, e.details, e.severity, e.created_at
         FROM colony_events e
         JOIN colonies c ON c.id = e.colony_id
         LEFT JOIN boxes b ON b.id = e.box_id
         WHERE e.colony_id = ?
         ORDER BY e.occurred_at DESC, e.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn timeline_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<TimelineEntry>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, TimelineEntry>(
        "SELECT source_type, source_id, occurred_at, title, details, box_code, severity
         FROM (
            SELECT
                'event' AS source_type,
                e.id AS source_id,
                e.occurred_at,
                COALESCE(
                    e.title,
                    CASE e.event_type
                        WHEN 'swarming' THEN 'Enxameação'
                        WHEN 'abandonment' THEN 'Abandono'
                        WHEN 'queen_loss' THEN 'Perda de rainha'
                        WHEN 'attack' THEN 'Ataque'
                        WHEN 'pest' THEN 'Praga ou inimigo'
                        WHEN 'recovery' THEN 'Recuperação'
                        WHEN 'maintenance' THEN 'Manutenção'
                        WHEN 'observation' THEN 'Observação'
                        ELSE 'Outro evento'
                    END
                ) AS title,
                e.details,
                b.code AS box_code,
                e.severity
            FROM colony_events e
            LEFT JOIN boxes b ON b.id = e.box_id
            WHERE e.colony_id = ?

            UNION ALL

            SELECT
                'inspection',
                i.id,
                i.inspected_at,
                'Inspeção',
                COALESCE(i.observations, i.actions_taken),
                b.code,
                CASE WHEN i.strength = 'weak' THEN 'attention' ELSE 'info' END
            FROM inspections i
            LEFT JOIN boxes b ON b.id = i.box_id
            WHERE i.colony_id = ?

            UNION ALL

            SELECT
                'feeding',
                f.id,
                f.fed_at,
                'Alimentação: ' || f.food_type,
                COALESCE(f.response_notes, f.notes),
                b.code,
                'info'
            FROM feedings f
            LEFT JOIN boxes b ON b.id = f.box_id
            WHERE f.colony_id = ?

            UNION ALL

            SELECT
                'production',
                p.id,
                p.harvested_at,
                printf(
                    'Produção: %s · %g %s',
                    CASE p.product_type
                        WHEN 'honey' THEN 'Mel'
                        WHEN 'pollen' THEN 'Pólen'
                        WHEN 'propolis' THEN 'Própolis'
                        WHEN 'wax' THEN 'Cera'
                        WHEN 'cerumen' THEN 'Cerume'
                        ELSE 'Outro produto'
                    END,
                    p.quantity,
                    p.unit
                ),
                COALESCE(p.notes, p.purpose),
                b.code,
                'info'
            FROM production_records p
            LEFT JOIN boxes b ON b.id = p.box_id
            WHERE p.colony_id = ?

            UNION ALL

            SELECT
                'movement',
                m.id,
                m.moved_at,
                CASE m.movement_type
                    WHEN 'internal_transfer' THEN 'Transferência entre meliponários'
                    WHEN 'external_transfer' THEN 'Transferência para fora do plantel'
                    ELSE 'Transporte'
                END,
                COALESCE(tm.name, m.destination, m.notes),
                fb.code,
                'info'
            FROM colony_movements m
            LEFT JOIN meliponaries tm ON tm.id = m.to_meliponary_id
            LEFT JOIN boxes fb ON fb.id = m.from_box_id
            WHERE m.colony_id = ?

            UNION ALL

            SELECT
                'box_occupancy',
                o.id,
                o.started_at,
                CASE
                    WHEN o.reason IS NOT NULL AND TRIM(o.reason) <> '' THEN o.reason
                    ELSE 'Colônia colocada em caixa'
                END,
                o.notes,
                b.code,
                'info'
            FROM colony_box_occupancies o
            JOIN boxes b ON b.id = o.box_id
            WHERE o.colony_id = ?
         ) timeline
         ORDER BY occurred_at DESC, source_id DESC",
    )
    .bind(&colony_id)
    .bind(&colony_id)
    .bind(&colony_id)
    .bind(&colony_id)
    .bind(&colony_id)
    .bind(&colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM colony_events")
        .fetch_one(pool)
        .await?)
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

    async fn seed(pool: &SqlitePool) -> (String, String, String) {
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
        let box_one = repository::create_box(
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
        let box_two = repository::create_box(
            pool,
            CreateHiveBox {
                meliponary_id: meliponary.id.clone(),
                code: "CX-002".into(),
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
                box_id: box_one.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::place_colony(
            pool,
            PlaceColony {
                colony_id: colony.id.clone(),
                box_id: box_two.id.clone(),
                started_at: Some("2026-02-01 09:00:00".into()),
                reason: Some("Troca de caixa".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        (colony.id, box_one.id, box_two.id)
    }

    #[tokio::test]
    async fn retrospective_event_keeps_box_from_event_date() {
        let pool = test_pool().await;
        let (colony_id, box_one_id, _) = seed(&pool).await;
        let event = create(
            &pool,
            CreateColonyEvent {
                colony_id,
                event_type: "observation".into(),
                occurred_at: Some("2026-01-15 12:00:00".into()),
                title: Some("Observação antiga".into()),
                details: None,
                severity: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(event.box_id.as_deref(), Some(box_one_id.as_str()));
        assert_eq!(event.box_code.as_deref(), Some("CX-001"));
    }

    #[tokio::test]
    async fn timeline_combines_occupancy_inspection_and_event() {
        let pool = test_pool().await;
        let (colony_id, _, _) = seed(&pool).await;
        inspections::create(
            &pool,
            CreateInspection {
                colony_id: colony_id.clone(),
                inspected_at: Some("2026-02-10 10:00:00".into()),
                strength: Some("medium".into()),
                queen_present: Some(true),
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: Some("Tudo estável".into()),
                actions_taken: None,
                next_inspection_at: None,
            },
        )
        .await
        .unwrap();
        create(
            &pool,
            CreateColonyEvent {
                colony_id: colony_id.clone(),
                event_type: "attack".into(),
                occurred_at: Some("2026-02-15 08:00:00".into()),
                title: None,
                details: Some("Ataque observado na entrada".into()),
                severity: Some("attention".into()),
            },
        )
        .await
        .unwrap();

        let timeline = timeline_by_colony(&pool, &colony_id).await.unwrap();
        assert_eq!(timeline[0].source_type, "event");
        assert!(timeline.iter().any(|entry| entry.source_type == "inspection"));
        assert!(timeline.iter().any(|entry| entry.source_type == "box_occupancy"));
    }

    #[tokio::test]
    async fn event_rejects_unknown_type() {
        let pool = test_pool().await;
        let (colony_id, _, _) = seed(&pool).await;
        let result = create(
            &pool,
            CreateColonyEvent {
                colony_id,
                event_type: "teleportation".into(),
                occurred_at: None,
                title: None,
                details: None,
                severity: None,
            },
        )
        .await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
