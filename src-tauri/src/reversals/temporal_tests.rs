use super::*;
use crate::{
    domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
    lifecycle::{self, ChangeColonyLifecycle},
    movements::{self, CreateMovement},
    repository,
};
use sqlx::sqlite::SqlitePoolOptions;

struct Seed {
    pool: SqlitePool,
    colony_id: String,
    source_meliponary_id: String,
    target_meliponary_id: String,
    source_box_id: String,
    target_box_id: String,
}

async fn seed() -> Seed {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    let source_meliponary = repository::create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Origem temporal".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let target_meliponary = repository::create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Destino temporal".into(),
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
            common_name: "Jataí temporal".into(),
            scientific_name: None,
            genus: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let source_box = repository::create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: source_meliponary.id.clone(),
            code: "T-ORIGEM".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let target_box = repository::create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: target_meliponary.id.clone(),
            code: "T-DESTINO".into(),
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
            meliponary_id: source_meliponary.id.clone(),
            species_id: species.id,
            code: "JAT-TEMP".into(),
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
            box_id: source_box.id.clone(),
            started_at: Some("2026-01-01 09:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    Seed {
        pool,
        colony_id: colony.id,
        source_meliponary_id: source_meliponary.id,
        target_meliponary_id: target_meliponary.id,
        source_box_id: source_box.id,
        target_box_id: target_box.id,
    }
}

async fn occupancies(seed: &Seed) -> Vec<(String, String, Option<String>)> {
    sqlx::query_as(
        "SELECT box_id,started_at,ended_at
         FROM colony_box_occupancies
         WHERE colony_id=?
         ORDER BY started_at,id",
    )
    .bind(&seed.colony_id)
    .fetch_all(&seed.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn movement_reversal_preserves_past_occupancy_and_restores_origin_from_reversal_time() {
    let seed = seed().await;
    let movement = movements::create(
        &seed.pool,
        CreateMovement {
            colony_id: seed.colony_id.clone(),
            movement_type: "internal_transfer".into(),
            moved_at: Some("2026-02-01 10:00:00".into()),
            to_meliponary_id: Some(seed.target_meliponary_id.clone()),
            to_box_id: Some(seed.target_box_id.clone()),
            destination: None,
            document_reference: None,
            notes: None,
        },
    )
    .await
    .unwrap();

    reverse_movement(
        &seed.pool,
        ReverseRecord {
            id: movement.id.clone(),
            reason: "Desfazer consequência atual".into(),
        },
    )
    .await
    .unwrap();

    let reversed_at: String = sqlx::query_scalar(
        "SELECT reversed_at FROM colony_movements WHERE id=?",
    )
    .bind(movement.id)
    .fetch_one(&seed.pool)
    .await
    .unwrap();
    let rows = occupancies(&seed).await;
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0], (seed.source_box_id.clone(), "2026-01-01 09:00:00".into(), Some("2026-02-01 10:00:00".into())));
    assert_eq!(rows[1], (seed.target_box_id, "2026-02-01 10:00:00".into(), Some(reversed_at.clone())));
    assert_eq!(rows[2], (seed.source_box_id, reversed_at, None));
}

#[tokio::test]
async fn lifecycle_reversal_restores_box_from_reversal_time_without_rewriting_history() {
    let seed = seed().await;
    let lifecycle = lifecycle::change(
        &seed.pool,
        ChangeColonyLifecycle {
            colony_id: seed.colony_id.clone(),
            action: "deactivate".into(),
            occurred_at: Some("2026-02-01 10:00:00".into()),
            reason: Some("Pausa operacional".into()),
            notes: None,
        },
    )
    .await
    .unwrap();

    reverse_lifecycle(
        &seed.pool,
        ReverseRecord {
            id: lifecycle.id.clone(),
            reason: "Retomar consequência atual".into(),
        },
    )
    .await
    .unwrap();

    let reversed_at: String = sqlx::query_scalar(
        "SELECT reversed_at FROM colony_lifecycle_records WHERE id=?",
    )
    .bind(lifecycle.id)
    .fetch_one(&seed.pool)
    .await
    .unwrap();
    let rows = occupancies(&seed).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], (seed.source_box_id.clone(), "2026-01-01 09:00:00".into(), Some("2026-02-01 10:00:00".into())));
    assert_eq!(rows[1], (seed.source_box_id, reversed_at, None));
}
