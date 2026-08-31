use super::*;

pub async fn place_colony(
    pool: &SqlitePool,
    input: PlaceColony,
) -> Result<ColonyBoxOccupancy, AppError> {
    let colony_id = required(&input.colony_id, "Colônia")?;
    let box_id = required(&input.box_id, "Caixa")?;
    let started_at = match optional(&input.started_at) {
        Some(value) => time::normalize(&value, false)?,
        None => time::local_now(pool).await?,
    };
    let mut tx = pool.begin().await?;

    let colony_context: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT c.meliponary_id, m.archived_at
         FROM colonies c
         JOIN meliponaries m ON m.id = c.meliponary_id
         WHERE c.id = ?",
    )
    .bind(&colony_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (colony_meliponary, colony_meliponary_archived) =
        colony_context.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;
    if colony_meliponary_archived.is_some() {
        return Err(AppError::Validation(
            "O meliponário da colônia está arquivado e não aceita nova ocupação.".to_owned(),
        ));
    }

    let target_box: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT b.meliponary_id, b.status, m.archived_at
         FROM boxes b
         JOIN meliponaries m ON m.id = b.meliponary_id
         WHERE b.id = ?",
    )
    .bind(&box_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (box_meliponary, box_status, box_meliponary_archived) =
        target_box.ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))?;

    if box_meliponary_archived.is_some() {
        return Err(AppError::Validation(
            "O meliponário da caixa está arquivado.".to_owned(),
        ));
    }
    if box_status != "active" {
        return Err(AppError::Validation(
            "Somente uma caixa ativa pode receber uma nova ocupação.".to_owned(),
        ));
    }
    if colony_meliponary != box_meliponary {
        return Err(AppError::Validation(
            "A colônia e a caixa precisam pertencer ao mesmo meliponário.".to_owned(),
        ));
    }

    let current_occupancy: Option<(String, String, String)> = sqlx::query_as(
        "SELECT id, box_id, started_at FROM colony_box_occupancies
         WHERE colony_id = ? AND ended_at IS NULL",
    )
    .bind(&colony_id)
    .fetch_optional(&mut *tx)
    .await?;

    if current_occupancy
        .as_ref()
        .is_some_and(|(_, current_box, _)| current_box == &box_id)
    {
        return Err(AppError::Validation(
            "A colônia já está registrada nesta caixa.".to_owned(),
        ));
    }

    let target_occupant: Option<String> = sqlx::query_scalar(
        "SELECT colony_id FROM colony_box_occupancies
         WHERE box_id = ? AND ended_at IS NULL",
    )
    .bind(&box_id)
    .fetch_optional(&mut *tx)
    .await?;
    if target_occupant.is_some() {
        return Err(AppError::Validation(
            "A caixa já está ocupada por outra colônia.".to_owned(),
        ));
    }

    let before = current_occupancy
        .as_ref()
        .map(|(id, current_box, current_started_at)| {
            json!({
                "occupancy_id": id,
                "colony_id": colony_id,
                "box_id": current_box,
                "started_at": current_started_at,
                "ended_at": null
            })
        });

    if let Some((occupancy_id, _, current_started_at)) = &current_occupancy {
        if started_at < *current_started_at {
            return Err(AppError::Validation(
                "A data da troca não pode ser anterior ao início da ocupação atual.".to_owned(),
            ));
        }
        sqlx::query(
            "UPDATE colony_box_occupancies
             SET ended_at = ?
             WHERE id = ? AND ended_at IS NULL",
        )
        .bind(&started_at)
        .bind(occupancy_id)
        .execute(&mut *tx)
        .await?;
    }

    let occupancy_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO colony_box_occupancies
            (id, colony_id, box_id, started_at, reason, notes)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&occupancy_id)
    .bind(&colony_id)
    .bind(&box_id)
    .bind(&started_at)
    .bind(optional(&input.reason))
    .bind(optional(&input.notes))
    .execute(&mut *tx)
    .await?;

    let audit_reason =
        optional(&input.reason).unwrap_or_else(|| "Alteração de ocupação de caixa".to_owned());
    audit::record_tx(
        &mut tx,
        "box_occupancy",
        &occupancy_id,
        "place",
        &audit_reason,
        before,
        Some(json!({
            "occupancy_id": occupancy_id,
            "colony_id": colony_id,
            "box_id": box_id,
            "started_at": started_at,
            "ended_at": null
        })),
    )
    .await?;

    tx.commit().await?;

    Ok(sqlx::query_as::<_, ColonyBoxOccupancy>(
        "SELECT id, colony_id, box_id, started_at, ended_at, reason, notes, corrected_at
         FROM colony_box_occupancies WHERE id = ?",
    )
    .bind(occupancy_id)
    .fetch_one(pool)
    .await?)
}
