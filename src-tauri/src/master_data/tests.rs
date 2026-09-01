use super::*;
use crate::{
    agenda,
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
async fn meliponary_edit_archive_reactivate_and_empty_delete_are_safe() {
    let pool = pool().await;
    let item = repository::create_meliponary(
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
    let edited = edit_meliponary(
        &pool,
        EditMeliponary {
            id: item.id.clone(),
            name: "Principal Norte".into(),
            responsible_name: Some("Marcelo".into()),
            location: None,
            notes: None,
            reason: "Correção cadastral".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(edited.name, "Principal Norte");
    let archived = archive_meliponary(
        &pool,
        EntityAction {
            id: item.id.clone(),
            reason: "Sem uso".into(),
        },
    )
    .await
    .unwrap();
    assert!(archived.archived_at.is_some());
    let active = reactivate_meliponary(
        &pool,
        EntityAction {
            id: item.id.clone(),
            reason: "Retorno".into(),
        },
    )
    .await
    .unwrap();
    assert!(active.archived_at.is_none());
    delete_meliponary(
        &pool,
        EntityAction {
            id: item.id.clone(),
            reason: "Cadastro de teste".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meliponaries WHERE id=?")
            .bind(item.id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn archive_and_reactivate_meliponary_reconcile_only_derived_agenda() {
    let pool = pool().await;
    let mel = repository::create_meliponary(
        &pool,
        CreateMeliponary {
            name: "Operacional".into(),
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
    sqlx::query(
        "INSERT INTO inspections(id,colony_id,inspected_at,strength,next_inspection_at)
         VALUES('i1',?,'2026-01-10 10:00:00','medium','2026-03-01 10:00:00')",
    )
    .bind(&colony.id)
    .execute(&pool)
    .await
    .unwrap();
    agenda::reconcile_inspection(&pool, &colony.id)
        .await
        .unwrap();
    let derived_id: String = sqlx::query_scalar(
        "SELECT id FROM scheduled_tasks
         WHERE source_type='inspection' AND source_id='i1' AND status='pending'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO scheduled_tasks(id,meliponary_id,task_type,title,scheduled_for)
         VALUES('manual',?,'generic','Conferir cadastro','2026-03-01 12:00:00')",
    )
    .bind(&mel.id)
    .execute(&pool)
    .await
    .unwrap();

    archive_meliponary(
        &pool,
        EntityAction {
            id: mel.id.clone(),
            reason: "Pausa operacional".into(),
        },
    )
    .await
    .unwrap();
    let derived_status: String =
        sqlx::query_scalar("SELECT status FROM scheduled_tasks WHERE id=?")
            .bind(&derived_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let manual_status: String =
        sqlx::query_scalar("SELECT status FROM scheduled_tasks WHERE id='manual'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(derived_status, "cancelled");
    assert_eq!(manual_status, "pending");

    reactivate_meliponary(
        &pool,
        EntityAction {
            id: mel.id.clone(),
            reason: "Retorno operacional".into(),
        },
    )
    .await
    .unwrap();
    let restored: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scheduled_tasks
         WHERE source_type='inspection' AND source_id='i1' AND status='pending'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let manual_status: String =
        sqlx::query_scalar("SELECT status FROM scheduled_tasks WHERE id='manual'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(restored, 1);
    assert_eq!(manual_status, "pending");
}

#[tokio::test]
async fn used_meliponary_cannot_be_deleted() {
    let pool = pool().await;
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
    repository::create_box(
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
    assert!(delete_meliponary(
        &pool,
        EntityAction {
            id: mel.id,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
}

#[tokio::test]
async fn species_used_by_colony_cannot_be_deleted_but_empty_species_can() {
    let pool = pool().await;
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
    let used = repository::create_species(
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
    let empty = repository::create_species(
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
    repository::create_colony(
        &pool,
        CreateColony {
            meliponary_id: mel.id,
            species_id: used.id.clone(),
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
    assert!(delete_species(
        &pool,
        EntityAction {
            id: used.id,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
    delete_species(
        &pool,
        EntityAction {
            id: empty.id.clone(),
            reason: "Cadastro duplicado".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM species WHERE id=?")
            .bind(empty.id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn unused_box_and_colony_can_be_deleted_but_history_blocks_delete() {
    let pool = pool().await;
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
    let empty_box = repository::create_box(
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
    let empty_colony = repository::create_colony(
        &pool,
        CreateColony {
            meliponary_id: mel.id.clone(),
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
    delete_box(
        &pool,
        EntityAction {
            id: empty_box.id,
            reason: "Nunca usada".into(),
        },
    )
    .await
    .unwrap();
    delete_colony(
        &pool,
        EntityAction {
            id: empty_colony.id,
            reason: "Nunca usada".into(),
        },
    )
    .await
    .unwrap();

    let used_box = repository::create_box(
        &pool,
        CreateHiveBox {
            meliponary_id: mel.id.clone(),
            code: "CX-002".into(),
            model: None,
            material: None,
            location_note: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    let used_colony = repository::create_colony(
        &pool,
        CreateColony {
            meliponary_id: mel.id,
            species_id: species.id,
            code: "JAT-002".into(),
            origin_type: None,
            origin_notes: None,
            installed_at: None,
            mother_colony_id: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at) VALUES('o1',?,?,datetime('now','localtime'))")
            .bind(&used_colony.id).bind(&used_box.id).execute(&pool).await.unwrap();
    assert!(delete_box(
        &pool,
        EntityAction {
            id: used_box.id,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
    assert!(delete_colony(
        &pool,
        EntityAction {
            id: used_colony.id,
            reason: "Teste".into()
        }
    )
    .await
    .is_err());
}
