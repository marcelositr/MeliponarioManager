use csv::{ReaderBuilder, StringRecord, Trim};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use std::{collections::HashSet, fs, path::Path};
use tauri::State;
use uuid::Uuid;

const MAX_CSV_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PREVIEW_ROWS: usize = 50;

#[derive(Debug, Clone)]
struct ParsedSpeciesRow {
    row_number: usize,
    common_name: String,
    scientific_name: Option<String>,
    genus: Option<String>,
    notes: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct ExistingSpecies {
    common_name: String,
    scientific_name: Option<String>,
    genus: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesImportPreviewRow {
    row_number: usize,
    common_name: String,
    scientific_name: Option<String>,
    genus: Option<String>,
    status: String,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesImportPreview {
    file_name: String,
    total_rows: usize,
    new_rows: usize,
    duplicate_rows: usize,
    invalid_rows: usize,
    rows: Vec<SpeciesImportPreviewRow>,
    truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesImportResult {
    total_rows: usize,
    imported_rows: usize,
    duplicate_rows: usize,
}

#[derive(Debug)]
enum RowStatus {
    New,
    Duplicate,
    Invalid(String),
}

#[tauri::command]
pub async fn analyze_species_csv(
    pool: State<'_, SqlitePool>,
    source_path: String,
) -> Result<SpeciesImportPreview, String> {
    let path = validate_source_path(&source_path)?;
    let bytes = fs::read(&path).map_err(|error| format!("Não foi possível ler o CSV: {error}"))?;
    let rows = parse_csv(&bytes)?;
    let existing = load_existing(&pool).await?;
    Ok(build_preview(&path, rows, &existing))
}

#[tauri::command]
pub async fn import_species_csv(
    pool: State<'_, SqlitePool>,
    source_path: String,
) -> Result<SpeciesImportResult, String> {
    let path = validate_source_path(&source_path)?;
    let bytes = fs::read(&path).map_err(|error| format!("Não foi possível ler o CSV: {error}"))?;
    let rows = parse_csv(&bytes)?;

    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    let existing = sqlx::query_as::<_, ExistingSpecies>(
        "SELECT common_name, scientific_name, genus FROM species",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|error| error.to_string())?;

    let (statuses, new_rows, duplicate_rows, invalid_rows) = classify_rows(&rows, &existing);
    if invalid_rows > 0 {
        return Err(format!(
            "O CSV possui {invalid_rows} linha(s) inválida(s). Corrija o arquivo antes de importar."
        ));
    }

    let mut imported_rows = 0usize;
    for (row, status) in rows.iter().zip(statuses.iter()) {
        if !matches!(status, RowStatus::New) {
            continue;
        }

        sqlx::query(
            "INSERT INTO species (id, common_name, scientific_name, genus, notes) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&row.common_name)
        .bind(&row.scientific_name)
        .bind(&row.genus)
        .bind(&row.notes)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
        imported_rows += 1;
    }

    debug_assert_eq!(imported_rows, new_rows);
    tx.commit().await.map_err(|error| error.to_string())?;

    Ok(SpeciesImportResult {
        total_rows: rows.len(),
        imported_rows,
        duplicate_rows,
    })
}

fn validate_source_path(source_path: &str) -> Result<std::path::PathBuf, String> {
    let source_path = source_path.trim();
    if source_path.is_empty() {
        return Err("Selecione um arquivo CSV.".to_owned());
    }

    let path = Path::new(source_path);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !extension.eq_ignore_ascii_case("csv") {
        return Err("A importação de espécies aceita somente arquivos .csv.".to_owned());
    }

    let metadata = fs::metadata(path).map_err(|error| format!("Não foi possível acessar o CSV: {error}"))?;
    if !metadata.is_file() {
        return Err("O caminho selecionado não é um arquivo CSV válido.".to_owned());
    }
    if metadata.len() > MAX_CSV_BYTES {
        return Err("O CSV excede o limite de 2 MiB para importação de espécies.".to_owned());
    }

    Ok(path.to_path_buf())
}

async fn load_existing(pool: &SqlitePool) -> Result<Vec<ExistingSpecies>, String> {
    sqlx::query_as::<_, ExistingSpecies>(
        "SELECT common_name, scientific_name, genus FROM species",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())
}

fn build_preview(
    path: &Path,
    rows: Vec<ParsedSpeciesRow>,
    existing: &[ExistingSpecies],
) -> SpeciesImportPreview {
    let (statuses, new_rows, duplicate_rows, invalid_rows) = classify_rows(&rows, existing);
    let total_rows = rows.len();
    let truncated = total_rows > MAX_PREVIEW_ROWS;
    let preview_rows = rows
        .into_iter()
        .zip(statuses)
        .take(MAX_PREVIEW_ROWS)
        .map(|(row, status)| SpeciesImportPreviewRow {
            row_number: row.row_number,
            common_name: row.common_name,
            scientific_name: row.scientific_name,
            genus: row.genus,
            status: match status {
                RowStatus::New => "new".to_owned(),
                RowStatus::Duplicate => "duplicate".to_owned(),
                RowStatus::Invalid(_) => "invalid".to_owned(),
            },
            message: match status {
                RowStatus::Invalid(message) => Some(message),
                RowStatus::Duplicate => Some("Já existe no catálogo ou está repetida no CSV.".to_owned()),
                RowStatus::New => None,
            },
        })
        .collect();

    SpeciesImportPreview {
        file_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("lista.csv")
            .to_owned(),
        total_rows,
        new_rows,
        duplicate_rows,
        invalid_rows,
        rows: preview_rows,
        truncated,
    }
}

fn classify_rows(
    rows: &[ParsedSpeciesRow],
    existing: &[ExistingSpecies],
) -> (Vec<RowStatus>, usize, usize, usize) {
    let mut known = existing
        .iter()
        .map(|item| duplicate_key(&item.common_name, item.scientific_name.as_deref(), item.genus.as_deref()))
        .collect::<HashSet<_>>();

    let mut statuses = Vec::with_capacity(rows.len());
    let mut new_rows = 0usize;
    let mut duplicate_rows = 0usize;
    let mut invalid_rows = 0usize;

    for row in rows {
        if let Some(error) = &row.error {
            invalid_rows += 1;
            statuses.push(RowStatus::Invalid(error.clone()));
            continue;
        }

        let key = duplicate_key(
            &row.common_name,
            row.scientific_name.as_deref(),
            row.genus.as_deref(),
        );
        if known.contains(&key) {
            duplicate_rows += 1;
            statuses.push(RowStatus::Duplicate);
        } else {
            known.insert(key);
            new_rows += 1;
            statuses.push(RowStatus::New);
        }
    }

    (statuses, new_rows, duplicate_rows, invalid_rows)
}

fn duplicate_key(common_name: &str, scientific_name: Option<&str>, genus: Option<&str>) -> String {
    if let Some(scientific_name) = scientific_name.map(str::trim).filter(|value| !value.is_empty()) {
        return format!("scientific:{}", normalize_value(scientific_name));
    }

    format!(
        "common:{}|genus:{}",
        normalize_value(common_name),
        normalize_value(genus.unwrap_or_default())
    )
}

fn parse_csv(bytes: &[u8]) -> Result<Vec<ParsedSpeciesRow>, String> {
    if bytes.is_empty() {
        return Err("O CSV está vazio.".to_owned());
    }

    let delimiter = detect_delimiter(bytes);
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .trim(Trim::All)
        .flexible(true)
        .from_reader(bytes);

    let headers = reader
        .headers()
        .map_err(|error| format!("Cabeçalho CSV inválido: {error}"))?
        .clone();
    let columns = HeaderColumns::from_headers(&headers)?;

    let mut rows = Vec::new();
    for (index, record) in reader.records().enumerate() {
        let row_number = index + 2;
        let record = record.map_err(|error| format!("CSV inválido na linha {row_number}: {error}"))?;
        let common_name = field(&record, columns.common_name).unwrap_or_default();
        let scientific_name = columns.scientific_name.and_then(|column| optional_field(&record, column));
        let genus = columns.genus.and_then(|column| optional_field(&record, column));
        let notes = columns.notes.and_then(|column| optional_field(&record, column));
        let error = if common_name.trim().is_empty() {
            Some("Nome popular é obrigatório.".to_owned())
        } else {
            None
        };

        rows.push(ParsedSpeciesRow {
            row_number,
            common_name: common_name.trim().to_owned(),
            scientific_name,
            genus,
            notes,
            error,
        });
    }

    if rows.is_empty() {
        return Err("O CSV não possui linhas de espécies para importar.".to_owned());
    }

    Ok(rows)
}

#[derive(Debug)]
struct HeaderColumns {
    common_name: usize,
    scientific_name: Option<usize>,
    genus: Option<usize>,
    notes: Option<usize>,
}

impl HeaderColumns {
    fn from_headers(headers: &StringRecord) -> Result<Self, String> {
        let normalized = headers.iter().map(normalize_header).collect::<Vec<_>>();
        let common_name = find_column(&normalized, &["nome_popular", "common_name"])
            .ok_or_else(|| "O CSV precisa da coluna 'nome_popular' (ou 'common_name').".to_owned())?;

        Ok(Self {
            common_name,
            scientific_name: find_column(&normalized, &["nome_cientifico", "scientific_name"]),
            genus: find_column(&normalized, &["genero", "genus"]),
            notes: find_column(&normalized, &["observacoes", "notes"]),
        })
    }
}

fn find_column(headers: &[String], aliases: &[&str]) -> Option<usize> {
    headers
        .iter()
        .position(|header| aliases.iter().any(|alias| header == alias))
}

fn field(record: &StringRecord, column: usize) -> Option<String> {
    record.get(column).map(ToOwned::to_owned)
}

fn optional_field(record: &StringRecord, column: usize) -> Option<String> {
    record
        .get(column)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn detect_delimiter(bytes: &[u8]) -> u8 {
    let first_line = bytes.split(|byte| *byte == b'\n').next().unwrap_or(bytes);
    let semicolons = first_line.iter().filter(|byte| **byte == b';').count();
    let commas = first_line.iter().filter(|byte| **byte == b',').count();
    if semicolons >= commas { b';' } else { b',' }
}

fn normalize_header(value: &str) -> String {
    value
        .trim_start_matches('\u{feff}')
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            ' ' | '-' => '_',
            other => other,
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn normalize_value(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_portuguese_semicolon_csv_with_bom() {
        let csv = "\u{feff}nome_popular;nome_cientifico;genero;observacoes\nJataí;Tetragonisca angustula;Tetragonisca;Entrada manual\n";
        let rows = parse_csv(csv.as_bytes()).expect("CSV should parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].common_name, "Jataí");
        assert_eq!(rows[0].scientific_name.as_deref(), Some("Tetragonisca angustula"));
        assert!(rows[0].error.is_none());
    }

    #[test]
    fn parses_comma_csv_and_quoted_values() {
        let csv = "common_name,scientific_name,genus,notes\nJataí,Tetragonisca angustula,Tetragonisca,\"Texto, com vírgula\"\n";
        let rows = parse_csv(csv.as_bytes()).expect("CSV should parse");
        assert_eq!(rows[0].notes.as_deref(), Some("Texto, com vírgula"));
    }

    #[test]
    fn duplicate_key_prefers_scientific_name() {
        assert_eq!(
            duplicate_key("Jataí", Some("Tetragonisca angustula"), Some("Tetragonisca")),
            "scientific:tetragonisca angustula"
        );
        assert_eq!(
            duplicate_key("Jataí", None, Some("Tetragonisca")),
            "common:jataí|genus:tetragonisca"
        );
    }

    #[test]
    fn classifies_existing_and_repeated_rows_as_duplicates() {
        let existing = vec![ExistingSpecies {
            common_name: "Jataí".to_owned(),
            scientific_name: Some("Tetragonisca angustula".to_owned()),
            genus: Some("Tetragonisca".to_owned()),
        }];
        let rows = vec![
            ParsedSpeciesRow {
                row_number: 2,
                common_name: "Jataí".to_owned(),
                scientific_name: Some("Tetragonisca angustula".to_owned()),
                genus: Some("Tetragonisca".to_owned()),
                notes: None,
                error: None,
            },
            ParsedSpeciesRow {
                row_number: 3,
                common_name: "Mandaçaia".to_owned(),
                scientific_name: Some("Melipona quadrifasciata".to_owned()),
                genus: Some("Melipona".to_owned()),
                notes: None,
                error: None,
            },
            ParsedSpeciesRow {
                row_number: 4,
                common_name: "Mandaçaia duplicada".to_owned(),
                scientific_name: Some("Melipona quadrifasciata".to_owned()),
                genus: Some("Melipona".to_owned()),
                notes: None,
                error: None,
            },
        ];

        let (_, new_rows, duplicate_rows, invalid_rows) = classify_rows(&rows, &existing);
        assert_eq!(new_rows, 1);
        assert_eq!(duplicate_rows, 2);
        assert_eq!(invalid_rows, 0);
    }
}
