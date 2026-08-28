use crate::repository::AppError;
use sqlx::SqlitePool;

pub fn is_manageable_status(status: &str) -> bool {
    matches!(status, "active" | "weak" | "recovering")
}

pub async fn ensure_colony_exists(pool: &SqlitePool, colony_id: &str) -> Result<(), AppError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE id = ?)")
        .bind(colony_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(AppError::NotFound("Colônia não encontrada.".to_owned()))
    }
}

pub async fn ensure_colony_available_at(
    pool: &SqlitePool,
    colony_id: &str,
    occurred_at: &str,
) -> Result<(), AppError> {
    let colony: Option<(String, String)> = sqlx::query_as(
        "SELECT COALESCE(installed_at, created_at), status FROM colonies WHERE id = ?",
    )
    .bind(colony_id)
    .fetch_optional(pool)
    .await?;
    let (entry_at, current_status) =
        colony.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;
    if occurred_at < entry_at.as_str() {
        return Err(AppError::Validation(
            "A data do manejo não pode ser anterior à entrada da colônia no plantel.".to_owned(),
        ));
    }
    let external_transfer: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM colony_movements
            WHERE colony_id = ? AND movement_type = 'external_transfer'
              AND moved_at <= ? AND voided_at IS NULL AND reversed_at IS NULL
         )",
    )
    .bind(colony_id)
    .bind(occurred_at)
    .fetch_one(pool)
    .await?;
    if external_transfer {
        return Err(AppError::Validation(
            "A colônia já havia sido transferida para fora do plantel nesta data.".to_owned(),
        ));
    }
    let status_at: Option<String> = sqlx::query_scalar(
        "SELECT new_status FROM colony_lifecycle_records
         WHERE colony_id = ? AND occurred_at <= ? AND reversed_at IS NULL
         ORDER BY occurred_at DESC, created_at DESC, id DESC LIMIT 1",
    )
    .bind(colony_id)
    .bind(occurred_at)
    .fetch_optional(pool)
    .await?;
    let effective_status = match status_at {
        Some(status) => status,
        None => {
            let later: bool = sqlx::query_scalar(
                "SELECT EXISTS(
                    SELECT 1 FROM colony_lifecycle_records
                    WHERE colony_id = ? AND reversed_at IS NULL
                 ) OR EXISTS(
                    SELECT 1 FROM colony_movements
                    WHERE colony_id = ? AND movement_type = 'external_transfer'
                      AND voided_at IS NULL AND reversed_at IS NULL
                 )",
            )
            .bind(colony_id)
            .bind(colony_id)
            .fetch_one(pool)
            .await?;
            if later {
                "active".to_owned()
            } else {
                current_status
            }
        }
    };
    if is_manageable_status(&effective_status) {
        Ok(())
    } else {
        Err(AppError::Validation(
            "A colônia não estava operacionalmente disponível nesta data.".to_owned(),
        ))
    }
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
        pool
    }
    async fn seed_colony(pool: &SqlitePool, status: &str) -> String {
        sqlx::query("INSERT INTO meliponaries (id, name) VALUES ('m1', 'Principal')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO species (id, common_name) VALUES ('s1', 'Jataí')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO colonies (id, meliponary_id, species_id, code, installed_at, status) VALUES ('c1','m1','s1','JAT-001','2026-01-01 09:00:00',?)").bind(status).execute(pool).await.unwrap();
        "c1".to_owned()
    }
    #[tokio::test]
    async fn active_and_legacy_manageable_statuses_are_available() {
        for status in ["active", "weak", "recovering"] {
            let pool = pool().await;
            let id = seed_colony(&pool, status).await;
            ensure_colony_available_at(&pool, &id, "2026-02-01 10:00:00")
                .await
                .unwrap();
        }
    }
    #[tokio::test]
    async fn retrospective_fact_before_loss_is_allowed_but_later_fact_is_rejected() {
        let pool = pool().await;
        let id = seed_colony(&pool, "lost").await;
        sqlx::query("INSERT INTO colony_lifecycle_records(id,colony_id,action,occurred_at,previous_status,new_status)VALUES('life1',?,'loss','2026-03-10 10:00:00','active','lost')").bind(&id).execute(&pool).await.unwrap();
        ensure_colony_available_at(&pool, &id, "2026-02-20 10:00:00")
            .await
            .unwrap();
        assert!(
            ensure_colony_available_at(&pool, &id, "2026-03-11 10:00:00")
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn inactivity_interval_is_rejected_and_reactivation_restores_availability() {
        let pool = pool().await;
        let id = seed_colony(&pool, "active").await;
        sqlx::query("INSERT INTO colony_lifecycle_records(id,colony_id,action,occurred_at,previous_status,new_status)VALUES('life1',?,'deactivate','2026-03-01 10:00:00','active','inactive'),('life2',?,'reactivate','2026-04-01 10:00:00','inactive','active')").bind(&id).bind(&id).execute(&pool).await.unwrap();
        assert!(
            ensure_colony_available_at(&pool, &id, "2026-03-15 10:00:00")
                .await
                .is_err()
        );
        ensure_colony_available_at(&pool, &id, "2026-04-02 10:00:00")
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn external_transfer_blocks_later_management_but_preserves_retrospective_facts() {
        let pool = pool().await;
        let id = seed_colony(&pool, "transferred").await;
        sqlx::query("INSERT INTO colony_movements(id,colony_id,movement_type,moved_at,from_meliponary_id,destination)VALUES('move1',?,'external_transfer','2026-05-01 10:00:00','m1','Outro criador')").bind(&id).execute(&pool).await.unwrap();
        ensure_colony_available_at(&pool, &id, "2026-04-30 10:00:00")
            .await
            .unwrap();
        assert!(
            ensure_colony_available_at(&pool, &id, "2026-05-02 10:00:00")
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn reversed_external_transfer_no_longer_blocks_management() {
        let pool = pool().await;
        let id = seed_colony(&pool, "active").await;
        sqlx::query("INSERT INTO colony_movements(id,colony_id,movement_type,moved_at,from_meliponary_id,destination,reversed_at,reversal_reason)VALUES('move1',?,'external_transfer','2026-05-01 10:00:00','m1','Outro','2026-05-02 10:00:00','erro')").bind(&id).execute(&pool).await.unwrap();
        ensure_colony_available_at(&pool, &id, "2026-05-03 10:00:00")
            .await
            .unwrap();
    }
}
