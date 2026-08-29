use super::{
    optional, resolve_filter, ProductionAggregate, ProductionReport, ProductionReportInput,
    ProductionReportRow, ResolvedFilter,
};
use crate::repository::AppError;
use sqlx::SqlitePool;
use std::collections::BTreeMap;

pub(super) async fn production_report(
    pool: &SqlitePool,
    input: &ProductionReportInput,
) -> Result<ProductionReport, AppError> {
    let filter = resolve_filter(pool, &input.filter).await?;
    let species_id = optional(&input.species_id);
    let colony_id = optional(&input.colony_id);
    let product_type = optional(&input.product_type);

    if let Some(id) = colony_id.as_deref() {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
            .bind(id)
            .fetch_one(pool)
            .await?;
        if !exists {
            return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
        }
    }

    let rows = sqlx::query_as::<_, ProductionReportRow>(
        "SELECT p.id, p.harvested_at,
                m.id AS meliponary_id, m.name AS meliponary_name,
                c.id AS colony_id, c.code AS colony_code,
                s.id AS species_id, s.common_name AS species_name,
                p.product_type, p.quantity, p.unit, p.purpose, p.notes
         FROM production_records p
         JOIN colonies c ON c.id = p.colony_id
         JOIN meliponaries m ON m.id = c.meliponary_id
         JOIN species s ON s.id = c.species_id
         WHERE p.voided_at IS NULL
           AND p.harvested_at >= ? AND p.harvested_at <= ?
           AND (? IS NULL OR c.meliponary_id = ?)
           AND (? IS NULL OR c.species_id = ?)
           AND (? IS NULL OR c.id = ?)
           AND (? IS NULL OR p.product_type = ?)
         ORDER BY p.harvested_at, c.code COLLATE NOCASE, p.id",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .bind(species_id.as_deref())
    .bind(species_id.as_deref())
    .bind(colony_id.as_deref())
    .bind(colony_id.as_deref())
    .bind(product_type.as_deref())
    .bind(product_type.as_deref())
    .fetch_all(pool)
    .await?;

    let by_product_unit = aggregate(&rows, |row| product_label(&row.product_type).to_owned());
    let by_colony = aggregate(&rows, |row| row.colony_code.clone());
    let by_meliponary = aggregate(&rows, |row| row.meliponary_name.clone());
    let by_species = aggregate(&rows, |row| row.species_name.clone());

    Ok(ProductionReport {
        context: filter.context,
        rows,
        by_product_unit,
        by_colony,
        by_meliponary,
        by_species,
    })
}

pub(super) async fn production_summary(
    pool: &SqlitePool,
    filter: &ResolvedFilter,
) -> Result<Vec<ProductionAggregate>, AppError> {
    let rows: Vec<(String, String, f64)> = sqlx::query_as(
        "SELECT p.product_type, p.unit, SUM(p.quantity) AS quantity
         FROM production_records p
         JOIN colonies c ON c.id = p.colony_id
         WHERE p.voided_at IS NULL
           AND p.harvested_at >= ? AND p.harvested_at <= ?
           AND (? IS NULL OR c.meliponary_id = ?)
         GROUP BY p.product_type, p.unit
         ORDER BY p.product_type, p.unit",
    )
    .bind(&filter.start_at)
    .bind(&filter.end_at)
    .bind(filter.meliponary_id.as_deref())
    .bind(filter.meliponary_id.as_deref())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(product_type, unit, quantity)| ProductionAggregate {
            group_label: product_label(&product_type).to_owned(),
            product_type,
            unit,
            quantity,
        })
        .collect())
}

fn aggregate<F>(rows: &[ProductionReportRow], label: F) -> Vec<ProductionAggregate>
where
    F: Fn(&ProductionReportRow) -> String,
{
    let mut totals: BTreeMap<(String, String, String), f64> = BTreeMap::new();
    for row in rows {
        *totals
            .entry((label(row), row.product_type.clone(), row.unit.clone()))
            .or_default() += row.quantity;
    }
    totals
        .into_iter()
        .map(
            |((group_label, product_type, unit), quantity)| ProductionAggregate {
                group_label,
                product_type,
                unit,
                quantity,
            },
        )
        .collect()
}

pub(crate) fn product_label(value: &str) -> &str {
    match value {
        "honey" => "Mel",
        "pollen" => "Pólen",
        "propolis" => "Própolis",
        "wax" => "Cera",
        "cerumen" => "Cerume",
        _ => "Outro produto",
    }
}
