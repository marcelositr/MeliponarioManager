use super::{
    colony, operational, production, ColonyReportInput, CsvExportInput, CsvExportResult,
    ProductionReportInput,
};
use crate::repository::AppError;
use sqlx::SqlitePool;
use std::{fs, path::Path};

pub(super) async fn export(
    pool: &SqlitePool,
    input: CsvExportInput,
) -> Result<CsvExportResult, AppError> {
    let path = input.path.trim();
    if path.is_empty() {
        return Err(AppError::Validation(
            "Escolha um destino para o arquivo CSV.".to_owned(),
        ));
    }
    if !Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("csv"))
    {
        return Err(AppError::Validation(
            "A exportação precisa usar a extensão .csv.".to_owned(),
        ));
    }

    let (content, row_count) = match input.kind.as_str() {
        "production" => {
            let report = production::production_report(
                pool,
                &ProductionReportInput {
                    filter: input.filter.clone(),
                    species_id: input.species_id.clone(),
                    colony_id: input.colony_id.clone(),
                    product_type: input.product_type.clone(),
                },
            )
            .await?;
            let mut csv = String::from(
                "Data;Meliponário;Colônia;Espécie;Produto;Quantidade;Unidade;Finalidade;Observações\n",
            );
            for row in &report.rows {
                push_row(
                    &mut csv,
                    &[
                        text(&row.harvested_at),
                        safe(&row.meliponary_name),
                        safe(&row.colony_code),
                        safe(&row.species_name),
                        safe(production::product_label(&row.product_type)),
                        number(row.quantity),
                        safe(&row.unit),
                        safe(row.purpose.as_deref().unwrap_or("")),
                        safe(row.notes.as_deref().unwrap_or("")),
                    ],
                );
            }
            (csv, report.rows.len())
        }
        "agenda" => {
            let report = operational::agenda_report(pool, &input.filter).await?;
            let mut csv = String::from(
                "Agendado para;Meliponário;Colônia;Caixa;Tipo;Título;Status;Conclusão;Pontualidade\n",
            );
            for row in &report.rows {
                push_row(
                    &mut csv,
                    &[
                        text(&row.scheduled_for),
                        safe(&row.meliponary_name),
                        safe(row.colony_code.as_deref().unwrap_or("")),
                        safe(row.box_code.as_deref().unwrap_or("")),
                        safe(task_type_label(&row.task_type)),
                        safe(&row.title),
                        safe(task_status_label(&row.status)),
                        text(row.completed_at.as_deref().unwrap_or("")),
                        safe(timing_label(&row.timing)),
                    ],
                );
            }
            (csv, report.rows.len())
        }
        "colony" => {
            let colony_id = input
                .colony_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AppError::Validation("Selecione uma colônia para exportar.".to_owned())
                })?;
            let report = colony::colony_report(
                pool,
                &ColonyReportInput {
                    filter: input.filter.clone(),
                    colony_id: colony_id.to_owned(),
                    include_audit: input.include_audit.unwrap_or(false),
                },
            )
            .await?;
            let mut csv = String::from("Data;Colônia;Categoria;Título;Detalhes;Estado\n");
            for row in &report.timeline {
                push_row(
                    &mut csv,
                    &[
                        text(&row.occurred_at),
                        safe(&report.identity.colony_code),
                        safe(&row.category),
                        safe(&row.title),
                        safe(row.details.as_deref().unwrap_or("")),
                        safe(state_label(&row.state)),
                    ],
                );
            }
            (csv, report.timeline.len())
        }
        "costs" => {
            let report = operational::cost_report(pool, &input.filter).await?;
            let mut csv = String::from(
                "Data;Meliponário;Caixa;Colônia;Tipo;Responsável;Descrição;Custo (BRL)\n",
            );
            for row in &report.rows {
                push_row(
                    &mut csv,
                    &[
                        text(&row.maintained_at),
                        safe(&row.meliponary_name),
                        safe(&row.box_code),
                        safe(row.colony_code.as_deref().unwrap_or("")),
                        safe(maintenance_type_label(&row.maintenance_type)),
                        safe(row.performed_by.as_deref().unwrap_or("")),
                        safe(row.description.as_deref().unwrap_or("")),
                        number(row.cost),
                    ],
                );
            }
            (csv, report.rows.len())
        }
        _ => {
            return Err(AppError::Validation(
                "Tipo de CSV de relatório não suportado.".to_owned(),
            ))
        }
    };

    fs::write(path, content).map_err(|error| {
        AppError::Validation(format!("Não foi possível salvar o CSV selecionado: {error}"))
    })?;
    Ok(CsvExportResult {
        path: path.to_owned(),
        row_count,
    })
}

#[derive(Debug)]
struct CsvCell {
    value: String,
    protect_formula: bool,
}

fn safe(value: &str) -> CsvCell {
    CsvCell {
        value: value.to_owned(),
        protect_formula: true,
    }
}

fn text(value: &str) -> CsvCell {
    CsvCell {
        value: value.to_owned(),
        protect_formula: false,
    }
}

fn number(value: f64) -> CsvCell {
    CsvCell {
        value: value.to_string(),
        protect_formula: false,
    }
}

fn push_row(target: &mut String, cells: &[CsvCell]) {
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            target.push(';');
        }
        target.push_str(&encode(&cell.value, cell.protect_formula));
    }
    target.push('\n');
}

pub(crate) fn encode(value: &str, protect_formula: bool) -> String {
    let value = if protect_formula && starts_like_formula(value) {
        format!("'{value}")
    } else {
        value.to_owned()
    };
    if value.contains(';') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn starts_like_formula(value: &str) -> bool {
    matches!(
        value.trim_start().chars().next(),
        Some('=' | '+' | '-' | '@')
    )
}

fn task_type_label(value: &str) -> &str {
    match value {
        "inspection" => "Inspeção",
        "feeding" => "Alimentação",
        "maintenance" => "Manutenção",
        _ => "Geral",
    }
}

fn task_status_label(value: &str) -> &str {
    match value {
        "completed" => "Concluída",
        "cancelled" => "Cancelada",
        "skipped" => "Ignorada",
        "rescheduled" => "Reagendada",
        _ => "Pendente",
    }
}

fn timing_label(value: &str) -> &str {
    match value {
        "on_time" => "No prazo",
        "late" => "Após o prazo",
        _ => "Não se aplica",
    }
}

fn state_label(value: &str) -> &str {
    match value {
        "corrected" => "Corrigido",
        "voided" => "Anulado",
        "reversed" => "Revertido",
        "audit" => "Auditoria",
        _ => "Efetivo",
    }
}

fn maintenance_type_label(value: &str) -> &str {
    match value {
        "cleaning" => "Limpeza",
        "repair" => "Reparo",
        "painting" => "Pintura",
        "waterproofing" => "Impermeabilização",
        "roof" => "Cobertura",
        "entrance" => "Entrada",
        "internal_structure" => "Estrutura interna",
        "inspection" => "Revisão da caixa",
        _ => "Outro",
    }
}

#[cfg(test)]
mod csv_unit_tests {
    use super::*;

    #[test]
    fn csv_escapes_delimiter_quotes_newline_accents_and_formula_prefixes() {
        assert_eq!(encode("Jataí", true), "Jataí");
        assert_eq!(encode("a;b", true), "\"a;b\"");
        assert_eq!(encode("a\"b", true), "\"a\"\"b\"");
        assert_eq!(encode("linha 1\nlinha 2", true), "\"linha 1\nlinha 2\"");
        assert_eq!(encode("=1+1", true), "'=1+1");
        assert_eq!(encode(" +SUM(A1:A2)", true), "' +SUM(A1:A2)");
        assert_eq!(encode("-12.5", false), "-12.5");
    }

    #[test]
    fn csv_rows_use_semicolon_and_stable_newline() {
        let mut output = String::new();
        push_row(&mut output, &[safe("A"), safe("B")]);
        assert_eq!(output, "A;B\n");
    }
}
