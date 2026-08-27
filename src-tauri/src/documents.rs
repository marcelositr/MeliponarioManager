use crate::{movements, repository::AppError};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

const DOCUMENT_TYPES: &[&str] = &[
    "gta",
    "authorization",
    "invoice",
    "receipt",
    "declaration",
    "protocol",
    "certificate",
    "other",
];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MovementDocument {
    pub id: String,
    pub movement_id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub movement_type: String,
    pub moved_at: String,
    pub document_type: String,
    pub reference_number: String,
    pub source_system: Option<String>,
    pub issuer: Option<String>,
    pub issued_at: Option<String>,
    pub valid_until: Option<String>,
    pub file_path: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovementDocument {
    pub movement_id: String,
    pub document_type: String,
    pub reference_number: String,
    pub source_system: Option<String>,
    pub issuer: Option<String>,
    pub issued_at: Option<String>,
    pub valid_until: Option<String>,
    pub file_path: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TraceableMovement {
    pub id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub movement_type: String,
    pub moved_at: String,
    pub from_meliponary_id: String,
    pub from_meliponary_name: String,
    pub to_meliponary_id: Option<String>,
    pub to_meliponary_name: Option<String>,
    pub from_box_id: Option<String>,
    pub from_box_code: Option<String>,
    pub to_box_id: Option<String>,
    pub to_box_code: Option<String>,
    pub destination: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovementTraceability {
    pub movement: TraceableMovement,
    pub documents: Vec<MovementDocument>,
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

fn document_type(value: &str) -> Result<String, AppError> {
    let value = required(value, "Tipo do documento")?;
    if DOCUMENT_TYPES.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "Tipo de documento inválido.".to_owned(),
        ))
    }
}

async fn movement_exists(pool: &SqlitePool, movement_id: &str) -> Result<bool, AppError> {
    Ok(
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colony_movements WHERE id = ?)")
            .bind(movement_id)
            .fetch_one(pool)
            .await?,
    )
}

async fn colony_exists(pool: &SqlitePool, colony_id: &str) -> Result<bool, AppError> {
    Ok(
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
            .bind(colony_id)
            .fetch_one(pool)
            .await?,
    )
}

async fn get(pool: &SqlitePool, id: &str) -> Result<MovementDocument, AppError> {
    Ok(sqlx::query_as::<_, MovementDocument>(
        "SELECT
            d.id,
            d.movement_id,
            m.colony_id,
            c.code AS colony_code,
            m.movement_type,
            m.moved_at,
            d.document_type,
            d.reference_number,
            d.source_system,
            d.issuer,
            d.issued_at,
            d.valid_until,
            d.file_path,
            d.notes,
            d.created_at
         FROM movement_documents d
         JOIN colony_movements m ON m.id = d.movement_id
         JOIN colonies c ON c.id = m.colony_id
         WHERE d.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn create(
    pool: &SqlitePool,
    input: CreateMovementDocument,
) -> Result<MovementDocument, AppError> {
    let movement_id = required(&input.movement_id, "Movimentação")?;
    let document_type = document_type(&input.document_type)?;
    let reference_number = required(&input.reference_number, "Referência do documento")?;
    let source_system = optional(&input.source_system);
    let issuer = optional(&input.issuer);
    let issued_at = optional(&input.issued_at);
    let valid_until = optional(&input.valid_until);
    let file_path = optional(&input.file_path);
    let notes = optional(&input.notes);

    if !movement_exists(pool, &movement_id).await? {
        return Err(AppError::NotFound(
            "Movimentação não encontrada.".to_owned(),
        ));
    }

    if let (Some(issued_at), Some(valid_until)) = (&issued_at, &valid_until) {
        if valid_until < issued_at {
            return Err(AppError::Validation(
                "A validade do documento não pode ser anterior à emissão.".to_owned(),
            ));
        }
    }

    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM movement_documents
            WHERE movement_id = ?
              AND document_type = ?
              AND reference_number = ?
         )",
    )
    .bind(&movement_id)
    .bind(&document_type)
    .bind(&reference_number)
    .fetch_one(pool)
    .await?;

    if duplicate {
        return Err(AppError::Validation(
            "Este documento já está ligado à movimentação.".to_owned(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO movement_documents (
            id,
            movement_id,
            document_type,
            reference_number,
            source_system,
            issuer,
            issued_at,
            valid_until,
            file_path,
            notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&movement_id)
    .bind(&document_type)
    .bind(&reference_number)
    .bind(source_system)
    .bind(issuer)
    .bind(issued_at)
    .bind(valid_until)
    .bind(file_path)
    .bind(notes)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn list_by_movement(
    pool: &SqlitePool,
    movement_id: &str,
) -> Result<Vec<MovementDocument>, AppError> {
    let movement_id = required(movement_id, "Movimentação")?;
    if !movement_exists(pool, &movement_id).await? {
        return Err(AppError::NotFound(
            "Movimentação não encontrada.".to_owned(),
        ));
    }

    Ok(sqlx::query_as::<_, MovementDocument>(
        "SELECT
            d.id,
            d.movement_id,
            m.colony_id,
            c.code AS colony_code,
            m.movement_type,
            m.moved_at,
            d.document_type,
            d.reference_number,
            d.source_system,
            d.issuer,
            d.issued_at,
            d.valid_until,
            d.file_path,
            d.notes,
            d.created_at
         FROM movement_documents d
         JOIN colony_movements m ON m.id = d.movement_id
         JOIN colonies c ON c.id = m.colony_id
         WHERE d.movement_id = ?
         ORDER BY COALESCE(d.issued_at, d.created_at) DESC, d.created_at DESC, d.id DESC",
    )
    .bind(movement_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<MovementDocument>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    if !colony_exists(pool, &colony_id).await? {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, MovementDocument>(
        "SELECT
            d.id,
            d.movement_id,
            m.colony_id,
            c.code AS colony_code,
            m.movement_type,
            m.moved_at,
            d.document_type,
            d.reference_number,
            d.source_system,
            d.issuer,
            d.issued_at,
            d.valid_until,
            d.file_path,
            d.notes,
            d.created_at
         FROM movement_documents d
         JOIN colony_movements m ON m.id = d.movement_id
         JOIN colonies c ON c.id = m.colony_id
         WHERE m.colony_id = ?
         ORDER BY m.moved_at DESC, COALESCE(d.issued_at, d.created_at) DESC, d.id DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn traceability(
    pool: &SqlitePool,
    movement_id: &str,
) -> Result<MovementTraceability, AppError> {
    let movement_id = required(movement_id, "Movimentação")?;

    let movement = sqlx::query_as::<_, TraceableMovement>(
        "SELECT
            m.id,
            m.colony_id,
            c.code AS colony_code,
            m.movement_type,
            m.moved_at,
            m.from_meliponary_id,
            fm.name AS from_meliponary_name,
            m.to_meliponary_id,
            tm.name AS to_meliponary_name,
            m.from_box_id,
            fb.code AS from_box_code,
            m.to_box_id,
            tb.code AS to_box_code,
            m.destination,
            m.notes
         FROM colony_movements m
         JOIN colonies c ON c.id = m.colony_id
         JOIN meliponaries fm ON fm.id = m.from_meliponary_id
         LEFT JOIN meliponaries tm ON tm.id = m.to_meliponary_id
         LEFT JOIN boxes fb ON fb.id = m.from_box_id
         LEFT JOIN boxes tb ON tb.id = m.to_box_id
         WHERE m.id = ?",
    )
    .bind(&movement_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Movimentação não encontrada.".to_owned()))?;

    let documents = list_by_movement(pool, &movement_id).await?;
    Ok(MovementTraceability {
        movement,
        documents,
    })
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM movement_documents")
            .fetch_one(pool)
            .await?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        movements::{self, CreateMovement},
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

    async fn seed_movement(pool: &SqlitePool, legacy_reference: Option<&str>) -> (String, String) {
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
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        let movement = movements::create(
            pool,
            CreateMovement {
                colony_id: colony.id.clone(),
                movement_type: "transport".into(),
                moved_at: Some("2026-02-01 10:00:00".into()),
                to_meliponary_id: None,
                to_box_id: None,
                destination: Some("Evento técnico".into()),
                document_reference: legacy_reference.map(ToOwned::to_owned),
                notes: None,
            },
        )
        .await
        .unwrap();

        (movement.id, colony.id)
    }

    #[tokio::test]
    async fn structured_document_is_linked_to_movement_and_colony() {
        let pool = test_pool().await;
        let (movement_id, colony_id) = seed_movement(&pool, None).await;

        let document = create(
            &pool,
            CreateMovementDocument {
                movement_id: movement_id.clone(),
                document_type: "gta".into(),
                reference_number: "GTA-2026-001".into(),
                source_system: Some("GEDAVE".into()),
                issuer: Some("Órgão emissor".into()),
                issued_at: Some("2026-02-01 08:00:00".into()),
                valid_until: Some("2026-02-02 23:59:59".into()),
                file_path: Some("documents/movements/gta-2026-001.pdf".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(document.movement_id, movement_id);
        assert_eq!(document.colony_id, colony_id);
        assert_eq!(document.document_type, "gta");
        assert_eq!(document.source_system.as_deref(), Some("GEDAVE"));

        let bundle = traceability(&pool, &document.movement_id).await.unwrap();
        assert_eq!(bundle.documents.len(), 1);
        assert_eq!(bundle.movement.colony_id, document.colony_id);
    }

    #[tokio::test]
    async fn invalid_document_validity_is_rejected() {
        let pool = test_pool().await;
        let (movement_id, _) = seed_movement(&pool, None).await;

        let result = create(
            &pool,
            CreateMovementDocument {
                movement_id,
                document_type: "authorization".into(),
                reference_number: "AUT-001".into(),
                source_system: Some("GEFAU".into()),
                issuer: None,
                issued_at: Some("2026-02-10 10:00:00".into()),
                valid_until: Some("2026-02-09 10:00:00".into()),
                file_path: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn duplicate_document_reference_on_same_movement_is_rejected() {
        let pool = test_pool().await;
        let (movement_id, _) = seed_movement(&pool, None).await;

        let input = CreateMovementDocument {
            movement_id: movement_id.clone(),
            document_type: "protocol".into(),
            reference_number: "PROTO-77".into(),
            source_system: None,
            issuer: None,
            issued_at: None,
            valid_until: None,
            file_path: None,
            notes: None,
        };
        create(&pool, input).await.unwrap();

        let result = create(
            &pool,
            CreateMovementDocument {
                movement_id,
                document_type: "protocol".into(),
                reference_number: "PROTO-77".into(),
                source_system: None,
                issuer: None,
                issued_at: None,
                valid_until: None,
                file_path: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn legacy_movement_reference_is_normalized_by_database_trigger() {
        let pool = test_pool().await;
        let (movement_id, _) = seed_movement(&pool, Some("REF-LEGADO-42")).await;

        let legacy_column: Option<String> =
            sqlx::query_scalar("SELECT document_reference FROM colony_movements WHERE id = ?")
                .bind(&movement_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(legacy_column.is_none());

        let documents = list_by_movement(&pool, &movement_id).await.unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].document_type, "other");
        assert_eq!(documents[0].reference_number, "REF-LEGADO-42");
    }
}
