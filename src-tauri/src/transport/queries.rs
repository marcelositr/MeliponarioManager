use super::*;

pub async fn list_by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<TransportReturn>, AppError> {
    let colony_id = required(colony_id, "Colônia")?;
    Ok(sqlx::query_as::<_, TransportReturn>(
        "SELECT r.id, r.movement_id, r.returned_at, r.notes,
                r.reversed_at, r.reversal_reason, r.created_at
         FROM transport_returns r
         JOIN colony_movements m ON m.id = r.movement_id
         WHERE m.colony_id = ? AND r.reversed_at IS NULL
         ORDER BY r.returned_at DESC, r.created_at DESC, r.id DESC",
    )
    .bind(colony_id)
    .fetch_all(pool)
    .await?)
}
