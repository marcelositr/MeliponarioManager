use crate::repository::AppError;use serde::Serialize;use sqlx::{FromRow,SqlitePool};
#[derive(Debug,Clone,Serialize,FromRow)]#[serde(rename_all="camelCase")]
pub struct RecordState{pub id:String,pub corrected_at:Option<String>,pub voided_at:Option<String>,pub void_reason:Option<String>,pub reversed_at:Option<String>,pub reversal_reason:Option<String>}
fn req(v:&str,f:&str)->Result<String,AppError>{let v=v.trim();if v.is_empty(){Err(AppError::Validation(format!("{f} é obrigatório.")))}else{Ok(v.to_owned())}}
pub async fn list_states(p:&SqlitePool,entity_type:&str,scope_id:&str)->Result<Vec<RecordState>,AppError>{let scope=req(scope_id,"Escopo")?;let sql=match entity_type{
 "inspection"=>"SELECT id,corrected_at,voided_at,void_reason,NULL reversed_at,NULL reversal_reason FROM inspections WHERE colony_id=?",
 "feeding"=>"SELECT id,corrected_at,voided_at,void_reason,NULL reversed_at,NULL reversal_reason FROM feedings WHERE colony_id=?",
 "production"=>"SELECT id,corrected_at,voided_at,void_reason,NULL reversed_at,NULL reversal_reason FROM production_records WHERE colony_id=?",
 "box_maintenance"=>"SELECT id,corrected_at,voided_at,void_reason,NULL reversed_at,NULL reversal_reason FROM box_maintenance_records WHERE box_id=?",
 "colony_event"=>"SELECT id,corrected_at,voided_at,void_reason,NULL reversed_at,NULL reversal_reason FROM colony_events WHERE colony_id=?",
 "movement"=>"SELECT id,corrected_at,voided_at,void_reason,reversed_at,reversal_reason FROM colony_movements WHERE colony_id=?",
 "movement_document"=>"SELECT id,corrected_at,voided_at,void_reason,NULL reversed_at,NULL reversal_reason FROM movement_documents WHERE movement_id=?",
 "lifecycle"=>"SELECT id,NULL corrected_at,NULL voided_at,NULL void_reason,reversed_at,reversal_reason FROM colony_lifecycle_records WHERE colony_id=?",
 "box_occupancy"=>"SELECT id,corrected_at,NULL voided_at,NULL void_reason,NULL reversed_at,NULL reversal_reason FROM colony_box_occupancies WHERE colony_id=?",
 _=>return Err(AppError::Validation("Tipo de registro sem estado administrativo.".to_owned())),};Ok(sqlx::query_as::<_,RecordState>(sql).bind(scope).fetch_all(p).await?)}
pub async fn list_division_states(p:&SqlitePool,colony_id:&str)->Result<Vec<RecordState>,AppError>{let c=req(colony_id,"Colônia")?;Ok(sqlx::query_as::<_,RecordState>("SELECT id,corrected_at,voided_at,void_reason,NULL reversed_at,NULL reversal_reason FROM colony_divisions WHERE parent_colony_id=? OR daughter_colony_id=?").bind(&c).bind(&c).fetch_all(p).await?)}
pub async fn valid_count(p:&SqlitePool,entity_type:&str)->Result<i64,AppError>{let sql=match entity_type{"inspection"=>"SELECT COUNT(*) FROM inspections WHERE voided_at IS NULL","feeding"=>"SELECT COUNT(*) FROM feedings WHERE voided_at IS NULL","production"=>"SELECT COUNT(*) FROM production_records WHERE voided_at IS NULL","box_maintenance"=>"SELECT COUNT(*) FROM box_maintenance_records WHERE voided_at IS NULL","colony_event"=>"SELECT COUNT(*) FROM colony_events WHERE voided_at IS NULL","division"=>"SELECT COUNT(*) FROM colony_divisions WHERE voided_at IS NULL","movement"=>"SELECT COUNT(*) FROM colony_movements WHERE voided_at IS NULL","movement_document"=>"SELECT COUNT(*) FROM movement_documents WHERE voided_at IS NULL",_=>return Err(AppError::Validation("Tipo sem contador válido.".to_owned())),};Ok(sqlx::query_scalar(sql).fetch_one(p).await?)}
