use super::*;

fn blob_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

async fn export_rows(pool: &SqlitePool, sql: &'static str) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(sql).fetch_all(pool).await.map_err(|_| {
        "Não foi possível consultar uma estrutura para a exportação JSON.".to_owned()
    })?;
    let mut output = Vec::with_capacity(rows.len());
    for row in rows {
        let mut object = Map::new();
        for (index, column) in row.columns().iter().enumerate() {
            let value = match column.type_info().name() {
                "INTEGER" => row
                    .try_get::<Option<i64>, _>(index)
                    .map(|value| value.map_or(Value::Null, Value::from)),
                "REAL" => row
                    .try_get::<Option<f64>, _>(index)
                    .map(|value| value.map_or(Value::Null, Value::from)),
                "BLOB" => row.try_get::<Option<Vec<u8>>, _>(index).map(|value| {
                    value.map_or(Value::Null, |bytes| Value::String(blob_hex(&bytes)))
                }),
                _ => row
                    .try_get::<Option<String>, _>(index)
                    .map(|value| value.map_or(Value::Null, Value::String)),
            }
            .map_err(|_| {
                "Não foi possível serializar uma estrutura para a exportação JSON.".to_owned()
            })?;
            object.insert(column.name().to_owned(), value);
        }
        output.push(Value::Object(object));
    }
    Ok(output)
}

pub(super) async fn portable_tables(pool: &SqlitePool) -> Result<PortableTables, String> {
    Ok(PortableTables {
        meliponaries: export_rows(pool, "SELECT * FROM meliponaries ORDER BY created_at, id")
            .await?,
        species: export_rows(pool, "SELECT * FROM species ORDER BY created_at, id").await?,
        boxes: export_rows(pool, "SELECT * FROM boxes ORDER BY created_at, id").await?,
        box_state_records: export_rows(
            pool,
            "SELECT * FROM box_state_records ORDER BY occurred_at, created_at, id",
        )
        .await?,
        colonies: export_rows(pool, "SELECT * FROM colonies ORDER BY created_at, id").await?,
        colony_box_occupancies: export_rows(
            pool,
            "SELECT * FROM colony_box_occupancies ORDER BY started_at, created_at, id",
        )
        .await?,
        inspections: export_rows(
            pool,
            "SELECT * FROM inspections ORDER BY inspected_at, created_at, id",
        )
        .await?,
        inspection_photos: export_rows(
            pool,
            "SELECT * FROM inspection_photos ORDER BY created_at, id",
        )
        .await?,
        feedings: export_rows(
            pool,
            "SELECT * FROM feedings ORDER BY fed_at, created_at, id",
        )
        .await?,
        production_records: export_rows(
            pool,
            "SELECT * FROM production_records ORDER BY harvested_at, created_at, id",
        )
        .await?,
        box_maintenance_records: export_rows(
            pool,
            "SELECT * FROM box_maintenance_records ORDER BY maintained_at, created_at, id",
        )
        .await?,
        colony_events: export_rows(
            pool,
            "SELECT * FROM colony_events ORDER BY occurred_at, created_at, id",
        )
        .await?,
        colony_divisions: export_rows(
            pool,
            "SELECT * FROM colony_divisions ORDER BY performed_at, created_at, id",
        )
        .await?,
        colony_movements: export_rows(
            pool,
            "SELECT * FROM colony_movements ORDER BY moved_at, created_at, id",
        )
        .await?,
        transport_returns: export_rows(
            pool,
            "SELECT * FROM transport_returns ORDER BY returned_at, created_at, id",
        )
        .await?,
        movement_documents: export_rows(
            pool,
            "SELECT * FROM movement_documents ORDER BY created_at, id",
        )
        .await?,
        colony_lifecycle_records: export_rows(
            pool,
            "SELECT * FROM colony_lifecycle_records ORDER BY occurred_at, created_at, id",
        )
        .await?,
        scheduled_tasks: export_rows(
            pool,
            "SELECT * FROM scheduled_tasks ORDER BY created_at, id",
        )
        .await?,
        audit_records: export_rows(
            pool,
            "SELECT * FROM audit_records ORDER BY changed_at, created_at, id",
        )
        .await?,
        managed_attachments: export_rows(
            pool,
            "SELECT * FROM managed_attachments ORDER BY created_at, id",
        )
        .await?,
    })
}

pub async fn export_portable_json(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<GeneratedArtifact, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    let created_at = timestamp(&pool).await?;
    let export_dir = data_dir.join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|_| "Não foi possível preparar a pasta de exportações.".to_owned())?;
    let schema_version = database_schema_version(&pool).await?;
    let export = PortableExport {
        format: PORTABLE_FORMAT,
        format_version: PORTABLE_FORMAT_VERSION,
        generated_at: created_at.clone(),
        app_version: env!("CARGO_PKG_VERSION"),
        schema_version,
        assets_embedded: false,
        tables: portable_tables(&pool).await?,
    };
    let bytes = serde_json::to_vec_pretty(&export)
        .map_err(|_| "Não foi possível serializar a exportação JSON.".to_owned())?;
    let path = export_dir.join(format!("estrutura-{created_at}.json"));
    fs::write(&path, bytes).map_err(|_| "Não foi possível gravar a exportação JSON.".to_owned())?;
    Ok(GeneratedArtifact {
        kind: "json".to_owned(),
        path: path.to_string_lossy().into_owned(),
        created_at,
    })
}

pub async fn generate_management_report(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<GeneratedArtifact, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Não foi possível localizar os dados da aplicação.".to_owned())?;
    let created_at = timestamp(&pool).await?;
    let export_dir = data_dir.join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|_| "Não foi possível preparar a pasta de relatórios.".to_owned())?;
    let summary = repository::core_summary(&pool)
        .await
        .map_err(|_| "Não foi possível consultar o resumo do plantel.".to_owned())?;
    let overview = dashboard::overview(&pool)
        .await
        .map_err(|_| "Não foi possível consultar a visão operacional.".to_owned())?;

    let mut report = format!(
        "# Relatório do MeliponarioManager\
\
Gerado em: {created_at}\
\
## Estrutura\
\
- Meliponários: {}\
- Espécies: {}\
- Colônias: {}\
- Caixas: {}\
- Caixas ocupadas: {}\
- Caixas ativas e livres: {}\
\
## Situação das colônias\
",
        summary.meliponaries,
        summary.species,
        summary.colonies,
        summary.boxes,
        overview.occupied_boxes,
        overview.free_boxes
    );
    for item in &overview.colony_statuses {
        report.push_str(&format!(
            "- {}: {}\
",
            item.label, item.count
        ));
    }
    report.push_str(
        "\
## Distribuição por espécie\
",
    );
    for item in &overview.species_distribution {
        report.push_str(&format!(
            "- {}: {}\
",
            item.label, item.count
        ));
    }
    report.push_str(&format!(
        "\
## Pendências\
\
Alertas atuais: {}\
",
        overview.alerts.len()
    ));
    for alert in overview.alerts.iter().take(20) {
        let context = if let Some(colony_code) = alert.colony_code.as_deref() {
            format!("Colônia {colony_code}")
        } else if let Some(box_code) = alert.box_code.as_deref() {
            format!("Caixa {box_code}")
        } else {
            "Meliponário".to_owned()
        };
        report.push_str(&format!(
            "- {context}: {}{}\
",
            alert.title,
            alert
                .due_at
                .as_ref()
                .map(|value| format!(" ({value})"))
                .unwrap_or_default()
        ));
    }

    let path = export_dir.join(format!("relatorio-{created_at}.md"));
    fs::write(&path, report).map_err(|_| "Não foi possível gravar o relatório.".to_owned())?;
    Ok(GeneratedArtifact {
        kind: "report".to_owned(),
        path: path.to_string_lossy().into_owned(),
        created_at,
    })
}
