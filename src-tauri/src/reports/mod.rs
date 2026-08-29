mod colony;
mod csv;
mod operational;
mod production;

#[cfg(test)]
mod tests;

use crate::{repository::AppError, time};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportFilter {
    pub start_date: String,
    pub end_date: String,
    pub meliponary_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedFilter {
    pub start_at: String,
    pub end_at: String,
    pub meliponary_id: Option<String>,
    pub context: ReportContext,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportContext {
    pub start_date: String,
    pub end_date: String,
    pub generated_at: String,
    pub meliponary_id: Option<String>,
    pub meliponary_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountByLabel {
    pub key: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionAggregate {
    pub group_label: String,
    pub product_type: String,
    pub unit: String,
    pub quantity: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalPlantel {
    pub total_colonies: i64,
    pub active_colonies: i64,
    pub active_boxes: i64,
    pub current_occupancies: i64,
    pub colony_statuses: Vec<CountByLabel>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalManagement {
    pub inspections: i64,
    pub feedings: i64,
    pub maintenance: i64,
    pub events: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalMovements {
    pub transfers: i64,
    pub temporary_started: i64,
    pub returns_completed: i64,
    pub temporary_open_at_end: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgendaMetrics {
    pub created: i64,
    pub scheduled: i64,
    pub completed: i64,
    pub completed_on_time: i64,
    pub completed_late: i64,
    pub cancelled: i64,
    pub skipped: i64,
    pub rescheduled: i64,
    pub overdue_pending: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalReport {
    pub context: ReportContext,
    pub plantel: OperationalPlantel,
    pub management: OperationalManagement,
    pub production: Vec<ProductionAggregate>,
    pub movements: OperationalMovements,
    pub agenda: AgendaMetrics,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionReportInput {
    pub filter: ReportFilter,
    pub species_id: Option<String>,
    pub colony_id: Option<String>,
    pub product_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductionReportRow {
    pub id: String,
    pub harvested_at: String,
    pub meliponary_id: String,
    pub meliponary_name: String,
    pub colony_id: String,
    pub colony_code: String,
    pub species_id: String,
    pub species_name: String,
    pub product_type: String,
    pub quantity: f64,
    pub unit: String,
    pub purpose: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionReport {
    pub context: ReportContext,
    pub rows: Vec<ProductionReportRow>,
    pub by_product_unit: Vec<ProductionAggregate>,
    pub by_colony: Vec<ProductionAggregate>,
    pub by_meliponary: Vec<ProductionAggregate>,
    pub by_species: Vec<ProductionAggregate>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CostReportRow {
    pub maintained_at: String,
    pub meliponary_name: String,
    pub box_code: String,
    pub colony_code: Option<String>,
    pub maintenance_type: String,
    pub performed_by: Option<String>,
    pub description: Option<String>,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostReport {
    pub context: ReportContext,
    pub source_description: String,
    pub currency_assumption: String,
    pub total: f64,
    pub rows: Vec<CostReportRow>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgendaReportRow {
    pub id: String,
    pub scheduled_for: String,
    pub meliponary_name: String,
    pub colony_code: Option<String>,
    pub box_code: Option<String>,
    pub task_type: String,
    pub title: String,
    pub status: String,
    pub completed_at: Option<String>,
    pub timing: String,
    pub rescheduled_from_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgendaReport {
    pub context: ReportContext,
    pub metrics: AgendaMetrics,
    pub rows: Vec<AgendaReportRow>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColonyReportInput {
    pub filter: ReportFilter,
    pub colony_id: String,
    pub include_audit: bool,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyReportIdentity {
    pub colony_id: String,
    pub colony_code: String,
    pub meliponary_name: String,
    pub species_name: String,
    pub scientific_name: Option<String>,
    pub origin_type: String,
    pub origin_notes: Option<String>,
    pub installed_at: Option<String>,
    pub status: String,
    pub current_box_code: Option<String>,
    pub mother_colony_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyHistoryRow {
    pub source_id: String,
    pub occurred_at: String,
    pub category: String,
    pub title: String,
    pub details: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColonyReport {
    pub context: ReportContext,
    pub identity: ColonyReportIdentity,
    pub include_audit: bool,
    pub photo_count: i64,
    pub timeline: Vec<ColonyHistoryRow>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeliponaryReport {
    pub context: ReportContext,
    pub operational: OperationalReport,
    pub maintenance_cost_total: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvExportInput {
    pub kind: String,
    pub path: String,
    pub filter: ReportFilter,
    pub colony_id: Option<String>,
    pub include_audit: Option<bool>,
    pub species_id: Option<String>,
    pub product_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvExportResult {
    pub path: String,
    pub row_count: usize,
}

fn optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) async fn resolve_filter(
    pool: &SqlitePool,
    filter: &ReportFilter,
) -> Result<ResolvedFilter, AppError> {
    let start_at = time::normalize(filter.start_date.trim(), true)?;
    let end_start = time::normalize(filter.end_date.trim(), true)?;
    let end_at = format!("{} 23:59:59", &end_start[..10]);
    if start_at > end_at {
        return Err(AppError::Validation(
            "A data inicial não pode ser posterior à data final.".to_owned(),
        ));
    }

    let meliponary_id = optional(&filter.meliponary_id);
    let meliponary_name = if let Some(id) = meliponary_id.as_deref() {
        sqlx::query_scalar::<_, String>("SELECT name FROM meliponaries WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Meliponário não encontrado.".to_owned()))?
    } else {
        "Todos os meliponários".to_owned()
    };

    let generated_at = time::local_now(pool).await?;
    Ok(ResolvedFilter {
        start_at: start_at.clone(),
        end_at: end_at.clone(),
        meliponary_id: meliponary_id.clone(),
        context: ReportContext {
            start_date: start_at[..10].to_owned(),
            end_date: end_at[..10].to_owned(),
            generated_at,
            meliponary_id,
            meliponary_name,
        },
    })
}

fn message(error: AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn get_operational_report(
    pool: State<'_, SqlitePool>,
    filter: ReportFilter,
) -> Result<OperationalReport, String> {
    operational::operational_report(&pool, &filter)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn get_production_report(
    pool: State<'_, SqlitePool>,
    input: ProductionReportInput,
) -> Result<ProductionReport, String> {
    production::production_report(&pool, &input)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn get_cost_report(
    pool: State<'_, SqlitePool>,
    filter: ReportFilter,
) -> Result<CostReport, String> {
    operational::cost_report(&pool, &filter).await.map_err(message)
}

#[tauri::command]
pub async fn get_agenda_report(
    pool: State<'_, SqlitePool>,
    filter: ReportFilter,
) -> Result<AgendaReport, String> {
    operational::agenda_report(&pool, &filter).await.map_err(message)
}

#[tauri::command]
pub async fn get_colony_report(
    pool: State<'_, SqlitePool>,
    input: ColonyReportInput,
) -> Result<ColonyReport, String> {
    colony::colony_report(&pool, &input).await.map_err(message)
}

#[tauri::command]
pub async fn get_meliponary_report(
    pool: State<'_, SqlitePool>,
    filter: ReportFilter,
) -> Result<MeliponaryReport, String> {
    operational::meliponary_report(&pool, &filter)
        .await
        .map_err(message)
}

#[tauri::command]
pub async fn export_report_csv(
    pool: State<'_, SqlitePool>,
    input: CsvExportInput,
) -> Result<CsvExportResult, String> {
    csv::export(&pool, input).await.map_err(message)
}
