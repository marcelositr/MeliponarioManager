use super::*;

pub async fn core_summary(pool: &SqlitePool) -> Result<CoreSummary, AppError> {
    Ok(sqlx::query_as::<_, CoreSummary>(
        "SELECT
            (SELECT COUNT(*) FROM meliponaries) AS meliponaries,
            (SELECT COUNT(*) FROM species) AS species,
            (SELECT COUNT(*) FROM colonies) AS colonies,
            (SELECT COUNT(*) FROM boxes) AS boxes",
    )
    .fetch_one(pool)
    .await?)
}

pub async fn create_meliponary(
    pool: &SqlitePool,
    input: CreateMeliponary,
) -> Result<Meliponary, AppError> {
    let id = Uuid::new_v4().to_string();
    let name = required(&input.name, "Nome do meliponário")?;
    crate::identity::ensure_meliponary_name_available(pool, &name, None).await?;

    sqlx::query(
        "INSERT INTO meliponaries (id, name, responsible_name, location, notes)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(optional(&input.responsible_name))
    .bind(optional(&input.location))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    Ok(sqlx::query_as::<_, Meliponary>(
        "SELECT id, name, responsible_name, location, notes, archived_at, archive_reason, created_at
         FROM meliponaries WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_meliponaries(pool: &SqlitePool) -> Result<Vec<Meliponary>, AppError> {
    Ok(sqlx::query_as::<_, Meliponary>(
        "SELECT id, name, responsible_name, location, notes, archived_at, archive_reason, created_at
         FROM meliponaries
         ORDER BY archived_at IS NOT NULL, name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_species(pool: &SqlitePool, input: CreateSpecies) -> Result<Species, AppError> {
    let id = Uuid::new_v4().to_string();
    let common_name = required(&input.common_name, "Nome popular")?;
    let scientific_name = optional(&input.scientific_name);
    let genus = optional(&input.genus);
    crate::identity::ensure_species_identity_available(
        pool,
        &common_name,
        scientific_name.as_deref(),
        genus.as_deref(),
        None,
    )
    .await?;

    sqlx::query(
        "INSERT INTO species (id, common_name, scientific_name, genus, notes)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(common_name)
    .bind(scientific_name)
    .bind(genus)
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    Ok(sqlx::query_as::<_, Species>(
        "SELECT id, common_name, scientific_name, genus, notes, archived_at, archive_reason, created_at
         FROM species WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_species(pool: &SqlitePool) -> Result<Vec<Species>, AppError> {
    Ok(sqlx::query_as::<_, Species>(
        "SELECT id, common_name, scientific_name, genus, notes, archived_at, archive_reason, created_at
         FROM species
         ORDER BY archived_at IS NOT NULL, common_name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_box(pool: &SqlitePool, input: CreateHiveBox) -> Result<HiveBox, AppError> {
    let id = Uuid::new_v4().to_string();
    let meliponary_id = required(&input.meliponary_id, "Meliponário")?;
    let code = required(&input.code, "Identificação da caixa")?;

    let meliponary_archived: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM meliponaries WHERE id = ?")
            .bind(&meliponary_id)
            .fetch_optional(pool)
            .await?;
    match meliponary_archived {
        None => return Err(AppError::NotFound("Meliponário não encontrado.".to_owned())),
        Some(Some(_)) => {
            return Err(AppError::Validation(
                "Um meliponário arquivado não pode receber novas caixas.".to_owned(),
            ))
        }
        Some(None) => {}
    }
    crate::identity::ensure_box_code_available(pool, &meliponary_id, &code, None).await?;

    sqlx::query(
        "INSERT INTO boxes (id, meliponary_id, code, model, material, location_note, notes)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(meliponary_id)
    .bind(code)
    .bind(optional(&input.model))
    .bind(optional(&input.material))
    .bind(optional(&input.location_note))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    get_box(pool, &id).await
}

async fn get_box(pool: &SqlitePool, id: &str) -> Result<HiveBox, AppError> {
    Ok(sqlx::query_as::<_, HiveBox>(
        "SELECT b.id, b.meliponary_id, b.code, b.model, b.material, b.location_note,
                b.status, b.notes, c.code AS current_colony_code, b.created_at
         FROM boxes b
         LEFT JOIN colony_box_occupancies o ON o.box_id = b.id AND o.ended_at IS NULL
         LEFT JOIN colonies c ON c.id = o.colony_id
         WHERE b.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_boxes(pool: &SqlitePool) -> Result<Vec<HiveBox>, AppError> {
    Ok(sqlx::query_as::<_, HiveBox>(
        "SELECT b.id, b.meliponary_id, b.code, b.model, b.material, b.location_note,
                b.status, b.notes, c.code AS current_colony_code, b.created_at
         FROM boxes b
         LEFT JOIN colony_box_occupancies o ON o.box_id = b.id AND o.ended_at IS NULL
         LEFT JOIN colonies c ON c.id = o.colony_id
         ORDER BY b.code COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_colony(pool: &SqlitePool, input: CreateColony) -> Result<Colony, AppError> {
    let id = Uuid::new_v4().to_string();
    let meliponary_id = required(&input.meliponary_id, "Meliponário")?;
    let species_id = required(&input.species_id, "Espécie")?;
    let code = required(&input.code, "Identificação da colônia")?;
    let origin_type = optional(&input.origin_type).unwrap_or_else(|| "historical".to_owned());

    let meliponary_archived: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM meliponaries WHERE id = ?")
            .bind(&meliponary_id)
            .fetch_optional(pool)
            .await?;
    match meliponary_archived {
        None => return Err(AppError::NotFound("Meliponário não encontrado.".to_owned())),
        Some(Some(_)) => {
            return Err(AppError::Validation(
                "Um meliponário arquivado não pode receber novas colônias.".to_owned(),
            ))
        }
        Some(None) => {}
    }

    let species_archived: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM species WHERE id = ?")
            .bind(&species_id)
            .fetch_optional(pool)
            .await?;
    match species_archived {
        None => return Err(AppError::NotFound("Espécie não encontrada.".to_owned())),
        Some(Some(_)) => {
            return Err(AppError::Validation(
                "Uma espécie arquivada não pode ser usada em uma nova colônia.".to_owned(),
            ))
        }
        Some(None) => {}
    }
    crate::identity::ensure_colony_code_available(pool, &meliponary_id, &code, None).await?;

    sqlx::query(
        "INSERT INTO colonies (
            id, meliponary_id, species_id, code, origin_type, origin_notes,
            installed_at, mother_colony_id, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(meliponary_id)
    .bind(species_id)
    .bind(code)
    .bind(origin_type)
    .bind(optional(&input.origin_notes))
    .bind(optional(&input.installed_at))
    .bind(optional(&input.mother_colony_id))
    .bind(optional(&input.notes))
    .execute(pool)
    .await?;

    get_colony(pool, &id).await
}

async fn get_colony(pool: &SqlitePool, id: &str) -> Result<Colony, AppError> {
    Ok(sqlx::query_as::<_, Colony>(
        "SELECT c.id, c.meliponary_id, c.species_id, c.code, c.origin_type,
                c.origin_notes, c.installed_at, c.status, c.mother_colony_id, c.notes,
                b.code AS current_box_code, c.created_at
         FROM colonies c
         LEFT JOIN colony_box_occupancies o ON o.colony_id = c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id = o.box_id
         WHERE c.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_colonies(pool: &SqlitePool) -> Result<Vec<Colony>, AppError> {
    Ok(sqlx::query_as::<_, Colony>(
        "SELECT c.id, c.meliponary_id, c.species_id, c.code, c.origin_type,
                c.origin_notes, c.installed_at, c.status, c.mother_colony_id, c.notes,
                b.code AS current_box_code, c.created_at
         FROM colonies c
         LEFT JOIN colony_box_occupancies o ON o.colony_id = c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id = o.box_id
         ORDER BY c.code COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}
