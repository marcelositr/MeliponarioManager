use super::*;
use crate::{
    agenda,
    domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
    history,
    inspections::{self, CreateInspection},
    repository,
};
use sqlx::sqlite::SqlitePoolOptions;

struct Seed {
    source_meliponary_id: String,
    target_meliponary_id: String,
    species_id: String,
    source_box_id: String,
    target_box_id: String,
    colony_id: String,
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

async fn seed(pool: &SqlitePool) -> Seed {
    let source = repository::create_meliponary(
        pool,
        CreateMeliponary {
            name: "Meliponário A".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let target = repository::create_meliponary(
        pool,
        CreateMeliponary {
            name: "Meliponário B".into(),
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
    let source_box = repository::create_box(
        pool,
        CreateHiveBox {
            meliponary_id: source.id.clone(),
            code: "CX-A1".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let target_box = repository::create_box(
        pool,
        CreateHiveBox {
            meliponary_id: target.id.clone(),
            code: "CX-B1".into(),
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
            meliponary_id: source.id.clone(),
            species_id: species.id.clone(),
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
            box_id: source_box.id.clone(),
            started_at: Some("2026-01-01 09:00:00".into()),
            reason: Some("Instalação".into()),
            notes: None,
        },
    )
    .await
    .unwrap();

    Seed {
        source_meliponary_id: source.id,
        target_meliponary_id: target.id,
        species_id: species.id,
        source_box_id: source_box.id,
        target_box_id: target_box.id,
        colony_id: colony.id,
    }
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
            next_inspection_at: Some("2026-04-01 10:00:00".into()),
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
async fn internal_transfer_moves_colony_and_updates_derived_agenda_context() {
    let pool = test_pool().await;
    let seed = seed(&pool).await;
    let task_id = create_pending_inspection(&pool, &seed.colony_id).await;

    let movement = create(
        &pool,
        CreateMovement {
            colony_id: seed.colony_id.clone(),
            movement_type: "internal_transfer".into(),
            moved_at: Some("2026-02-01 10:00:00".into()),
            to_meliponary_id: Some(seed.target_meliponary_id.clone()),
            to_box_id: Some(seed.target_box_id.clone()),
            destination: None,
            document_reference: Some("REF-001".into()),
            notes: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(
        movement.from_box_id.as_deref(),
        Some(seed.source_box_id.as_str())
    );
    assert_eq!(
        movement.to_box_id.as_deref(),
        Some(seed.target_box_id.as_str())
    );

    let colony_state: (String, String) =
        sqlx::query_as("SELECT meliponary_id, status FROM colonies WHERE id = ?")
            .bind(&seed.colony_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(colony_state.0, seed.target_meliponary_id);
    assert_eq!(colony_state.1, "active");

    let active_box: String = sqlx::query_scalar(
        "SELECT box_id FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&seed.colony_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_box, seed.target_box_id);

    let history_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM colony_box_occupancies WHERE colony_id = ?")
            .bind(&seed.colony_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(history_count, 2);

    let task_context: (String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT meliponary_id,box_id,colony_id FROM scheduled_tasks WHERE id=?")
            .bind(task_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(task_context.0, seed.target_meliponary_id);
    assert_eq!(task_context.1.as_deref(), Some(seed.target_box_id.as_str()));
    assert_eq!(task_context.2.as_deref(), Some(seed.colony_id.as_str()));
}

#[tokio::test]
async fn occupied_target_box_rolls_back_internal_transfer() {
    let pool = test_pool().await;
    let seed = seed(&pool).await;

    let occupant = repository::create_colony(
        &pool,
        CreateColony {
            meliponary_id: seed.target_meliponary_id.clone(),
            species_id: seed.species_id.clone(),
            code: "JAT-900".into(),
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
        &pool,
        PlaceColony {
            colony_id: occupant.id,
            box_id: seed.target_box_id.clone(),
            started_at: Some("2026-01-10 09:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();

    let result = create(
        &pool,
        CreateMovement {
            colony_id: seed.colony_id.clone(),
            movement_type: "internal_transfer".into(),
            moved_at: Some("2026-02-01 10:00:00".into()),
            to_meliponary_id: Some(seed.target_meliponary_id),
            to_box_id: Some(seed.target_box_id),
            destination: None,
            document_reference: None,
            notes: None,
        },
    )
    .await;
    assert!(matches!(result, Err(AppError::Validation(_))));

    let current_meliponary: String =
        sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id = ?")
            .bind(&seed.colony_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(current_meliponary, seed.source_meliponary_id);

    let current_box: String = sqlx::query_scalar(
        "SELECT box_id FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&seed.colony_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(current_box, seed.source_box_id);

    let movement_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM colony_movements")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(movement_count, 0);
}

#[tokio::test]
async fn external_transfer_cancels_derived_agenda() {
    let pool = test_pool().await;
    let seed = seed(&pool).await;
    let task_id = create_pending_inspection(&pool, &seed.colony_id).await;

    let movement = create(
        &pool,
        CreateMovement {
            colony_id: seed.colony_id.clone(),
            movement_type: "external_transfer".into(),
            moved_at: Some("2026-03-01 10:00:00".into()),
            to_meliponary_id: None,
            to_box_id: None,
            destination: Some("Meliponário parceiro".into()),
            document_reference: Some("DOC-42".into()),
            notes: Some("Transferência definitiva".into()),
        },
    )
    .await
    .unwrap();

    assert_eq!(
        movement.destination.as_deref(),
        Some("Meliponário parceiro")
    );

    let status: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id = ?")
        .bind(&seed.colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "transferred");

    let active_occupancies: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&seed.colony_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_occupancies, 0);

    let task_status: String = sqlx::query_scalar("SELECT status FROM scheduled_tasks WHERE id=?")
        .bind(task_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(task_status, "cancelled");
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scheduled_tasks
         WHERE colony_id=? AND task_type='inspection' AND status='pending'",
    )
    .bind(&seed.colony_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pending, 0);
}

#[tokio::test]
async fn transport_records_history_without_changing_colony_state() {
    let pool = test_pool().await;
    let seed = seed(&pool).await;

    let movement = create(
        &pool,
        CreateMovement {
            colony_id: seed.colony_id.clone(),
            movement_type: "transport".into(),
            moved_at: Some("2026-01-20 08:00:00".into()),
            to_meliponary_id: None,
            to_box_id: None,
            destination: Some("Exposição municipal".into()),
            document_reference: None,
            notes: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(
        movement.from_box_id.as_deref(),
        Some(seed.source_box_id.as_str())
    );

    let colony_state: (String, String) =
        sqlx::query_as("SELECT meliponary_id, status FROM colonies WHERE id = ?")
            .bind(&seed.colony_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(colony_state.0, seed.source_meliponary_id);
    assert_eq!(colony_state.1, "active");

    let current_box: String = sqlx::query_scalar(
        "SELECT box_id FROM colony_box_occupancies
             WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&seed.colony_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(current_box, seed.source_box_id);
}

#[tokio::test]
async fn movement_appears_in_colony_timeline() {
    let pool = test_pool().await;
    let seed = seed(&pool).await;

    create(
        &pool,
        CreateMovement {
            colony_id: seed.colony_id.clone(),
            movement_type: "transport".into(),
            moved_at: Some("2026-01-20 08:00:00".into()),
            to_meliponary_id: None,
            to_box_id: None,
            destination: Some("Feira técnica".into()),
            document_reference: None,
            notes: None,
        },
    )
    .await
    .unwrap();

    let timeline = history::timeline_by_colony(&pool, &seed.colony_id)
        .await
        .unwrap();
    assert!(timeline.iter().any(|entry| entry.source_type == "movement"));
}
