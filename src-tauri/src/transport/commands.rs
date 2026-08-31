use super::*;

#[tauri::command]
pub async fn complete_transport(
    pool: State<'_, SqlitePool>,
    mut input: CompleteTransport,
) -> Result<TransportReturn, String> {
    input.returned_at = Some(
        time::normalize_or_now(&pool, &input.returned_at, false)
            .await
            .map_err(message)?,
    );
    complete(&pool, input).await.map_err(message)
}

#[tauri::command]
pub async fn list_transport_returns(
    pool: State<'_, SqlitePool>,
    colony_id: String,
) -> Result<Vec<TransportReturn>, String> {
    list_by_colony(&pool, &colony_id).await.map_err(message)
}

#[tauri::command]
pub async fn reopen_transport(
    pool: State<'_, SqlitePool>,
    input: ReopenTransport,
) -> Result<(), String> {
    reopen(&pool, &input.movement_id, &input.reason)
        .await
        .map_err(message)
}
