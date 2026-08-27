use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

const RESULTS: &[&str] = &["successful", "partial", "failed"];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyDivision {
    pub id: String,
    pub parent_colony_id: String,
    pub parent_colony_code: String,
    pub daughter_colony_id: Option<String>,
    pub daughter_colony_code: Option<String>,
    pub source_box_id: Option<String>,
    pub source_box_code: Option<String>,
    pub performed_at: String,
    pub result: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDivision {
    pub parent_colony_id: String,
    pub daughter_code: Option<String>,
    pub daughter_notes: Option<String>,
    pub performed_at: Option<String>,
    pub result: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GenealogyNode {
    pub colony_id: String,
    pub code: String,
    pub mother_colony_id: Option<String>,
    pub mother_colony_code: Option<String>,
    pub generation: i64,
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

fn division_result(value: &Option<String>) -> Result<String, AppError> {
    let value = optional(value).unwrap_or_else(|| "successful".to_owned());
    if RESULTS.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "Resultado da divisão inválido. Use successful, partial ou failed.".to_owned(),
        ))
    }
}

async fn get(pool: &SqlitePool, id: &str) -> Result<ColonyDivision, AppError> {
    Ok(sqlx::query_as::<_, ColonyDivision>(
        "SELECT d.id,
                d.parent_colony_id,
                parent.code AS parent_colony_code,
                d.daughter_colony_id,
                daughter.code AS daughter_colony_code,
                d.source_box_id,
                b.code AS source_box_code,
                d.performed_at,
                d.result,
                d.notes,
                d.created_at
         FROM colony_divisions d
         JOIN colonies parent ON parent.id = d.parent_colony_id
         LEFT JOIN colonies daughter ON daughter.id = d.daughter_colony_id
         LEFT JOIN boxes b ON b.id = d.source_box_id
         WHERE d.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn create(pool: &SqlitePool, input: CreateDivision) -> Result<ColonyDivision, AppError> {
    let parent_colony_id = required(&input.parent_colony_id, "Colônia mãe")?;
    let result = division_result(&input.result)?;
    let mut tx = pool.begin().await?;

    let parent: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT meliponary_id, species_id, code, status
         FROM colonies
         WHERE id = ?",
    )
    .bind(&parent_colony_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (meliponary_id, species_id, parent_code, parent_status) =
        parent.ok_or_else(|| AppError::NotFound("Colônia mãe não encontrada.".to_owned()))?;

    if matches!(parent_status.as_str(), "lost" | "inactive" | "transferred") {
        return Err(AppError::Validation(
            "Esta colônia não está ativa para registrar uma divisão.".to_owned(),
        ));
    }

    let performed_at = match optional(&input.performed_at) {
        Some(value) => value,
        None => {
            sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
                .fetch_one(&mut *tx)
                .await?
        }
    };

    let source_box_id: Option<String> = sqlx::query_scalar(
        "SELECT box_id
         FROM colony_box_occupancies
         WHERE colony_id = ?
           AND started_at <= ?
           AND (ended_at IS NULL OR ended_at >= ?)
         ORDER BY started_at DESC
         LIMIT 1",
    )
    .bind(&parent_colony_id)
    .bind(&performed_at)
    .bind(&performed_at)
    .fetch_optional(&mut *tx)
    .await?;

    let daughter_code = optional(&input.daughter_code);
    let daughter_colony_id = if result == "failed" {
        if daughter_code.is_some() {
            return Err(AppError::Validation(
                "Uma divisão malsucedida não pode criar uma colônia filha.".to_owned(),
            ));
        }
        None
    } else {
        let daughter_code = daughter_code.as_deref().ok_or_else(|| {
            AppError::Validation("Identificação da colônia filha é obrigatória.".to_owned())
        })?;

        let duplicate: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM colonies
                WHERE meliponary_id = ? AND code = ? COLLATE NOCASE
             )",
        )
        .bind(&meliponary_id)
        .bind(daughter_code)
        .fetch_one(&mut *tx)
        .await?;

        if duplicate {
            return Err(AppError::Validation(
                "Já existe uma colônia com esta identificação neste meliponário.".to_owned(),
            ));
        }

        let daughter_id = Uuid::new_v4().to_string();
        let origin_notes = format!("Divisão da colônia {parent_code}");

        sqlx::query(
            "INSERT INTO colonies (
                id, meliponary_id, species_id, code, origin_type, origin_notes,
                installed_at, mother_colony_id, notes
             ) VALUES (?, ?, ?, ?, 'multiplication', ?, ?, ?, ?)",
        )
        .bind(&daughter_id)
        .bind(&meliponary_id)
        .bind(&species_id)
        .bind(daughter_code)
        .bind(origin_notes)
        .bind(&performed_at)
        .bind(&parent_colony_id)
        .bind(optional(&input.daughter_notes))
        .execute(&mut *tx)
        .await?;

        Some(daughter_id)
    };

    let division_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO colony_divisions (
            id, parent_colony_id, daughter_colony_id, source_box_id,
            performed_at, result, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&division_id)
    .bind(&parent_colony_id)
    .bind(&daughter_colony_id)
    .bind(&source_box_id)
    .bind(&performed_at)
    .bind(&result)
    .bind(optional(&input.notes))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    get(pool, &division_id).await
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<ColonyDivision>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
        .bind(&colony_id)
        .fetch_one(pool)
        .await?;

    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, ColonyDivision>(
        "SELECT d.id,
                d.parent_colony_id,
                parent.code AS parent_colony_code,
                d.daughter_colony_id,
                daughter.code AS daughter_colony_code,
                d.source_box_id,
                b.code AS source_box_code,
                d.performed_at,
                d.result,
                d.notes,
                d.created_at
         FROM colony_divisions d
         JOIN colonies parent ON parent.id = d.parent_colony_id
         LEFT JOIN colonies daughter ON daughter.id = d.daughter_colony_id
         LEFT JOIN boxes b ON b.id = d.source_box_id
         WHERE d.parent_colony_id = ? OR d.daughter_colony_id = ?
         ORDER BY d.performed_at DESC, d.created_at DESC",
    )
    .bind(&colony_id)
    .bind(&colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn genealogy(
    pool: &SqlitePool,
    root_colony_id: &str,
) -> Result<Vec<GenealogyNode>, AppError> {
    let root_colony_id = required(root_colony_id, "Colônia")?;

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
        .bind(&root_colony_id)
        .fetch_one(pool)
        .await?;

    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, GenealogyNode>(
        "WITH RECURSIVE family(colony_id, code, mother_colony_id, generation) AS (
            SELECT id, code, mother_colony_id, 0
            FROM colonies
            WHERE id = ?

            UNION ALL

            SELECT c.id, c.code, c.mother_colony_id, family.generation + 1
            FROM colonies c
            JOIN family ON c.mother_colony_id = family.colony_id
         )
         SELECT family.colony_id,
                family.code,
                family.mother_colony_id,
                mother.code AS mother_colony_code,
                family.generation
         FROM family
         LEFT JOIN colonies mother ON mother.id = family.mother_colony_id
         ORDER BY family.generation, family.code COLLATE NOCASE",
    )
    .bind(root_colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM colony_divisions")
        .fetch_one(pool)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
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

    async fn seed_parent(pool: &SqlitePool) -> String {
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

        let parent = repository::create_colony(
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
                colony_id: parent.id.clone(),
                box_id: hive_box.id,
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: Some("Instalação".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        parent.id
    }

    #[tokio::test]
    async fn successful_division_creates_daughter_and_genealogy() {
        let pool = test_pool().await;
        let parent_id = seed_parent(&pool).await;

        let division = create(
            &pool,
            CreateDivision {
                parent_colony_id: parent_id.clone(),
                daughter_code: Some("JAT-002".into()),
                daughter_notes: Some("Primeira filha".into()),
                performed_at: Some("2026-02-01 10:00:00".into()),
                result: Some("successful".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(division.parent_colony_code, "JAT-001");
        assert_eq!(division.daughter_colony_code.as_deref(), Some("JAT-002"));
        assert_eq!(division.source_box_code.as_deref(), Some("CX-001"));

        let daughter_id = division.daughter_colony_id.unwrap();
        let mother_id: Option<String> =
            sqlx::query_scalar("SELECT mother_colony_id FROM colonies WHERE id = ?")
                .bind(&daughter_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(mother_id.as_deref(), Some(parent_id.as_str()));

        let tree = genealogy(&pool, &parent_id).await.unwrap();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].generation, 0);
        assert_eq!(tree[1].generation, 1);
    }

    #[tokio::test]
    async fn failed_division_does_not_create_daughter() {
        let pool = test_pool().await;
        let parent_id = seed_parent(&pool).await;

        let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM colonies")
            .fetch_one(&pool)
            .await
            .unwrap();

        let division = create(
            &pool,
            CreateDivision {
                parent_colony_id: parent_id,
                daughter_code: None,
                daughter_notes: None,
                performed_at: Some("2026-02-01 10:00:00".into()),
                result: Some("failed".into()),
                notes: Some("Não vingou".into()),
            },
        )
        .await
        .unwrap();

        let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM colonies")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(before, after);
        assert!(division.daughter_colony_id.is_none());
    }

    #[tokio::test]
    async fn duplicate_daughter_code_is_rejected_without_division_record() {
        let pool = test_pool().await;
        let parent_id = seed_parent(&pool).await;

        create(
            &pool,
            CreateDivision {
                parent_colony_id: parent_id.clone(),
                daughter_code: Some("JAT-002".into()),
                daughter_notes: None,
                performed_at: Some("2026-02-01 10:00:00".into()),
                result: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let result = create(
            &pool,
            CreateDivision {
                parent_colony_id: parent_id,
                daughter_code: Some("JAT-002".into()),
                daughter_notes: None,
                performed_at: Some("2026-03-01 10:00:00".into()),
                result: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::Validation(_))));

        let divisions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM colony_divisions")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(divisions, 1);
    }
}
