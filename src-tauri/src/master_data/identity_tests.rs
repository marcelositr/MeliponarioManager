use super::*;
use crate::{
    domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies},
    repository,
};
use sqlx::sqlite::SqlitePoolOptions;

async fn pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn edits_reject_new_normalized_collisions_but_preserve_legacy_identity() {
    let pool = pool().await;
    let primary = repository::create_meliponary(
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
    let secondary = repository::create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Secundário".into(),
            responsible_name: None,
            location: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        edit_meliponary(
            &pool,
            EditMeliponary {
                id: secondary.id,
                name: "  PRINCIPAL  ".into(),
                responsible_name: None,
                location: None,
                notes: None,
                reason: "Teste de duplicidade".into(),
            },
        )
        .await,
        Err(AppError::Validation(_))
    ));

    let jatai = repository::create_species(
        &pool,
        CreateSpecies {
            common_name: "Jataí".into(),
            scientific_name: Some("Tetragonisca angustula".into()),
            genus: Some("Tetragonisca".into()),
            notes: None,
        },
    )
    .await
    .unwrap();
    let mandacaia = repository::create_species(
        &pool,
        CreateSpecies {
            common_name: "Mandaçaia".into(),
            scientific_name: Some("Melipona quadrifasciata".into()),
            genus: Some("Melipona".into()),
            notes: None,
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        edit_species(
            &pool,
            EditSpecies {
                id: mandacaia.id,
                common_name: "Outro nome".into(),
                scientific_name: Some(" TETRAGONISCA ANGUSTULA ".into()),
                genus: Some("Outro".into()),
                notes: None,
                reason: "Teste de duplicidade".into(),
            },
        )
        .await,
        Err(AppError::Validation(_))
    ));

    let box_one = repository::create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: primary.id.clone(),
            code: "CX-01".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let box_two = repository::create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: primary.id.clone(),
            code: "CX-02".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        edit_box(
            &pool,
            EditBox {
                id: box_two.id,
                code: " cx-01 ".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
                reason: "Teste de duplicidade".into(),
            },
        )
        .await,
        Err(AppError::Validation(_))
    ));

    let colony_one = repository::create_colony(
        &pool,
        CreateColony {
            meliponary_id: primary.id.clone(),
            species_id: jatai.id.clone(),
            code: "JAT-01".into(),
            origin_type: None,
            origin_notes: None,
            installed_at: None,
            mother_colony_id: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let colony_two = repository::create_colony(
        &pool,
        CreateColony {
            meliponary_id: primary.id.clone(),
            species_id: jatai.id,
            code: "JAT-02".into(),
            origin_type: None,
            origin_notes: None,
            installed_at: None,
            mother_colony_id: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        edit_colony(
            &pool,
            EditColony {
                id: colony_two.id,
                code: " jat-01 ".into(),
                origin_notes: None,
                notes: None,
                reason: "Teste de duplicidade".into(),
            },
        )
        .await,
        Err(AppError::Validation(_))
    ));

    sqlx::query("INSERT INTO boxes (id, meliponary_id, code) VALUES ('legacy-box', ?, 'cx-01')")
        .bind(&primary.id)
        .execute(&pool)
        .await
        .unwrap();
    let edited = edit_box(
        &pool,
        EditBox {
            id: box_one.id,
            code: "CX-01".into(),
            model: None,
            material: None,
            location_note: None,
            notes: Some("Edição não identitária permitida".into()),
            reason: "Revisão cadastral".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(edited.code, "CX-01");
    assert_eq!(
        edited.notes.as_deref(),
        Some("Edição não identitária permitida")
    );
    assert_eq!(colony_one.code, "JAT-01");
}
