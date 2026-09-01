use super::*;

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<ColonyMovement>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
        .bind(&colony_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Colônia não encontrada.".to_owned()));
    }

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
         WHERE m.colony_id = ?
         ORDER BY m.moved_at DESC, m.created_at DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM colony_movements")
        .fetch_one(pool)
        .await?)
}
