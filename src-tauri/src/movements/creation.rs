use super::*;

async fn get(pool: &SqlitePool, id: &str) -> Result<ColonyMovement, AppError> {
    Ok(sqlx::query_as::<_, ColonyMovement>(
        "SELECT
            m.id,
            m.colony_id,
            c.code AS colony_code,
            m.movement_type,
            m.moved_at,
            m.from_meliponary_id,
            fm.name AS from_meliponary_name,
            m.to_meliponary_id,
            tm.name AS to_meliponary_name,
            m.from_box_id,
            fb.code AS from_box_code,
            m.to_box_id,
            tb.code AS to_box_code,
            m.destination,
            m.document_reference,
            m.notes,
            m.created_at
         FROM colony_movements m
         JOIN colonies c ON c.id = m.colony_id
         JOIN meliponaries fm ON fm.id = m.from_meliponary_id
         LEFT JOIN meliponaries tm ON tm.id = m.to_meliponary_id
         LEFT JOIN boxes fb ON fb.id = m.from_box_id
         LEFT JOIN boxes tb ON tb.id = m.to_box_id
         WHERE m.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

async fn historical_box(
    pool: &SqlitePool,
    colony_id: &str,
    moved_at: &str,
) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT box_id
         FROM colony_box_occupancies
         WHERE colony_id = ?
           AND started_at <= ?
           AND (ended_at IS NULL OR ended_at >= ?)
         ORDER BY started_at DESC
         LIMIT 1",
    )
    .bind(colony_id)
    .bind(moved_at)
    .bind(moved_at)
    .fetch_optional(pool)
    .await?)
}

pub async fn create(pool: &SqlitePool, input: CreateMovement) -> Result<ColonyMovement, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let movement_type = movement_type(&input.movement_type)?;
    let to_meliponary_id = optional(&input.to_meliponary_id);
    let to_box_id = optional(&input.to_box_id);
    let destination = optional(&input.destination);
    let document_reference = optional(&input.document_reference);
    let notes = optional(&input.notes);

    let moved_at = match optional(&input.moved_at) {
        Some(value) => value,
        None => {
            sqlx::query_scalar::<_, String>("SELECT CURRENT_TIMESTAMP")
                .fetch_one(pool)
                .await?
        }
    };

    if movement_type == "transport" {
        let colony: Option<(String, String)> =
            sqlx::query_as("SELECT meliponary_id, status FROM colonies WHERE id = ?")
                .bind(&colony_id)
                .fetch_optional(pool)
                .await?;
        let (from_meliponary_id, status) =
            colony.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;

        if !MOVABLE_STATUSES.contains(&status.as_str()) {
            return Err(AppError::Validation(
                "Esta colônia não está disponível para movimentação.".to_owned(),
            ));
        }
        if to_meliponary_id.is_some() || to_box_id.is_some() {
            return Err(AppError::Validation(
                "Transporte temporário não altera meliponário nem caixa de destino.".to_owned(),
            ));
        }
        let destination = destination
            .ok_or_else(|| AppError::Validation("Informe o destino do transporte.".to_owned()))?;
        let from_box_id = historical_box(pool, &colony_id, &moved_at).await?;
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO colony_movements (
                id, colony_id, movement_type, moved_at, from_meliponary_id,
                from_box_id, destination, document_reference, notes
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&colony_id)
        .bind(&movement_type)
        .bind(&moved_at)
        .bind(&from_meliponary_id)
        .bind(from_box_id)
        .bind(destination)
        .bind(document_reference)
        .bind(notes)
        .execute(pool)
        .await?;

        return get(pool, &id).await;
    }

    let mut tx = pool.begin().await?;

    let colony: Option<(String, String, String)> =
        sqlx::query_as("SELECT meliponary_id, status, code FROM colonies WHERE id = ?")
            .bind(&colony_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (from_meliponary_id, status, colony_code) =
        colony.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;

    if !MOVABLE_STATUSES.contains(&status.as_str()) {
        return Err(AppError::Validation(
            "Esta colônia não está disponível para transferência.".to_owned(),
        ));
    }

    let active_occupancy: Option<(String, String, String)> = sqlx::query_as(
        "SELECT id, box_id, started_at
         FROM colony_box_occupancies
         WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&colony_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some((_, _, started_at)) = &active_occupancy {
        if moved_at < *started_at {
            return Err(AppError::Validation(
                "A data da transferência não pode ser anterior ao início da ocupação atual."
                    .to_owned(),
            ));
        }
    }

    let from_box_id = active_occupancy
        .as_ref()
        .map(|(_, box_id, _)| box_id.clone());
    let id = Uuid::new_v4().to_string();

    match movement_type.as_str() {
        "internal_transfer" => {
            let target_meliponary_id = to_meliponary_id.ok_or_else(|| {
                AppError::Validation("Informe o meliponário de destino.".to_owned())
            })?;

            if destination.is_some() {
                return Err(AppError::Validation(
                    "Transferência interna usa um meliponário cadastrado como destino.".to_owned(),
                ));
            }
            if target_meliponary_id == from_meliponary_id {
                return Err(AppError::Validation(
                    "O meliponário de destino precisa ser diferente do atual.".to_owned(),
                ));
            }

            let target_exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meliponaries WHERE id = ?)")
                    .bind(&target_meliponary_id)
                    .fetch_one(&mut *tx)
                    .await?;
            if !target_exists {
                return Err(AppError::NotFound(
                    "Meliponário de destino não encontrado.".to_owned(),
                ));
            }

            let code_conflict: bool = sqlx::query_scalar(
                "SELECT EXISTS(
                    SELECT 1 FROM colonies
                    WHERE meliponary_id = ? AND code = ? AND id <> ?
                 )",
            )
            .bind(&target_meliponary_id)
            .bind(&colony_code)
            .bind(&colony_id)
            .fetch_one(&mut *tx)
            .await?;
            if code_conflict {
                return Err(AppError::Validation(
                    "Já existe uma colônia com este código no meliponário de destino.".to_owned(),
                ));
            }

            if let Some(target_box_id) = &to_box_id {
                let target_box: Option<(String, String)> =
                    sqlx::query_as("SELECT meliponary_id, status FROM boxes WHERE id = ?")
                        .bind(target_box_id)
                        .fetch_optional(&mut *tx)
                        .await?;
                let (box_meliponary_id, box_status) = target_box.ok_or_else(|| {
                    AppError::NotFound("Caixa de destino não encontrada.".to_owned())
                })?;

                if box_meliponary_id != target_meliponary_id {
                    return Err(AppError::Validation(
                        "A caixa de destino precisa pertencer ao meliponário de destino."
                            .to_owned(),
                    ));
                }
                if box_status != "active" {
                    return Err(AppError::Validation(
                        "A caixa de destino precisa estar ativa.".to_owned(),
                    ));
                }

                let occupied: bool = sqlx::query_scalar(
                    "SELECT EXISTS(
                        SELECT 1 FROM colony_box_occupancies
                        WHERE box_id = ? AND ended_at IS NULL
                     )",
                )
                .bind(target_box_id)
                .fetch_one(&mut *tx)
                .await?;
                if occupied {
                    return Err(AppError::Validation(
                        "A caixa de destino já está ocupada.".to_owned(),
                    ));
                }
            }

            if active_occupancy.is_some() {
                sqlx::query(
                    "UPDATE colony_box_occupancies
                     SET ended_at = ?
                     WHERE colony_id = ? AND ended_at IS NULL",
                )
                .bind(&moved_at)
                .bind(&colony_id)
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "UPDATE colonies
                 SET meliponary_id = ?, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(&target_meliponary_id)
            .bind(&colony_id)
            .execute(&mut *tx)
            .await?;

            if let Some(target_box_id) = &to_box_id {
                let occupancy_id = Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO colony_box_occupancies (
                        id, colony_id, box_id, started_at, reason, notes
                     ) VALUES (?, ?, ?, ?, 'Transferência entre meliponários', ?)",
                )
                .bind(occupancy_id)
                .bind(&colony_id)
                .bind(target_box_id)
                .bind(&moved_at)
                .bind(notes.clone())
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "INSERT INTO colony_movements (
                    id, colony_id, movement_type, moved_at,
                    from_meliponary_id, to_meliponary_id,
                    from_box_id, to_box_id, document_reference, notes
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&colony_id)
            .bind(&movement_type)
            .bind(&moved_at)
            .bind(&from_meliponary_id)
            .bind(&target_meliponary_id)
            .bind(from_box_id)
            .bind(to_box_id)
            .bind(document_reference)
            .bind(notes)
            .execute(&mut *tx)
            .await?;
        }
        "external_transfer" => {
            if to_meliponary_id.is_some() || to_box_id.is_some() {
                return Err(AppError::Validation(
                    "Transferência externa usa um destino textual, não um meliponário ou caixa cadastrados."
                        .to_owned(),
                ));
            }
            let destination = destination.ok_or_else(|| {
                AppError::Validation("Informe o destino da transferência.".to_owned())
            })?;

            if active_occupancy.is_some() {
                sqlx::query(
                    "UPDATE colony_box_occupancies
                     SET ended_at = ?
                     WHERE colony_id = ? AND ended_at IS NULL",
                )
                .bind(&moved_at)
                .bind(&colony_id)
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "UPDATE colonies
                 SET status = 'transferred', updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(&colony_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO colony_movements (
                    id, colony_id, movement_type, moved_at,
                    from_meliponary_id, from_box_id,
                    destination, document_reference, notes
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&colony_id)
            .bind(&movement_type)
            .bind(&moved_at)
            .bind(&from_meliponary_id)
            .bind(from_box_id)
            .bind(destination)
            .bind(document_reference)
            .bind(notes)
            .execute(&mut *tx)
            .await?;
        }
        _ => unreachable!(),
    }

    agenda::reconcile_inspection_tx(&mut tx, &colony_id).await?;
    agenda::reconcile_feeding_tx(&mut tx, &colony_id).await?;
    tx.commit().await?;
    get(pool, &id).await
}
