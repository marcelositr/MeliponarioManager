use crate::repository::AppError;
use serde::Serialize;
use sqlx::SqlitePool;
use std::{
    collections::HashSet,
    fs,
    path::{Component, Path, PathBuf},
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedFileIssue {
    pub kind: String,
    pub record_id: String,
    pub label: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedFilesDiagnostic {
    pub expected_files: usize,
    pub present_files: usize,
    pub missing_files: Vec<ManagedFileIssue>,
    pub orphan_files: Vec<String>,
}

pub(crate) fn absolute_media_path(
    data_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, AppError> {
    let path = Path::new(relative_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AppError::Validation(
            "O caminho armazenado do arquivo é inválido.".to_owned(),
        ));
    }

    let first = path.components().next().and_then(|component| match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    });
    if first != Some("media") {
        return Err(AppError::Validation(
            "O arquivo está fora da área gerenciada da aplicação.".to_owned(),
        ));
    }

    Ok(data_dir.join(path))
}

pub(crate) fn ensure_managed_prefix(
    relative_path: &str,
    prefix: &[&str],
) -> Result<(), AppError> {
    let components: Vec<_> = Path::new(relative_path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect();
    if components.len() < prefix.len()
        || !prefix
            .iter()
            .zip(components.iter())
            .all(|(expected, actual)| expected == actual)
    {
        return Err(AppError::Validation(
            "O arquivo está fora da área gerenciada esperada.".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn file_exists(data_dir: &Path, relative_path: &str) -> bool {
    absolute_media_path(data_dir, relative_path)
        .ok()
        .is_some_and(|path| path.is_file())
}

fn public_open_error(action: &str) -> String {
    format!("Não foi possível {action} este arquivo. Verifique se ele ainda existe e tente novamente.")
}

fn open_path(app: &AppHandle, relative_path: &str, reveal: bool) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    let path = absolute_media_path(&data_dir, relative_path).map_err(|error| error.to_string())?;
    if !path.is_file() {
        return Err("Arquivo não encontrado na área gerenciada da aplicação.".to_owned());
    }

    if reveal {
        app.opener()
            .reveal_item_in_dir(&path)
            .map_err(|_| public_open_error("mostrar"))
    } else {
        app.opener()
            .open_path(&path, None::<&str>)
            .map_err(|_| public_open_error("abrir"))
    }
}

#[tauri::command]
pub async fn open_managed_attachment(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    attachment_id: String,
) -> Result<(), String> {
    let relative_path: Option<String> =
        sqlx::query_scalar("SELECT relative_path FROM managed_attachments WHERE id = ?")
            .bind(attachment_id.trim())
            .fetch_optional(&*pool)
            .await
            .map_err(|_| "Não foi possível consultar o arquivo.".to_owned())?;
    let relative_path = relative_path.ok_or_else(|| "Anexo não encontrado.".to_owned())?;
    ensure_managed_prefix(&relative_path, &["media", "attachments", "meliponaries"])
        .map_err(|error| error.to_string())?;
    open_path(&app, &relative_path, false)
}

#[tauri::command]
pub async fn reveal_managed_attachment(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    attachment_id: String,
) -> Result<(), String> {
    let relative_path: Option<String> =
        sqlx::query_scalar("SELECT relative_path FROM managed_attachments WHERE id = ?")
            .bind(attachment_id.trim())
            .fetch_optional(&*pool)
            .await
            .map_err(|_| "Não foi possível consultar o arquivo.".to_owned())?;
    let relative_path = relative_path.ok_or_else(|| "Anexo não encontrado.".to_owned())?;
    ensure_managed_prefix(&relative_path, &["media", "attachments", "meliponaries"])
        .map_err(|error| error.to_string())?;
    open_path(&app, &relative_path, true)
}

#[tauri::command]
pub async fn open_inspection_photo(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    photo_id: String,
) -> Result<(), String> {
    let relative_path: Option<String> =
        sqlx::query_scalar("SELECT relative_path FROM inspection_photos WHERE id = ?")
            .bind(photo_id.trim())
            .fetch_optional(&*pool)
            .await
            .map_err(|_| "Não foi possível consultar a foto.".to_owned())?;
    let relative_path = relative_path.ok_or_else(|| "Foto não encontrada.".to_owned())?;
    ensure_managed_prefix(&relative_path, &["media", "inspections"])
        .map_err(|error| error.to_string())?;
    open_path(&app, &relative_path, false)
}

#[tauri::command]
pub async fn reveal_inspection_photo(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    photo_id: String,
) -> Result<(), String> {
    let relative_path: Option<String> =
        sqlx::query_scalar("SELECT relative_path FROM inspection_photos WHERE id = ?")
            .bind(photo_id.trim())
            .fetch_optional(&*pool)
            .await
            .map_err(|_| "Não foi possível consultar a foto.".to_owned())?;
    let relative_path = relative_path.ok_or_else(|| "Foto não encontrada.".to_owned())?;
    ensure_managed_prefix(&relative_path, &["media", "inspections"])
        .map_err(|error| error.to_string())?;
    open_path(&app, &relative_path, true)
}

fn collect_files(root: &Path, data_dir: &Path, output: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, data_dir, output);
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(data_dir) {
                output.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

#[tauri::command]
pub async fn diagnose_managed_files(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<ManagedFilesDiagnostic, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;

    let attachments: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, original_name, relative_path FROM managed_attachments ORDER BY created_at, id",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| "Não foi possível verificar os anexos registrados.".to_owned())?;
    let photos: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, original_name, relative_path FROM inspection_photos ORDER BY created_at, id",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| "Não foi possível verificar as fotos registradas.".to_owned())?;

    let mut expected = HashSet::new();
    let mut missing_files = Vec::new();
    let mut present_files = 0usize;

    for (kind, records) in [("Anexo", attachments), ("Foto", photos)] {
        for (record_id, label, relative_path) in records {
            expected.insert(relative_path.clone());
            if file_exists(&data_dir, &relative_path) {
                present_files += 1;
            } else {
                missing_files.push(ManagedFileIssue {
                    kind: kind.to_owned(),
                    record_id,
                    label,
                    relative_path,
                });
            }
        }
    }

    let mut physical_files = Vec::new();
    collect_files(&data_dir.join("media").join("inspections"), &data_dir, &mut physical_files);
    collect_files(
        &data_dir.join("media").join("attachments"),
        &data_dir,
        &mut physical_files,
    );
    physical_files.sort();
    physical_files.dedup();

    let orphan_files = physical_files
        .into_iter()
        .filter(|relative_path| !expected.contains(relative_path))
        .collect::<Vec<_>>();

    Ok(ManagedFilesDiagnostic {
        expected_files: expected.len(),
        present_files,
        missing_files,
        orphan_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_and_traversal_paths() {
        let root = Path::new("/tmp/app");
        assert!(absolute_media_path(root, "/etc/passwd").is_err());
        assert!(absolute_media_path(root, "media/../secret").is_err());
        assert!(absolute_media_path(root, "../media/file").is_err());
    }

    #[test]
    fn accepts_relative_media_paths_only() {
        let root = Path::new("/tmp/app");
        let path = absolute_media_path(root, "media/attachments/meliponaries/a/file.pdf")
            .expect("managed path");
        assert_eq!(
            path,
            root.join("media/attachments/meliponaries/a/file.pdf")
        );
        assert!(ensure_managed_prefix(
            "media/attachments/meliponaries/a/file.pdf",
            &["media", "attachments", "meliponaries"]
        )
        .is_ok());
        assert!(ensure_managed_prefix(
            "media/inspections/a/file.jpg",
            &["media", "attachments", "meliponaries"]
        )
        .is_err());
    }
}
