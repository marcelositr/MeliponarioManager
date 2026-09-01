use super::*;
use crate::{
    agenda,
    inspections::{self, CreateInspection},
};
use sqlx::sqlite::SqlitePoolOptions;

#[test]
fn database_error_display_is_safe_for_ipc() {
    let error = AppError::Database(sqlx::Error::RowNotFound);
    assert_eq!(
        error.to_string(),
        "Não foi possível acessar os dados locais."
    );
}

#[test]
fn domain_error_display_preserves_user_facing_messages() {
    assert_eq!(
        AppError::Validation("Valor inválido.".to_owned()).to_string(),
        "Valor inválido."
    );
    assert_eq!(
        AppError::NotFound("Registro não encontrado.".to_owned()).to_string(),
        "Registro não encontrado."
    );
}

async fn test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    pool
}

async fn seed(pool: &SqlitePool) -> (Meliponary, Species, HiveBox, HiveBox, Colony) {
    let meliponary = create_meliponary(
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

    let species = create_species(
        pool,
        CreateSpecies {
            common_name: "Jataí".into(),
            scientific_name: Some("Tetragonisca angustula".into()),
            genus: Some("Tetragonisca".into()),
            notes: None,
        },
    )
    .await
    .unwrap();

    let box_one = create_box(
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

    let box_two = create_box(
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

    let colony = create_colony(
        pool,
        CreateColony {
            meliponary_id: meliponary.id.clone(),
            species_id: species.id.clone(),
            code: "JAT-001".into(),
            origin_type: Some("historical".into()),
            origin_notes: None,
            installed_at: None,
            mother_colony_id: None,
            notes: None,
        },
    )
    .await
    .unwrap();

    (meliponary, species, box_one, box_two, colony)
}

async fn create_pending_inspection(pool: &SqlitePool, colony_id: &str) -> String {
    inspections::create(
        pool,
        CreateInspection {
            colony_id: colony_id.to_owned(),
            inspected_at: Some("2026-01-10 10:00:00".into()),
            strength: Some("medium".into()),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: Some("2026-03-01 10:00:00".into()),
        },
    )
    .await
    .unwrap();
    agenda::reconcile_inspection(pool, colony_id).await.unwrap();
    sqlx::query_scalar(
        "SELECT id FROM scheduled_tasks
         WHERE colony_id=? AND task_type='inspection' AND status='pending'",
    )
    .bind(colony_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn moving_colony_preserves_box_history_and_updates_derived_agenda_context() {
    let pool = test_pool().await;
    let (_, _, box_one, box_two, colony) = seed(&pool).await;

    place_colony(
        &pool,
        PlaceColony {
            colony_id: colony.id.clone(),
            box_id: box_one.id.clone(),
            started_at: Some("2026-01-01 10:00:00".into()),
            reason: Some("Instalação".into()),
            notes: None,
        },
    )
    .await
    .unwrap();

    let task_id = create_pending_inspection(&pool, &colony.id).await;
    let task_box_before: Option<String> =
        sqlx::query_scalar("SELECT box_id FROM scheduled_tasks WHERE id=?")
            .bind(&task_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(task_box_before.as_deref(), Some(box_one.id.as_str()));

    place_colony(
        &pool,
        PlaceColony {
            colony_id: colony.id.clone(),
            box_id: box_two.id.clone(),
            started_at: Some("2026-02-01 10:00:00".into()),
            reason: Some("Troca de caixa".into()),
            notes: None,
        },
    )
    .await
    .unwrap();

    let history_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM colony_box_occupancies WHERE colony_id = ?")
            .bind(&colony.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(history_count, 2);

    let colonies = list_colonies(&pool).await.unwrap();
    assert_eq!(colonies[0].current_box_code.as_deref(), Some("CX-002"));

    let task_after: (String, Option<String>) =
        sqlx::query_as("SELECT id,box_id FROM scheduled_tasks WHERE id=?")
            .bind(&task_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(task_after.0, task_id);
    assert_eq!(task_after.1.as_deref(), Some(box_two.id.as_str()));
}

#[tokio::test]
async fn agenda_failure_rolls_back_box_placement() {
    let pool = test_pool().await;
    let (_, _, box_one, box_two, colony) = seed(&pool).await;
    place_colony(
        &pool,
        PlaceColony {
            colony_id: colony.id.clone(),
            box_id: box_one.id.clone(),
            started_at: Some("2026-01-01 10:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    create_pending_inspection(&pool, &colony.id).await;
    sqlx::query(
        "CREATE TRIGGER fail_agenda_context_update
         BEFORE UPDATE ON scheduled_tasks
         WHEN OLD.source_type='inspection'
         BEGIN
           SELECT RAISE(ABORT,'forced agenda failure');
         END",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = place_colony(
        &pool,
        PlaceColony {
            colony_id: colony.id.clone(),
            box_id: box_two.id,
            started_at: Some("2026-02-01 10:00:00".into()),
            reason: Some("Troca deve falhar".into()),
            notes: None,
        },
    )
    .await;
    assert!(result.is_err());

    let active_box: String = sqlx::query_scalar(
        "SELECT box_id FROM colony_box_occupancies
         WHERE colony_id=? AND ended_at IS NULL",
    )
    .bind(&colony.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_box, box_one.id);
    let occupancy_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM colony_box_occupancies WHERE colony_id=?")
            .bind(&colony.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(occupancy_count, 1);
}

#[tokio::test]
async fn colony_cannot_be_placed_in_box_from_another_meliponary() {
    let pool = test_pool().await;
    let (_, _, _, _, colony) = seed(&pool).await;

    let other = create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Outro meliponário".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();

    let other_box = create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: other.id,
            code: "CX-100".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();

    let result = place_colony(
        &pool,
        PlaceColony {
            colony_id: colony.id,
            box_id: other_box.id,
            started_at: None,
            reason: None,
            notes: None,
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn nonactive_box_is_rejected_by_repository() {
    for status in ["maintenance", "retired"] {
        let pool = test_pool().await;
        let (_, _, box_one, _, colony) = seed(&pool).await;
        sqlx::query("UPDATE boxes SET status = ? WHERE id = ?")
            .bind(status)
            .bind(&box_one.id)
            .execute(&pool)
            .await
            .unwrap();

        let result = place_colony(
            &pool,
            PlaceColony {
                colony_id: colony.id,
                box_id: box_one.id,
                started_at: Some("2026-01-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}

#[tokio::test]
async fn archived_master_data_cannot_receive_new_operational_children() {
    let pool = test_pool().await;
    let mel = create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Arquivado".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let species = create_species(
        &pool,
        CreateSpecies {
            common_name: "Mandaçaia".into(),
            scientific_name: None,
            genus: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    sqlx::query("UPDATE meliponaries SET archived_at = datetime('now','localtime') WHERE id = ?")
        .bind(&mel.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE species SET archived_at = datetime('now','localtime') WHERE id = ?")
        .bind(&species.id)
        .execute(&pool)
        .await
        .unwrap();

    assert!(create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: mel.id.clone(),
            code: "CX-X".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .is_err());

    let active_mel = create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Ativo".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    assert!(create_colony(
        &pool,
        CreateColony {
            meliponary_id: active_mel.id,
            species_id: species.id,
            code: "MAN-001".into(),
            origin_type: None,
            origin_notes: None,
            installed_at: None,
            mother_colony_id: None,
            notes: None,
        },
    )
    .await
    .is_err());
}
