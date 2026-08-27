use crate::repository::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::path::{Component, Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InspectionPhoto {
    pub id: String,
    pub inspection_id: String,
    pub colony_id: String,
    pub colony_code: String,
    pub relative_path: String,
    pub original_name: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub captured_at: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportInspectionPhoto {
    pub inspection_id: String,
    pub source_path: String,
    pub captured_at: Option<String>,
    pub notes: Option<String>,
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

fn storage_error(context: &str, error: std::io::Error) -> AppError {
    AppError::Validation(format!("{context}: {error}"))
}

fn image_format(path: &Path) -> Result<(&'static str, &'static str), AppError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| {
            AppError::Validation("A foto precisa ter uma extensão reconhecida.".to_owned())
        })?;

    match extension.as_str() {
        "jpg" | "jpeg" => Ok(("jpg", "image/jpeg")),
        "png" => Ok(("png", "image/png")),
        "webp" => Ok(("webp", "image/webp")),
        _ => Err(AppError::Validation(
            "Formato de foto não suportado. Use JPG, PNG ou WebP.".to_owned(),
        )),
    }
}

fn absolute_media_path(data_dir: &Path, relative_path: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(relative_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AppError::Validation(
            "Caminho de mídia armazenado é inválido.".to_owned(),
        ));
    }

    let mut components = path.components();
    let first = components.next().and_then(|component| match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    });
    let second = components.next().and_then(|component| match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    });

    if first != Some("media") || second != Some("inspections") {
        return Err(AppError::Validation(
            "Caminho de mídia fora do diretório permitido.".to_owned(),
        ));
    }

    Ok(data_dir.join(path))
}

async fn get(pool: &SqlitePool, id: &str) -> Result<InspectionPhoto, AppError> {
    Ok(sqlx::query_as::<_, InspectionPhoto>(
        "SELECT
            p.id,
            p.inspection_id,
            i.colony_id,
            c.code AS colony_code,
            p.relative_path,
            p.original_name,
            p.mime_type,
            p.byte_size,
            p.captured_at,
            p.notes,
            p.created_at
         FROM inspection_photos p
         JOIN inspections i ON i.id = p.inspection_id
         JOIN colonies c ON c.id = i.colony_id
         WHERE p.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn import_photo(
    pool: &SqlitePool,
    data_dir: &Path,
    input: ImportInspectionPhoto,
) -> Result<InspectionPhoto, AppError> {
    let inspection_id = required(&input.inspection_id, "Inspeção")?;
    let source_path = PathBuf::from(required(&input.source_path, "Arquivo de origem")?);

    let inspection: Option<(String, String)> =
        sqlx::query_as("SELECT colony_id, inspected_at FROM inspections WHERE id = ?")
            .bind(&inspection_id)
            .fetch_optional(pool)
            .await?;
    let (_, inspected_at) =
        inspection.ok_or_else(|| AppError::NotFound("Inspeção não encontrada.".to_owned()))?;

    let metadata = fs::metadata(&source_path)
        .await
        .map_err(|error| storage_error("Não foi possível acessar a foto de origem", error))?;
    if !metadata.is_file() {
        return Err(AppError::Validation(
            "O caminho informado precisa apontar para um arquivo.".to_owned(),
        ));
    }

    let (extension, mime_type) = image_format(&source_path)?;
    let original_name = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::Validation("Nome do arquivo de origem é inválido.".to_owned()))?
        .to_owned();
    let byte_size = i64::try_from(metadata.len()).map_err(|_| {
        AppError::Validation("O arquivo é grande demais para ser registrado.".to_owned())
    })?;
    let captured_at = optional(&input.captured_at).unwrap_or(inspected_at);
    let notes = optional(&input.notes);
    let id = Uuid::new_v4().to_string();

    let target_dir = data_dir
        .join("media")
        .join("inspections")
        .join(&inspection_id);
    fs::create_dir_all(&target_dir)
        .await
        .map_err(|error| storage_error("Não foi possível criar o diretório de mídia", error))?;

    let stored_name = format!("{id}.{extension}");
    let target_path = target_dir.join(&stored_name);
    let relative_path = format!("media/inspections/{inspection_id}/{stored_name}");

    fs::copy(&source_path, &target_path)
        .await
        .map_err(|error| storage_error("Não foi possível copiar a foto", error))?;

    let insert_result = sqlx::query(
        "INSERT INTO inspection_photos (
            id, inspection_id, relative_path, original_name, mime_type,
            byte_size, captured_at, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&inspection_id)
    .bind(&relative_path)
    .bind(original_name)
    .bind(mime_type)
    .bind(byte_size)
    .bind(captured_at)
    .bind(notes)
    .execute(pool)
    .await;

    if let Err(error) = insert_result {
        let _ = fs::remove_file(&target_path).await;
        return Err(AppError::Database(error));
    }

    get(pool, &id).await
}

pub async fn list_by_inspection(
    pool: &SqlitePool,
    inspection_id: &str,
) -> Result<Vec<InspectionPhoto>, AppError> {
    let inspection_id = required(inspection_id, "Inspeção")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM inspections WHERE id = ?)")
        .bind(&inspection_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Inspeção não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, InspectionPhoto>(
        "SELECT
            p.id,
            p.inspection_id,
            i.colony_id,
            c.code AS colony_code,
            p.relative_path,
            p.original_name,
            p.mime_type,
            p.byte_size,
            p.captured_at,
            p.notes,
            p.created_at
         FROM inspection_photos p
         JOIN inspections i ON i.id = p.inspection_id
         JOIN colonies c ON c.id = i.colony_id
         WHERE p.inspection_id = ?
         ORDER BY p.captured_at DESC, p.created_at DESC",
    )
    .bind(inspection_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<InspectionPhoto>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
        .bind(&colony_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, InspectionPhoto>(
        "SELECT
            p.id,
            p.inspection_id,
            i.colony_id,
            c.code AS colony_code,
            p.relative_path,
            p.original_name,
            p.mime_type,
            p.byte_size,
            p.captured_at,
            p.notes,
            p.created_at
         FROM inspection_photos p
         JOIN inspections i ON i.id = p.inspection_id
         JOIN colonies c ON c.id = i.colony_id
         WHERE i.colony_id = ?
         ORDER BY p.captured_at DESC, p.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn delete_photo(
    pool: &SqlitePool,
    data_dir: &Path,
    photo_id: &str,
) -> Result<(), AppError> {
    let photo_id = required(photo_id, "Foto")?;
    let relative_path: Option<String> =
        sqlx::query_scalar("SELECT relative_path FROM inspection_photos WHERE id = ?")
            .bind(&photo_id)
            .fetch_optional(pool)
            .await?;
    let relative_path =
        relative_path.ok_or_else(|| AppError::NotFound("Foto não encontrada.".to_owned()))?;
    let file_path = absolute_media_path(data_dir, &relative_path)?;

    let mut tx = pool.begin().await?;
    let tombstone = file_path.with_file_name(format!(".deleting-{}", Uuid::new_v4()));
    let moved_to_tombstone = match fs::metadata(&file_path).await {
        Ok(metadata) if metadata.is_file() => {
            fs::rename(&file_path, &tombstone).await.map_err(|error| {
                storage_error("Não foi possível preparar a exclusão da foto", error)
            })?;
            true
        }
        Ok(_) => {
            return Err(AppError::Validation(
                "O caminho da mídia não aponta para um arquivo.".to_owned(),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            return Err(storage_error(
                "Não foi possível verificar a foto armazenada",
                error,
            ));
        }
    };

    let delete_result = sqlx::query("DELETE FROM inspection_photos WHERE id = ?")
        .bind(&photo_id)
        .execute(&mut *tx)
        .await;
    if let Err(error) = delete_result {
        if moved_to_tombstone {
            let _ = fs::rename(&tombstone, &file_path).await;
        }
        return Err(AppError::Database(error));
    }

    if let Err(error) = tx.commit().await {
        if moved_to_tombstone {
            let _ = fs::rename(&tombstone, &file_path).await;
        }
        return Err(AppError::Database(error));
    }

    if moved_to_tombstone {
        fs::remove_file(&tombstone).await.map_err(|error| {
            storage_error(
                "Metadados removidos, mas não foi possível apagar o arquivo",
                error,
            )
        })?;
    }

    Ok(())
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM inspection_photos")
        .fetch_one(pool)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateMeliponary, CreateSpecies},
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

    async fn seed_inspection(pool: &SqlitePool) -> (String, String) {
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
        let inspection = inspections::create(
            pool,
            CreateInspection {
                colony_id: colony.id.clone(),
                inspected_at: Some("2026-04-10 09:30:00".into()),
                strength: Some("medium".into()),
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
        (colony.id, inspection.id)
    }

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!("meliponariomanager-media-{}", Uuid::new_v4()))
    }

    #[tokio::test]
    async fn imports_photo_to_managed_storage_and_preserves_context() {
        let pool = test_pool().await;
        let (colony_id, inspection_id) = seed_inspection(&pool).await;
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("IMG_0001.JPG");
        fs::write(&source, b"fake-jpeg-content").await.unwrap();
        let data_dir = root.join("data");

        let photo = import_photo(
            &pool,
            &data_dir,
            ImportInspectionPhoto {
                inspection_id: inspection_id.clone(),
                source_path: source.to_string_lossy().into_owned(),
                captured_at: None,
                notes: Some("Vista frontal".into()),
            },
        )
        .await
        .unwrap();

        assert_eq!(photo.inspection_id, inspection_id);
        assert_eq!(photo.colony_id, colony_id);
        assert_eq!(photo.mime_type, "image/jpeg");
        assert_eq!(photo.captured_at, "2026-04-10 09:30:00");
        assert!(data_dir.join(&photo.relative_path).is_file());

        let by_colony = list_by_colony(&pool, &photo.colony_id).await.unwrap();
        assert_eq!(by_colony.len(), 1);

        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn rejects_unsupported_file_extension() {
        let pool = test_pool().await;
        let (_, inspection_id) = seed_inspection(&pool).await;
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("notes.txt");
        fs::write(&source, b"not an image").await.unwrap();

        let result = import_photo(
            &pool,
            &root.join("data"),
            ImportInspectionPhoto {
                inspection_id,
                source_path: source.to_string_lossy().into_owned(),
                captured_at: None,
                notes: None,
            },
        )
        .await;
        assert!(matches!(result, Err(AppError::Validation(_))));

        let count = count(&pool).await.unwrap();
        assert_eq!(count, 0);
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn rejects_unknown_inspection_before_copying_file() {
        let pool = test_pool().await;
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("photo.png");
        fs::write(&source, b"fake-png-content").await.unwrap();
        let data_dir = root.join("data");

        let result = import_photo(
            &pool,
            &data_dir,
            ImportInspectionPhoto {
                inspection_id: "missing".into(),
                source_path: source.to_string_lossy().into_owned(),
                captured_at: None,
                notes: None,
            },
        )
        .await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
        assert!(!data_dir.exists());
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn delete_photo_removes_metadata_and_managed_file() {
        let pool = test_pool().await;
        let (_, inspection_id) = seed_inspection(&pool).await;
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("photo.webp");
        fs::write(&source, b"fake-webp-content").await.unwrap();
        let data_dir = root.join("data");

        let photo = import_photo(
            &pool,
            &data_dir,
            ImportInspectionPhoto {
                inspection_id,
                source_path: source.to_string_lossy().into_owned(),
                captured_at: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let managed_path = data_dir.join(&photo.relative_path);
        assert!(managed_path.exists());

        delete_photo(&pool, &data_dir, &photo.id).await.unwrap();

        assert!(!managed_path.exists());
        assert_eq!(count(&pool).await.unwrap(), 0);
        let _ = fs::remove_dir_all(root).await;
    }
}
