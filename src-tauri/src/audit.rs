use crate::repository::AppError;
use serde::Serialize;
use serde_json::Value;
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub changed_at: String,
    pub reason: String,
    pub before_json: Option<String>,
    pub after_json: Option<String>,
    pub actor: String,
    pub created_at: String,
}

fn required(value: &str, field: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} é obrigatório.")));
    }
    Ok(value.to_owned())
}

pub fn value<T: Serialize>(value: &T) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(|error| {
        AppError::Validation(format!(
            "Não foi possível preparar a trilha de auditoria: {error}"
        ))
    })
}

pub async fn record_tx(
    tx: &mut Transaction<'_, Sqlite>,
    entity_type: &str,
    entity_id: &str,
    action: &str,
    reason: &str,
    before: Option<Value>,
    after: Option<Value>,
) -> Result<String, AppError> {
    let entity_type = required(entity_type, "Tipo da entidade")?;
    let entity_id = required(entity_id, "Identificador da entidade")?;
    let action = required(action, "Ação de auditoria")?;
    let reason = required(reason, "Motivo")?;
    let changed_at: String =
        sqlx::query_scalar("SELECT strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')")
            .fetch_one(&mut **tx)
            .await?;
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO audit_records (
            id, entity_type, entity_id, action, changed_at, reason,
            before_json, after_json, actor
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'local_user')",
    )
    .bind(&id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(changed_at)
    .bind(reason)
    .bind(before.map(|value| value.to_string()))
    .bind(after.map(|value| value.to_string()))
    .execute(&mut **tx)
    .await?;

    Ok(id)
}

pub async fn list_by_entity(
    pool: &SqlitePool,
    entity_type: &str,
    entity_id: &str,
) -> Result<Vec<AuditRecord>, AppError> {
    let entity_type = required(entity_type, "Tipo da entidade")?;
    let entity_id = required(entity_id, "Identificador da entidade")?;

    Ok(sqlx::query_as::<_, AuditRecord>(
        "SELECT id, entity_type, entity_id, action, changed_at, reason,
                before_json, after_json, actor, created_at
         FROM audit_records
         WHERE entity_type = ? AND entity_id = ?
         ORDER BY changed_at DESC, created_at DESC, id DESC",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn audit_preserves_entity_action_reason_and_snapshots() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        record_tx(
            &mut tx,
            "meliponary",
            "m1",
            "edit",
            "Correção administrativa",
            Some(json!({ "name": "Antigo" })),
            Some(json!({ "name": "Novo" })),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let records = list_by_entity(&pool, "meliponary", "m1").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].action, "edit");
        assert_eq!(records[0].reason, "Correção administrativa");
        assert!(records[0].changed_at.len() == 19);
        assert!(records[0]
            .before_json
            .as_deref()
            .unwrap()
            .contains("Antigo"));
        assert!(records[0].after_json.as_deref().unwrap().contains("Novo"));
        assert_eq!(records[0].actor, "local_user");
    }
}
