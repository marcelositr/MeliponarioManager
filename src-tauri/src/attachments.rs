use crate::{managed_files, repository::AppError};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
struct ManagedAttachmentRow {
    id: String,
    meliponary_id: String,
    original_name: String,
    relative_path: String,
    extension: Option<String>,
    mime_type: Option<String>,
    byte_size: i64,
    description: Option<String>,
    notes: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedAttachment {
    pub id: String,
    pub meliponary_id: String,
    pub original_name: String,
    pub extension: Option<String>,
    pub mime_type: Option<String>,
    pub byte_size: i64,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub file_exists: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportManagedAttachment {
    pub meliponary_id: String,
    pub source_path: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManagedAttachment {
    pub id: String,
    pub description: Option<String>,
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

fn storage_error(message: &str) -> AppError {
    AppError::Validation(message.to_owned())
}

fn safe_extension(path: &Path) -> Option<String> {
    let value = path.extension()?.to_str()?.trim().to_ascii_lowercase();
    if value.is_empty()
        || value.len() > 16
        || !value.chars().all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(value)
}

fn mime_type(extension: Option<&str>) -> Option<String> {
    let value = match extension? {
        "pdf" => "application/pdf",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "txt" | "md" | "csv" => "text/plain",
        "json" => "application/json",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odt" => "application/vnd.oasis.opendocument.text",
        _ => return None,
    };
    Some(value.to_owned())
}

fn into_attachment(row: ManagedAttachmentRow, data_dir: &Path) -> ManagedAttachment {
    let file_exists = managed_files::file_exists(data_dir, &row.relative_path);
    ManagedAttachment {
        id: row.id,
        meliponary_id: row.meliponary_id,
        original_name: row.original_name,
        extension: row.extension,
        mime_type: row.mime_type,
        byte_size: row.byte_size,
        description: row.description,
        notes: row.notes,
        created_at: row.created_at,
        file_exists,
    }
}

async fn get_row(pool: &SqlitePool, id: &str) -> Result<ManagedAttachmentRow, AppError> {
    Ok(sqlx::query_as::<_, ManagedAttachmentRow>(
        "SELECT id, meliponary_id, original_name, relative_path, extension, mime_type,
                byte_size, description, notes, created_at
         FROM managed_attachments
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn import(
    pool: &SqlitePool,
    data_dir: &Path,
    input: ImportManagedAttachment,
) -> Result<ManagedAttachment, AppError> {
    let meliponary_id = required(&input.meliponary_id, "Meliponário")?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meliponaries WHERE id = ?)")
            .bind(&meliponary_id)
            .fetch_one(pool)
            .await?;
    if !exists {
        return Err(AppError::NotFound(
            "Meliponário não encontrado.".to_owned(),
        ));
    }

    let source_path = PathBuf::from(required(&input.source_path, "Arquivo de origem")?);
    let metadata = fs::metadata(&source_path)
        .await
        .map_err(|_| storage_error("Não foi possível acessar o arquivo selecionado."))?;
    if !metadata.is_file() {
        return Err(AppError::Validation(
            "O item selecionado precisa ser um arquivo.".to_owned(),
        ));
    }

    let original_name = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Validation("O nome do arquivo é inválido.".to_owned()))?
        .to_owned();
    let byte_size = i64::try_from(metadata.len()).map_err(|_| {
        AppError::Validation("O arquivo é grande demais para ser registrado.".to_owned())
    })?;
    let extension = safe_extension(&source_path);
    let mime_type = mime_type(extension.as_deref());
    let description = optional(&input.description);
    let notes = optional(&input.notes);
    let id = Uuid::new_v4().to_string();

    let target_dir = data_dir
        .join("media")
        .join("attachments")
        .join("meliponaries")
        .join(&meliponary_id);
    fs::create_dir_all(&target_dir)
        .await
        .map_err(|_| storage_error("Não foi possível preparar a área de anexos."))?;

    let stored_name = extension
        .as_ref()
        .map(|extension| format!("{id}.{extension}"))
        .unwrap_or_else(|| id.clone());
    let target_path = target_dir.join(&stored_name);
    let relative_path = format!(
        "media/attachments/meliponaries/{meliponary_id}/{stored_name}"
    );

    fs::copy(&source_path, &target_path)
        .await
        .map_err(|_| storage_error("Não foi possível copiar o arquivo para a área gerenciada."))?;

    let insert_result = sqlx::query(
        "INSERT INTO managed_attachments (
            id, meliponary_id, original_name, relative_path, extension, mime_type,
            byte_size, description, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&meliponary_id)
    .bind(&original_name)
    .bind(&relative_path)
    .bind(extension.as_deref())
    .bind(mime_type.as_deref())
    .bind(byte_size)
    .bind(description.as_deref())
    .bind(notes.as_deref())
    .execute(pool)
    .await;

    if let Err(error) = insert_result {
        let _ = fs::remove_file(&target_path).await;
        return Err(AppError::Database(error));
    }

    Ok(into_attachment(get_row(pool, &id).await?, data_dir))
}

pub async fn list_by_meliponary(
    pool: &SqlitePool,
    data_dir: &Path,
    meliponary_id: &str,
) -> Result<Vec<ManagedAttachment>, AppError> {
    let meliponary_id = required(meliponary_id, "Meliponário")?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meliponaries WHERE id = ?)")
            .bind(&meliponary_id)
            .fetch_one(pool)
            .await?;
    if !exists {
        return Err(AppError::NotFound(
            "Meliponário não encontrado.".to_owned(),
        ));
    }

    let rows = sqlx::query_as::<_, ManagedAttachmentRow>(
        "SELECT id, meliponary_id, original_name, relative_path, extension, mime_type,
                byte_size, description, notes, created_at
         FROM managed_attachments
         WHERE meliponary_id = ?
         ORDER BY created_at DESC, original_name COLLATE NOCASE, id",
    )
    .bind(meliponary_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| into_attachment(row, data_dir))
        .collect())
}

pub async fn update(
    pool: &SqlitePool,
    data_dir: &Path,
    input: UpdateManagedAttachment,
) -> Result<ManagedAttachment, AppError> {
    let id = required(&input.id, "Anexo")?;
    let result = sqlx::query(
        "UPDATE managed_attachments SET description = ?, notes = ? WHERE id = ?",
    )
    .bind(optional(&input.description))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Anexo não encontrado.".to_owned()));
    }
    Ok(into_attachment(get_row(pool, &id).await?, data_dir))
}

pub async fn remove(
    pool: &SqlitePool,
    data_dir: &Path,
    attachment_id: &str,
) -> Result<(), AppError> {
    let attachment_id = required(attachment_id, "Anexo")?;
    let relative_path: Option<String> =
        sqlx::query_scalar("SELECT relative_path FROM managed_attachments WHERE id = ?")
            .bind(&attachment_id)
            .fetch_optional(pool)
            .await?;
    let relative_path =
        relative_path.ok_or_else(|| AppError::NotFound("Anexo não encontrado.".to_owned()))?;
    managed_files::ensure_managed_prefix(
        &relative_path,
        &["media", "attachments", "meliponaries"],
    )?;
    let file_path = managed_files::absolute_media_path(data_dir, &relative_path)?;
    let tombstone = file_path.with_file_name(format!(".deleting-{}", Uuid::new_v4()));

    let moved_to_tombstone = match fs::metadata(&file_path).await {
        Ok(metadata) if metadata.is_file() => {
            fs::rename(&file_path, &tombstone)
                .await
                .map_err(|_| storage_error("Não foi possível preparar a remoção do anexo."))?;
            true
        }
        Ok(_) => {
            return Err(AppError::Validation(
                "O caminho registrado não aponta para um arquivo.".to_owned(),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(_) => {
            return Err(storage_error(
                "Não foi possível verificar o arquivo gerenciado antes da remoção.",
            ));
        }
    };

    let mut tx = pool.begin().await?;
    let delete_result = sqlx::query("DELETE FROM managed_attachments WHERE id = ?")
        .bind(&attachment_id)
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

    if moved_to_tombstone && fs::remove_file(&tombstone).await.is_err() {
        return Err(storage_error(
            "O anexo foi removido do cadastro, mas a limpeza do arquivo físico não pôde ser concluída. Execute o diagnóstico de arquivos.",
        ));
    }

    Ok(())
}

fn public_error(error: AppError, fallback: &str) -> String {
    match error {
        AppError::Validation(message) | AppError::NotFound(message) => message,
        AppError::Database(_) => fallback.to_owned(),
    }
}

#[tauri::command]
pub async fn import_meliponary_attachment(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    input: ImportManagedAttachment,
) -> Result<ManagedAttachment, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    import(&pool, &data_dir, input)
        .await
        .map_err(|error| public_error(error, "Não foi possível registrar o anexo."))
}

#[tauri::command]
pub async fn list_meliponary_attachments(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    meliponary_id: String,
) -> Result<Vec<ManagedAttachment>, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    list_by_meliponary(&pool, &data_dir, &meliponary_id)
        .await
        .map_err(|error| public_error(error, "Não foi possível carregar os anexos."))
}

#[tauri::command]
pub async fn update_meliponary_attachment(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    input: UpdateManagedAttachment,
) -> Result<ManagedAttachment, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    update(&pool, &data_dir, input)
        .await
        .map_err(|error| public_error(error, "Não foi possível atualizar o anexo."))
}

#[tauri::command]
pub async fn remove_meliponary_attachment(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    attachment_id: String,
) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    remove(&pool, &data_dir, &attachment_id)
        .await
        .map_err(|error| public_error(error, "Não foi possível remover o anexo."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::CreateMeliponary, repository};
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

    async fn seed_meliponary(pool: &SqlitePool) -> String {
        repository::create_meliponary(
            pool,
            CreateMeliponary {
                name: "Meliponário Arquivos".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap()
        .id
    }

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!("meliponariomanager-attachments-{}", Uuid::new_v4()))
    }

    #[tokio::test]
    async fn imports_duplicate_original_names_without_collision() {
        let pool = test_pool().await;
        let meliponary_id = seed_meliponary(&pool).await;
        let root = temp_root();
        fs::create_dir_all(root.join("a")).await.unwrap();
        fs::create_dir_all(root.join("b")).await.unwrap();
        let source_a = root.join("a").join("documento.pdf");
        let source_b = root.join("b").join("documento.pdf");
        fs::write(&source_a, b"first").await.unwrap();
        fs::write(&source_b, b"second").await.unwrap();
        let data_dir = root.join("data");

        let first = import(
            &pool,
            &data_dir,
            ImportManagedAttachment {
                meliponary_id: meliponary_id.clone(),
                source_path: source_a.to_string_lossy().into_owned(),
                description: Some("Primeiro".into()),
                notes: None,
            },
        )
        .await
        .unwrap();
        let second = import(
            &pool,
            &data_dir,
            ImportManagedAttachment {
                meliponary_id: meliponary_id.clone(),
                source_path: source_b.to_string_lossy().into_owned(),
                description: Some("Segundo".into()),
                notes: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(first.original_name, "documento.pdf");
        assert_eq!(second.original_name, "documento.pdf");
        assert_ne!(first.id, second.id);
        let stored: Vec<String> =
            sqlx::query_scalar("SELECT relative_path FROM managed_attachments ORDER BY id")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(stored.len(), 2);
        assert_ne!(stored[0], stored[1]);
        assert!(stored.iter().all(|path| path.starts_with(&format!(
            "media/attachments/meliponaries/{meliponary_id}/"
        ))));
        assert!(stored.iter().all(|path| data_dir.join(path).is_file()));
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn missing_file_is_reported_without_deleting_metadata() {
        let pool = test_pool().await;
        let meliponary_id = seed_meliponary(&pool).await;
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("manual.txt");
        fs::write(&source, b"conteudo").await.unwrap();
        let data_dir = root.join("data");
        let attachment = import(
            &pool,
            &data_dir,
            ImportManagedAttachment {
                meliponary_id: meliponary_id.clone(),
                source_path: source.to_string_lossy().into_owned(),
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let relative_path: String =
            sqlx::query_scalar("SELECT relative_path FROM managed_attachments WHERE id = ?")
                .bind(&attachment.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        fs::remove_file(data_dir.join(relative_path)).await.unwrap();

        let items = list_by_meliponary(&pool, &data_dir, &meliponary_id)
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert!(!items[0].file_exists);
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM managed_attachments")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
        let _ = fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn removal_deletes_managed_copy_but_not_original() {
        let pool = test_pool().await;
        let meliponary_id = seed_meliponary(&pool).await;
        let root = temp_root();
        fs::create_dir_all(&root).await.unwrap();
        let source = root.join("origem.csv");
        fs::write(&source, b"a;b\n1;2").await.unwrap();
        let data_dir = root.join("data");
        let attachment = import(
            &pool,
            &data_dir,
            ImportManagedAttachment {
                meliponary_id,
                source_path: source.to_string_lossy().into_owned(),
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let relative_path: String =
            sqlx::query_scalar("SELECT relative_path FROM managed_attachments WHERE id = ?")
                .bind(&attachment.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let managed = data_dir.join(relative_path);
        assert!(managed.is_file());

        remove(&pool, &data_dir, &attachment.id).await.unwrap();
        assert!(!managed.exists());
        assert!(source.is_file());
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM managed_attachments")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let _ = fs::remove_dir_all(root).await;
    }

    #[test]
    fn only_safe_extensions_are_used_in_managed_names() {
        assert_eq!(safe_extension(Path::new("doc.PDF")).as_deref(), Some("pdf"));
        assert_eq!(safe_extension(Path::new("doc.bad-name")), None);
        assert_eq!(safe_extension(Path::new("doc.abcdefghijklmnopq")), None);
    }
}
