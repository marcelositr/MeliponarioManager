use super::*;
use crate::{
    alerts,
    domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
    feeding::{self, CreateFeeding},
    inspections::{self, CreateInspection},
    maintenance::{self, CreateBoxMaintenance},
    production::{self, CreateProductionRecord},
    repository,
};
use sqlx::sqlite::SqlitePoolOptions;

async fn seeded() -> (SqlitePool, String, String) {
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
    let box1 = repository::create_box(
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
            meliponary_id: mel.id,
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
            box_id: box1.id.clone(),
            started_at: Some("2026-01-01 09:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    (pool, colony.id, box1.id)
}

#[tokio::test]
async fn inspection_correction_and_void_preserve_latest_valid_semantics() {
    let (pool, colony_id, _) = seeded().await;
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
            next_inspection_at: Some("2026-01-20 10:00:00".into()),
        },
    )
    .await
    .unwrap();
    let latest = inspections::create(
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
    correct_inspection(
        &pool,
        CorrectInspection {
            id: old.id.clone(),
            inspected_at: "2026-01-11 10:00:00".into(),
            strength: "medium".into(),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: Some("2026-01-21 10:00:00".into()),
            reason: "Data corrigida".into(),
        },
    )
    .await
    .unwrap();
    void_inspection(
        &pool,
        VoidRecord {
            id: latest.id,
            reason: "Lançamento duplicado".into(),
        },
    )
    .await
    .unwrap();
    assert!(!alerts::list(&pool)
        .await
        .unwrap()
        .iter()
        .any(|a| a.alert_type == "weak_colony"));
    let audit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_records WHERE entity_type='inspection'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audit_count, 2);
}

#[tokio::test]
async fn invalid_inspection_correction_date_is_rejected() {
    let (pool, colony_id, _) = seeded().await;
    let item = inspections::create(
        &pool,
        CreateInspection {
            colony_id,
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
    assert!(correct_inspection(
        &pool,
        CorrectInspection {
            id: item.id,
            inspected_at: "2025-01-01 10:00:00".into(),
            strength: "strong".into(),
            queen_present: None,
            laying_status: None,
            food_reserves: None,
            brood_status: None,
            pests_notes: None,
            observations: None,
            actions_taken: None,
            next_inspection_at: None,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
}

#[tokio::test]
async fn voided_feeding_does_not_generate_due_alert() {
    let (pool, colony_id, _) = seeded().await;
    let item = feeding::create(
        &pool,
        CreateFeeding {
            colony_id,
            fed_at: Some("2026-01-10 10:00:00".into()),
            food_type: "Xarope".into(),
            quantity: Some(10.0),
            unit: Some("ml".into()),
            response_notes: None,
            notes: None,
            next_feeding_at: Some("2026-01-11 10:00:00".into()),
        },
    )
    .await
    .unwrap();
    void_feeding(
        &pool,
        VoidRecord {
            id: item.id,
            reason: "Erro de lançamento".into(),
        },
    )
    .await
    .unwrap();
    assert!(!alerts::list(&pool)
        .await
        .unwrap()
        .iter()
        .any(|a| a.alert_type == "feeding_due"));
}

#[tokio::test]
async fn voided_production_is_excluded_from_valid_count() {
    let (pool, colony_id, _) = seeded().await;
    let item = production::create(
        &pool,
        CreateProductionRecord {
            colony_id,
            harvested_at: Some("2026-02-01 10:00:00".into()),
            product_type: "honey".into(),
            quantity: 20.0,
            unit: "ml".into(),
            purpose: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    void_production(
        &pool,
        VoidRecord {
            id: item.id,
            reason: "Pesagem incorreta".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(production::count(&pool).await.unwrap(), 0);
}

#[tokio::test]
async fn maintenance_correction_revalidates_next_date() {
    let (pool, _, box_id) = seeded().await;
    let item = maintenance::create(
        &pool,
        CreateBoxMaintenance {
            box_id: box_id.clone(),
            maintained_at: Some("2026-02-01 10:00:00".into()),
            maintenance_type: "repair".into(),
            description: None,
            performed_by: None,
            cost: None,
            next_maintenance_at: None,
        },
    )
    .await
    .unwrap();
    assert!(correct_maintenance(
        &pool,
        CorrectMaintenance {
            id: item.id,
            box_id,
            maintained_at: "2026-02-01 10:00:00".into(),
            maintenance_type: "repair".into(),
            description: None,
            performed_by: None,
            cost: None,
            next_maintenance_at: Some("2026-01-01 10:00:00".into()),
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
}

#[tokio::test]
async fn occupancy_overlap_is_rejected() {
    let (pool, colony_id, first_box) = seeded().await;
    let mel_id: String = sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id=?")
        .bind(&colony_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let second = repository::create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: mel_id,
            code: "CX-002".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    repository::place_colony(
        &pool,
        PlaceColony {
            colony_id: colony_id.clone(),
            box_id: second.id,
            started_at: Some("2026-03-01 10:00:00".into()),
            reason: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let first_id: String = sqlx::query_scalar(
        "SELECT id FROM colony_box_occupancies WHERE colony_id=? AND box_id=?",
    )
    .bind(&colony_id)
    .bind(first_box)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(correct_occupancy(
        &pool,
        CorrectOccupancy {
            id: first_id,
            started_at: "2026-01-01 09:00:00".into(),
            ended_at: Some("2026-04-01 10:00:00".into()),
            occupancy_reason: None,
            notes: None,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
}
