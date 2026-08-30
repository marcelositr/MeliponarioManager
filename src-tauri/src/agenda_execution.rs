use crate::{agenda, operational, repository::AppError, time};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

const MAINTENANCE_TYPES: &[&str] = &[
    "cleaning",
    "repair",
    "painting",
    "waterproofing",
    "roof",
    "entrance",
    "internal_structure",
    "inspection",
    "other",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompletion {
    pub task: agenda::ScheduledTask,
    pub fact_type: String,
    pub fact_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteInspectionTask {
    pub task_id: String,
    pub inspected_at: Option<String>,
    pub strength: Option<String>,
    pub queen_present: Option<bool>,
    pub laying_status: Option<String>,
    pub food_reserves: Option<String>,
    pub brood_status: Option<String>,
    pub pests_notes: Option<String>,
    pub observations: Option<String>,
    pub actions_taken: Option<String>,
    pub next_inspection_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteFeedingTask {
    pub task_id: String,
    pub fed_at: Option<String>,
    pub food_type: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub response_notes: Option<String>,
    pub notes: Option<String>,
    pub next_feeding_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteMaintenanceTask {
    pub task_id: String,
    pub maintained_at: Option<String>,
    pub maintenance_type: String,
    pub description: Option<String>,
    pub performed_by: Option<String>,
    pub cost: Option<f64>,
    pub next_maintenance_at: Option<String>,
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

fn inspection_strength(value: &Option<String>) -> Result<String, AppError> {
    let value = optional(value).unwrap_or_else(|| "unknown".to_owned());
    match value.as_str() {
        "strong" | "medium" | "weak" | "unknown" => Ok(value),
        _ => Err(AppError::Validation(
            "Força da colônia inválida. Use strong, medium, weak ou unknown.".to_owned(),
        )),
    }
}

fn feeding_unit(quantity: Option<f64>, unit: &Option<String>) -> Result<Option<String>, AppError> {
    let unit = optional(unit);
    match (quantity, unit) {
        (None, None) => Ok(None),
        (Some(value), Some(unit)) if value > 0.0 && value.is_finite() => Ok(Some(unit)),
        (Some(_), None) => Err(AppError::Validation(
            "Informe a unidade quando registrar uma quantidade.".to_owned(),
        )),
        (None, Some(_)) => Err(AppError::Validation(
            "Informe a quantidade quando registrar uma unidade.".to_owned(),
        )),
        (Some(_), Some(_)) => Err(AppError::Validation(
            "A quantidade precisa ser maior que zero.".to_owned(),
        )),
    }
}

fn maintenance_cost(cost: Option<f64>) -> Result<Option<f64>, AppError> {
    match cost {
        Some(value) if !value.is_finite() || value < 0.0 => Err(AppError::Validation(
            "O custo da manutenção precisa ser um valor válido e não negativo.".to_owned(),
        )),
        _ => Ok(cost),
    }
}

pub async fn complete_inspection(
    pool: &SqlitePool,
    input: CompleteInspectionTask,
) -> Result<TaskCompletion, AppError> {
    let task = agenda::get(pool, &input.task_id).await?;
    if task.task_type != "inspection" || task.status != "pending" {
        return Err(AppError::Validation(
            "A tarefa não está disponível para registrar inspeção.".to_owned(),
        ));
    }
    let colony_id = task.colony_id.clone().ok_or_else(|| {
        AppError::Validation("A tarefa de inspeção não possui colônia.".to_owned())
    })?;
    let inspected_at = time::normalize_or_now(pool, &input.inspected_at, false).await?;
    let next = time::normalize_optional(&input.next_inspection_at, false)?;
    time::ensure_not_before(
        &next,
        &inspected_at,
        "A próxima inspeção não pode ser anterior à inspeção registrada.",
    )?;
    operational::ensure_colony_available_at(pool, &colony_id, &inspected_at).await?;
    let strength = inspection_strength(&input.strength)?;

    let mut tx = pool.begin().await?;
    let box_id: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT box_id FROM colony_box_occupancies
         WHERE colony_id=? AND started_at<=? AND (ended_at IS NULL OR ended_at>=?)
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(&colony_id)
    .bind(&inspected_at)
    .bind(&inspected_at)
    .fetch_optional(&mut *tx)
    .await?;
    let fact_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO inspections(
           id,colony_id,box_id,inspected_at,strength,queen_present,laying_status,
           food_reserves,brood_status,pests_notes,observations,actions_taken,next_inspection_at
         ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&fact_id)
    .bind(&colony_id)
    .bind(box_id)
    .bind(&inspected_at)
    .bind(strength)
    .bind(input.queen_present)
    .bind(optional(&input.laying_status))
    .bind(optional(&input.food_reserves))
    .bind(optional(&input.brood_status))
    .bind(optional(&input.pests_notes))
    .bind(optional(&input.observations))
    .bind(optional(&input.actions_taken))
    .bind(next)
    .execute(&mut *tx)
    .await?;
    agenda::mark_completed_by_fact_tx(&mut tx, &task.id, "inspection", "inspection", &fact_id)
        .await?;
    tx.commit().await?;
    agenda::reconcile_inspection(pool, &colony_id).await?;
    Ok(TaskCompletion {
        task: agenda::get(pool, &task.id).await?,
        fact_type: "inspection".to_owned(),
        fact_id,
    })
}

pub async fn complete_feeding(
    pool: &SqlitePool,
    input: CompleteFeedingTask,
) -> Result<TaskCompletion, AppError> {
    let task = agenda::get(pool, &input.task_id).await?;
    if task.task_type != "feeding" || task.status != "pending" {
        return Err(AppError::Validation(
            "A tarefa não está disponível para registrar alimentação.".to_owned(),
        ));
    }
    let colony_id = task.colony_id.clone().ok_or_else(|| {
        AppError::Validation("A tarefa de alimentação não possui colônia.".to_owned())
    })?;
    let fed_at = time::normalize_or_now(pool, &input.fed_at, false).await?;
    let next = time::normalize_optional(&input.next_feeding_at, false)?;
    time::ensure_not_before(
        &next,
        &fed_at,
        "A próxima alimentação não pode ser anterior à alimentação registrada.",
    )?;
    operational::ensure_colony_available_at(pool, &colony_id, &fed_at).await?;
    let food_type = required(&input.food_type, "Tipo de alimentação")?;
    let unit = feeding_unit(input.quantity, &input.unit)?;

    let mut tx = pool.begin().await?;
    let box_id: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT box_id FROM colony_box_occupancies
         WHERE colony_id=? AND started_at<=? AND (ended_at IS NULL OR ended_at>=?)
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(&colony_id)
    .bind(&fed_at)
    .bind(&fed_at)
    .fetch_optional(&mut *tx)
    .await?;
    let fact_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO feedings(
           id,colony_id,box_id,fed_at,food_type,quantity,unit,response_notes,notes,next_feeding_at
         ) VALUES(?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&fact_id)
    .bind(&colony_id)
    .bind(box_id)
    .bind(&fed_at)
    .bind(food_type)
    .bind(input.quantity)
    .bind(unit)
    .bind(optional(&input.response_notes))
    .bind(optional(&input.notes))
    .bind(next)
    .execute(&mut *tx)
    .await?;
    agenda::mark_completed_by_fact_tx(&mut tx, &task.id, "feeding", "feeding", &fact_id).await?;
    tx.commit().await?;
    agenda::reconcile_feeding(pool, &colony_id).await?;
    Ok(TaskCompletion {
        task: agenda::get(pool, &task.id).await?,
        fact_type: "feeding".to_owned(),
        fact_id,
    })
}

pub async fn complete_maintenance(
    pool: &SqlitePool,
    input: CompleteMaintenanceTask,
) -> Result<TaskCompletion, AppError> {
    let task = agenda::get(pool, &input.task_id).await?;
    if task.task_type != "maintenance" || task.status != "pending" {
        return Err(AppError::Validation(
            "A tarefa não está disponível para registrar manutenção.".to_owned(),
        ));
    }
    let box_id = task.box_id.clone().ok_or_else(|| {
        AppError::Validation("A tarefa de manutenção não possui caixa.".to_owned())
    })?;
    let maintained_at = time::normalize_or_now(pool, &input.maintained_at, false).await?;
    let next = time::normalize_optional(&input.next_maintenance_at, false)?;
    time::ensure_not_before(
        &next,
        &maintained_at,
        "A próxima manutenção não pode ser anterior à manutenção registrada.",
    )?;
    let maintenance_type = required(&input.maintenance_type, "Tipo de manutenção")?;
    if !MAINTENANCE_TYPES.contains(&maintenance_type.as_str()) {
        return Err(AppError::Validation(
            "Tipo de manutenção inválido.".to_owned(),
        ));
    }
    let cost = maintenance_cost(input.cost)?;
    let box_status: Option<String> = sqlx::query_scalar("SELECT status FROM boxes WHERE id=?")
        .bind(&box_id)
        .fetch_optional(pool)
        .await?;
    let box_status =
        box_status.ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))?;
    if box_status == "retired" {
        return Err(AppError::Validation(
            "Caixa aposentada não pode receber nova manutenção operacional.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let colony_id: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT colony_id FROM colony_box_occupancies
         WHERE box_id=? AND started_at<=? AND (ended_at IS NULL OR ended_at>=?)
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(&box_id)
    .bind(&maintained_at)
    .bind(&maintained_at)
    .fetch_optional(&mut *tx)
    .await?;
    let fact_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO box_maintenance_records(
           id,box_id,colony_id,maintained_at,maintenance_type,description,performed_by,cost,next_maintenance_at
         ) VALUES(?,?,?,?,?,?,?,?,?)",
    )
    .bind(&fact_id)
    .bind(&box_id)
    .bind(colony_id)
    .bind(&maintained_at)
    .bind(maintenance_type)
    .bind(optional(&input.description))
    .bind(optional(&input.performed_by))
    .bind(cost)
    .bind(next)
    .execute(&mut *tx)
    .await?;
    agenda::mark_completed_by_fact_tx(&mut tx, &task.id, "maintenance", "maintenance", &fact_id)
        .await?;
    tx.commit().await?;
    agenda::reconcile_maintenance(pool, &box_id).await?;
    Ok(TaskCompletion {
        task: agenda::get(pool, &task.id).await?,
        fact_type: "maintenance".to_owned(),
        fact_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agenda::{CreateTask, TaskQuery},
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
    async fn inspection_completion_is_atomic_and_links_fact() {
        let (pool, meliponary_id, colony_id, _) = seeded().await;
        let task = agenda::create_manual(
            &pool,
            CreateTask {
                meliponary_id,
                colony_id: Some(colony_id),
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
        let result = complete_inspection(
            &pool,
            CompleteInspectionTask {
                task_id: task.id.clone(),
                inspected_at: Some("2026-03-01 10:00:00".into()),
                strength: Some("strong".into()),
                queen_present: Some(true),
                laying_status: None,
                food_reserves: None,
                brood_status: None,
                pests_notes: None,
                observations: None,
                actions_taken: None,
                next_inspection_at: Some("2026-03-08 10:00:00".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(result.task.status, "completed");
        assert_eq!(
            result.task.completed_by_id.as_deref(),
            Some(result.fact_id.as_str())
        );
        let facts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM inspections WHERE id=?")
            .bind(&result.fact_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(facts, 1);
        let pending = agenda::list(
            &pool,
            TaskQuery {
                view: Some("pending".into()),
                task_type: Some("inspection".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn feeding_and_maintenance_completion_link_real_facts() {
        let (pool, meliponary_id, colony_id, box_id) = seeded().await;
        let feeding_task = agenda::create_manual(
            &pool,
            CreateTask {
                meliponary_id: meliponary_id.clone(),
                colony_id: Some(colony_id),
                box_id: None,
                task_type: "feeding".into(),
                title: "Alimentar JAT-001".into(),
                description: None,
                scheduled_for: time::local_now(&pool).await.unwrap(),
                priority: None,
            },
        )
        .await
        .unwrap();
        let feeding = complete_feeding(
            &pool,
            CompleteFeedingTask {
                task_id: feeding_task.id,
                fed_at: Some("2026-03-01 11:00:00".into()),
                food_type: "Xarope".into(),
                quantity: Some(20.0),
                unit: Some("ml".into()),
                response_notes: None,
                notes: None,
                next_feeding_at: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(feeding.task.status, "completed");
        let maintenance_task = agenda::create_manual(
            &pool,
            CreateTask {
                meliponary_id,
                colony_id: None,
                box_id: Some(box_id),
                task_type: "maintenance".into(),
                title: "Revisar caixa CX-001".into(),
                description: None,
                scheduled_for: time::local_now(&pool).await.unwrap(),
                priority: None,
            },
        )
        .await
        .unwrap();
        let maintenance = complete_maintenance(
            &pool,
            CompleteMaintenanceTask {
                task_id: maintenance_task.id,
                maintained_at: Some("2026-03-02 10:00:00".into()),
                maintenance_type: "inspection".into(),
                description: None,
                performed_by: None,
                cost: None,
                next_maintenance_at: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(maintenance.task.status, "completed");
        assert_eq!(maintenance.fact_type, "maintenance");
    }
}
