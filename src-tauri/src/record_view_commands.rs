use crate::record_views;use sqlx::SqlitePool;use tauri::State;
#[tauri::command]pub async fn list_record_states(pool:State<'_,SqlitePool>,entity_type:String,scope_id:String)->Result<Vec<record_views::RecordState>,String>{record_views::list_states(&pool,&entity_type,&scope_id).await.map_err(|e|e.to_string())}
#[tauri::command]pub async fn list_division_states(pool:State<'_,SqlitePool>,colony_id:String)->Result<Vec<record_views::RecordState>,String>{record_views::list_division_states(&pool,&colony_id).await.map_err(|e|e.to_string())}
#[tauri::command]pub async fn get_valid_record_count(pool:State<'_,SqlitePool>,entity_type:String)->Result<i64,String>{record_views::valid_count(&pool,&entity_type).await.map_err(|e|e.to_string())}
