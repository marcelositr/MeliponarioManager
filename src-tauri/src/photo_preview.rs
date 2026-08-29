use crate::managed_files;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};
use tokio::fs;

const MAX_PREVIEW_BYTES: u64 = 384 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionPhotoPreview {
    pub file_exists: bool,
    pub mime_type: Option<String>,
    pub bytes: Option<Vec<u8>>,
    pub preview_limited: bool,
}

#[tauri::command]
pub async fn get_inspection_photo_preview(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    photo_id: String,
) -> Result<InspectionPhotoPreview, String> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT relative_path, mime_type FROM inspection_photos WHERE id = ?",
    )
    .bind(photo_id.trim())
    .fetch_optional(&*pool)
    .await
    .map_err(|_| "Não foi possível consultar a foto.".to_owned())?;
    let (relative_path, mime_type) = row.ok_or_else(|| "Foto não encontrada.".to_owned())?;
    managed_files::ensure_managed_prefix(&relative_path, &["media", "inspections"])
        .map_err(|error| error.to_string())?;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    let path = managed_files::absolute_media_path(&data_dir, &relative_path)
        .map_err(|error| error.to_string())?;
    let metadata = match fs::metadata(&path).await {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => {
            return Ok(InspectionPhotoPreview {
                file_exists: false,
                mime_type,
                bytes: None,
                preview_limited: false,
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(InspectionPhotoPreview {
                file_exists: false,
                mime_type,
                bytes: None,
                preview_limited: false,
            });
        }
        Err(_) => return Err("Não foi possível verificar o arquivo da foto.".to_owned()),
    };

    if metadata.len() > MAX_PREVIEW_BYTES {
        return Ok(InspectionPhotoPreview {
            file_exists: true,
            mime_type,
            bytes: None,
            preview_limited: true,
        });
    }

    let bytes = fs::read(&path)
        .await
        .map_err(|_| "Não foi possível carregar a prévia da foto.".to_owned())?;
    Ok(InspectionPhotoPreview {
        file_exists: true,
        mime_type,
        bytes: Some(bytes),
        preview_limited: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_limit_stays_bounded_for_list_usage() {
        assert_eq!(MAX_PREVIEW_BYTES, 384 * 1024);
    }
}
