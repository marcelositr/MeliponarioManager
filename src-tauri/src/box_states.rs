use crate::{repository::AppError, time};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BoxStateRecord {
    pub id: String,
    pub box_id: String,
    pub box_code: String,
    pub occurred_at: String,
    pub previous_status: String,
    pub new_status: String,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeBoxState {
    pub box_id: String,
    pub new_status: String,
    pub occurred_at: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

fn required(value: &str, field: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} é obrigatório.")));
    }
    Ok(value.to_owned())
}

fn optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn transition_allowed(previous: &str, next: &str) -> bool {
    matches!(
        (previous, next),
        ("active", "maintenance")
            | ("maintenance", "active")
            | ("active", "retired")
            | ("maintenance", "retired")
    )
}

async fn get(pool: &SqlitePool, id: &str) -> Result<BoxStateRecord, AppError> {
    Ok(sqlx::query_as::<_, BoxStateRecord>(
        "SELECT r.id, r.box_id, b.code AS box_code, r.occurred_at,
                r.previous_status, r.new_status, r.reason, r.notes, r.created_at
         FROM box_state_records r
         JOIN boxes b ON b.id = r.box_id
         WHERE r.id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

pub async fn change(
    pool: &SqlitePool,
    input: ChangeBoxState,
) -> Result<BoxStateRecord, AppError> {
    let box_id = required(&input.box_id, "Caixa")?;
    let new_status = required(&input.new_status, "Novo estado da caixa")?;
    if !matches!(new_status.as_str(), "active" | "maintenance" | "retired") {
        return Err(AppError::Validation(
            "Estado de caixa inválido. Use active, maintenance ou retired.".to_owned(),
        ));
    }
    let occurred_at = time::normalize_or_now(pool, &input.occurred_at, false).await?;
    let reason = optional(&input.reason);
    let notes = optional(&input.notes);

    let mut tx = pool.begin().await?;
    let current_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM boxes WHERE id = ?")
            .bind(&box_id)
            .fetch_optional(&mut *tx)
            .await?;
    let current_status =
        current_status.ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))?;

    if current_status == "retired" {
        return Err(AppError::Validation(
            "Uma caixa aposentada não volta ao fluxo operacional nesta etapa.".to_owned(),
        ));
    }
    if !transition_allowed(&current_status, &new_status) {
        return Err(AppError::Validation(
            "Transição de estado da caixa não permitida.".to_owned(),
        ));
    }

    let last_at: Option<String> = sqlx::query_scalar(
        "SELECT MAX(occurred_at) FROM box_state_records WHERE box_id = ?",
    )
    .bind(&box_id)
    .fetch_one(&mut *tx)
    .await?;
    if last_at
        .as_deref()
        .is_some_and(|last| occurred_at.as_str() < last)
    {
        return Err(AppError::Validation(
            "A mudança de estado não pode ser anterior à última mudança registrada.".to_owned(),
        ));
    }

    if new_status != "active" {
        let occupied: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM colony_box_occupancies
                WHERE box_id = ? AND ended_at IS NULL
             )",
        )
        .bind(&box_id)
        .fetch_one(&mut *tx)
        .await?;
        if occupied {
            return Err(AppError::Validation(
                "A caixa precisa estar vazia antes de entrar em manutenção ou ser aposentada."
                    .to_owned(),
            ));
        }
    }

    sqlx::query("UPDATE boxes SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&new_status)
        .bind(&box_id)
        .execute(&mut *tx)
        .await?;

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO box_state_records (
            id, box_id, occurred_at, previous_status, new_status, reason, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&box_id)
    .bind(&occurred_at)
    .bind(&current_status)
    .bind(&new_status)
    .bind(reason)
    .bind(notes)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    get(pool, &id).await
}

pub async fn list_by_box(
    pool: &SqlitePool,
    box_id: &str,
) -> Result<Vec<BoxStateRecord>, AppError> {
    let box_id = required(box_id, "Caixa")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM boxes WHERE id = ?)")
        .bind(&box_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Caixa não encontrada.".to_owned()));
    }

    Ok(sqlx::query_as::<_, BoxStateRecord>(
        "SELECT r.id, r.box_id, b.code AS box_code, r.occurred_at,
                r.previous_status, r.new_status, r.reason, r.notes, r.created_at
         FROM box_state_records r
         JOIN boxes b ON b.id = r.box_id
         WHERE r.box_id = ?
         ORDER BY r.occurred_at DESC, r.created_at DESC, r.id DESC",
    )
    .bind(box_id)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
        sqlx::query("INSERT INTO meliponaries (id, name) VALUES ('m1', 'Principal')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO species (id, common_name) VALUES ('s1', 'Jataí')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO boxes (id, meliponary_id, code) VALUES ('b1', 'm1', 'CX-001')")
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    async fn occupy(pool: &SqlitePool) {
        sqlx::query(
            "INSERT INTO colonies (id, meliponary_id, species_id, code) VALUES ('c1','m1','s1','JAT-001')",
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO colony_box_occupancies (id, colony_id, box_id, started_at)
             VALUES ('o1','c1','b1','2026-01-01 09:00:00')",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn valid_transitions_create_history_and_retired_is_terminal() {
        let pool = pool().await;
        let first = change(
            &pool,
            ChangeBoxState {
                box_id: "b1".into(),
                new_status: "maintenance".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: Some("Reparo".into()),
                notes: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(first.previous_status, "active");
        assert_eq!(first.new_status, "maintenance");

        change(
            &pool,
            ChangeBoxState {
                box_id: "b1".into(),
                new_status: "retired".into(),
                occurred_at: Some("2026-03-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(list_by_box(&pool, "b1").await.unwrap().len(), 2);
        assert!(change(
            &pool,
            ChangeBoxState {
                box_id: "b1".into(),
                new_status: "active".into(),
                occurred_at: Some("2026-04-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn occupied_box_cannot_leave_active_state() {
        let pool = pool().await;
        occupy(&pool).await;

        assert!(change(
            &pool,
            ChangeBoxState {
                box_id: "b1".into(),
                new_status: "maintenance".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn sqlite_rejects_nonactive_state_for_occupied_box() {
        let pool = pool().await;
        occupy(&pool).await;

        let result = sqlx::query("UPDATE boxes SET status = 'maintenance' WHERE id = 'b1'")
            .execute(&pool)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sqlite_rejects_occupancy_in_nonactive_box() {
        let pool = pool().await;
        sqlx::query("UPDATE boxes SET status = 'maintenance' WHERE id = 'b1'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO colonies (id, meliponary_id, species_id, code) VALUES ('c1','m1','s1','JAT-001')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let result = sqlx::query(
            "INSERT INTO colony_box_occupancies (id, colony_id, box_id, started_at)
             VALUES ('o1','c1','b1','2026-01-01 09:00:00')",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err());
    }
}
