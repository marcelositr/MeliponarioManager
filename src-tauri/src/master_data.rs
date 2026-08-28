use crate::{
    audit,
    domain::{Colony, HiveBox, Meliponary, Species},
    repository::AppError,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Sqlite, SqlitePool, Transaction};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditMeliponary {
    pub id: String,
    pub name: String,
    pub responsible_name: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSpecies {
    pub id: String,
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub genus: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditBox {
    pub id: String,
    pub code: String,
    pub model: Option<String>,
    pub material: Option<String>,
    pub location_note: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditColony {
    pub id: String,
    pub code: String,
    pub origin_notes: Option<String>,
    pub notes: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityAction {
    pub id: String,
    pub reason: String,
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

async fn local_now(tx: &mut Transaction<'_, Sqlite>) -> Result<String, AppError> {
    Ok(sqlx::query_scalar("SELECT datetime('now', 'localtime')")
        .fetch_one(&mut **tx)
        .await?)
}

async fn get_meliponary(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<Meliponary, AppError> {
    sqlx::query_as::<_, Meliponary>(
        "SELECT id, name, responsible_name, location, notes, archived_at, archive_reason, created_at
         FROM meliponaries WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Meliponário não encontrado.".to_owned()))
}

async fn get_species(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<Species, AppError> {
    sqlx::query_as::<_, Species>(
        "SELECT id, common_name, scientific_name, genus, notes, archived_at, archive_reason, created_at
         FROM species WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Espécie não encontrada.".to_owned()))
}

async fn get_box(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<HiveBox, AppError> {
    sqlx::query_as::<_, HiveBox>(
        "SELECT b.id, b.meliponary_id, b.code, b.model, b.material, b.location_note,
                b.status, b.notes, c.code AS current_colony_code, b.created_at
         FROM boxes b
         LEFT JOIN colony_box_occupancies o ON o.box_id = b.id AND o.ended_at IS NULL
         LEFT JOIN colonies c ON c.id = o.colony_id
         WHERE b.id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))
}

async fn get_colony(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<Colony, AppError> {
    sqlx::query_as::<_, Colony>(
        "SELECT c.id, c.meliponary_id, c.species_id, c.code, c.origin_type,
                c.origin_notes, c.installed_at, c.status, c.mother_colony_id, c.notes,
                b.code AS current_box_code, c.created_at
         FROM colonies c
         LEFT JOIN colony_box_occupancies o ON o.colony_id = c.id AND o.ended_at IS NULL
         LEFT JOIN boxes b ON b.id = o.box_id
         WHERE c.id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))
}

pub async fn edit_meliponary(
    pool: &SqlitePool,
    input: EditMeliponary,
) -> Result<Meliponary, AppError> {
    let id = required(&input.id, "Meliponário")?;
    let name = required(&input.name, "Nome do meliponário")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM meliponaries
            WHERE id <> ? AND lower(trim(name)) = lower(trim(?))
         )",
    )
    .bind(&id)
    .bind(&name)
    .fetch_one(pool)
    .await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe outro meliponário com este nome.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE meliponaries
         SET name = ?, responsible_name = ?, location = ?, notes = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(name)
    .bind(optional(&input.responsible_name))
    .bind(optional(&input.location))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_meliponary(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
        &id,
        "edit",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn archive_meliponary(
    pool: &SqlitePool,
    input: EntityAction,
) -> Result<Meliponary, AppError> {
    let id = required(&input.id, "Meliponário")?;
    let reason = required(&input.reason, "Motivo do arquivamento")?;
    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    if before.archived_at.is_some() {
        return Err(AppError::Validation(
            "O meliponário já está arquivado.".to_owned(),
        ));
    }
    let archived_at = local_now(&mut tx).await?;
    sqlx::query(
        "UPDATE meliponaries
         SET archived_at = ?, archive_reason = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(archived_at)
    .bind(&reason)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_meliponary(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
        &id,
        "archive",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn reactivate_meliponary(
    pool: &SqlitePool,
    input: EntityAction,
) -> Result<Meliponary, AppError> {
    let id = required(&input.id, "Meliponário")?;
    let reason = required(&input.reason, "Motivo da reativação")?;
    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    if before.archived_at.is_none() {
        return Err(AppError::Validation(
            "O meliponário já está ativo.".to_owned(),
        ));
    }
    sqlx::query(
        "UPDATE meliponaries
         SET archived_at = NULL, archive_reason = NULL, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_meliponary(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
        &id,
        "reactivate",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn delete_meliponary(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Meliponário")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool = sqlx::query_scalar(
        "SELECT
            EXISTS(SELECT 1 FROM boxes WHERE meliponary_id = ?)
            OR EXISTS(SELECT 1 FROM colonies WHERE meliponary_id = ?)
            OR EXISTS(SELECT 1 FROM colony_movements
                      WHERE from_meliponary_id = ? OR to_meliponary_id = ?)",
    )
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if used {
        return Err(AppError::Validation(
            "Este meliponário já foi utilizado. Arquive-o em vez de excluí-lo.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_meliponary(&mut tx, &id).await?;
    sqlx::query("DELETE FROM meliponaries WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "meliponary",
        &id,
        "delete",
        &reason,
        Some(audit::value(&before)?),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn edit_species(pool: &SqlitePool, input: EditSpecies) -> Result<Species, AppError> {
    let id = required(&input.id, "Espécie")?;
    let common_name = required(&input.common_name, "Nome popular")?;
    let scientific_name = optional(&input.scientific_name);
    let reason = required(&input.reason, "Motivo da edição")?;
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM species
            WHERE id <> ?
              AND lower(trim(common_name)) = lower(trim(?))
              AND lower(trim(COALESCE(scientific_name, ''))) = lower(trim(COALESCE(?, '')))
         )",
    )
    .bind(&id)
    .bind(&common_name)
    .bind(&scientific_name)
    .fetch_one(pool)
    .await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe uma espécie com estes nomes cadastrais.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE species
         SET common_name = ?, scientific_name = ?, genus = ?, notes = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(common_name)
    .bind(scientific_name)
    .bind(optional(&input.genus))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_species(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "species",
        &id,
        "edit",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn archive_species(pool: &SqlitePool, input: EntityAction) -> Result<Species, AppError> {
    let id = required(&input.id, "Espécie")?;
    let reason = required(&input.reason, "Motivo do arquivamento")?;
    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    if before.archived_at.is_some() {
        return Err(AppError::Validation(
            "A espécie já está arquivada.".to_owned(),
        ));
    }
    let archived_at = local_now(&mut tx).await?;
    sqlx::query(
        "UPDATE species
         SET archived_at = ?, archive_reason = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(archived_at)
    .bind(&reason)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_species(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "species",
        &id,
        "archive",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn reactivate_species(
    pool: &SqlitePool,
    input: EntityAction,
) -> Result<Species, AppError> {
    let id = required(&input.id, "Espécie")?;
    let reason = required(&input.reason, "Motivo da reativação")?;
    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    if before.archived_at.is_none() {
        return Err(AppError::Validation("A espécie já está ativa.".to_owned()));
    }
    sqlx::query(
        "UPDATE species
         SET archived_at = NULL, archive_reason = NULL, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_species(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "species",
        &id,
        "reactivate",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn delete_species(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Espécie")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colonies WHERE species_id = ?)")
            .bind(&id)
            .fetch_one(pool)
            .await?;
    if used {
        return Err(AppError::Validation(
            "Esta espécie já foi utilizada. Arquive-a em vez de excluí-la.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_species(&mut tx, &id).await?;
    sqlx::query("DELETE FROM species WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "species",
        &id,
        "delete",
        &reason,
        Some(audit::value(&before)?),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn edit_box(pool: &SqlitePool, input: EditBox) -> Result<HiveBox, AppError> {
    let id = required(&input.id, "Caixa")?;
    let code = required(&input.code, "Identificação da caixa")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    let meliponary_id: Option<String> =
        sqlx::query_scalar("SELECT meliponary_id FROM boxes WHERE id = ?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let meliponary_id =
        meliponary_id.ok_or_else(|| AppError::NotFound("Caixa não encontrada.".to_owned()))?;
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM boxes
            WHERE id <> ? AND meliponary_id = ?
              AND lower(trim(code)) = lower(trim(?))
         )",
    )
    .bind(&id)
    .bind(&meliponary_id)
    .bind(&code)
    .fetch_one(pool)
    .await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe uma caixa com esta identificação no meliponário.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let before = get_box(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE boxes
         SET code = ?, model = ?, material = ?, location_note = ?, notes = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(code)
    .bind(optional(&input.model))
    .bind(optional(&input.material))
    .bind(optional(&input.location_note))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_box(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "box",
        &id,
        "edit",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn delete_box(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Caixa")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool = sqlx::query_scalar(
        "SELECT
            EXISTS(SELECT 1 FROM colony_box_occupancies WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM box_maintenance_records WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM colony_events WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM inspections WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM feedings WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM production_records WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE box_id = ?)
            OR EXISTS(SELECT 1 FROM colony_movements WHERE from_box_id = ? OR to_box_id = ?)
            OR EXISTS(SELECT 1 FROM box_state_records WHERE box_id = ?)",
    )
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if used {
        return Err(AppError::Validation(
            "Esta caixa possui histórico. Aposente-a em vez de excluí-la.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_box(&mut tx, &id).await?;
    sqlx::query("DELETE FROM boxes WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "box",
        &id,
        "delete",
        &reason,
        Some(audit::value(&before)?),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn edit_colony(pool: &SqlitePool, input: EditColony) -> Result<Colony, AppError> {
    let id = required(&input.id, "Colônia")?;
    let code = required(&input.code, "Identificação da colônia")?;
    let reason = required(&input.reason, "Motivo da edição")?;
    let meliponary_id: Option<String> =
        sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id = ?")
            .bind(&id)
            .fetch_optional(pool)
            .await?;
    let meliponary_id =
        meliponary_id.ok_or_else(|| AppError::NotFound("Colônia não encontrada.".to_owned()))?;
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM colonies
            WHERE id <> ? AND meliponary_id = ?
              AND lower(trim(code)) = lower(trim(?))
         )",
    )
    .bind(&id)
    .bind(&meliponary_id)
    .bind(&code)
    .fetch_one(pool)
    .await?;
    if duplicate {
        return Err(AppError::Validation(
            "Já existe uma colônia com esta identificação no meliponário.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let before = get_colony(&mut tx, &id).await?;
    sqlx::query(
        "UPDATE colonies
         SET code = ?, origin_notes = ?, notes = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(code)
    .bind(optional(&input.origin_notes))
    .bind(optional(&input.notes))
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    let after = get_colony(&mut tx, &id).await?;
    audit::record_tx(
        &mut tx,
        "colony",
        &id,
        "edit",
        &reason,
        Some(audit::value(&before)?),
        Some(audit::value(&after)?),
    )
    .await?;
    tx.commit().await?;
    Ok(after)
}

pub async fn delete_colony(pool: &SqlitePool, input: EntityAction) -> Result<(), AppError> {
    let id = required(&input.id, "Colônia")?;
    let reason = required(&input.reason, "Motivo da exclusão")?;
    let used: bool = sqlx::query_scalar(
        "SELECT
            EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM inspections WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM feedings WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM production_records WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_events WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_divisions
                      WHERE parent_colony_id = ? OR daughter_colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_movements WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM box_maintenance_records WHERE colony_id = ?)
            OR EXISTS(SELECT 1 FROM colonies WHERE mother_colony_id = ?)
            OR EXISTS(SELECT 1 FROM colonies WHERE id = ? AND mother_colony_id IS NOT NULL)",
    )
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .bind(&id)
    .fetch_one(pool)
    .await?;
    if used {
        return Err(AppError::Validation(
            "Esta colônia possui histórico ou vínculo genealógico e não pode ser excluída. Use o ciclo de vida adequado.".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let before = get_colony(&mut tx, &id).await?;
    sqlx::query("DELETE FROM colonies WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "colony",
        &id,
        "delete",
        &reason,
        Some(audit::value(&before)?),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies},
        repository,
    };
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

    #[tokio::test]
    async fn meliponary_edit_archive_reactivate_and_empty_delete_are_safe() {
        let pool = pool().await;
        let item = repository::create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let edited = edit_meliponary(
            &pool,
            EditMeliponary {
                id: item.id.clone(),
                name: "Principal Norte".into(),
                responsible_name: Some("Marcelo".into()),
                location: None,
                notes: None,
                reason: "Correção cadastral".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(edited.name, "Principal Norte");
        let archived = archive_meliponary(
            &pool,
            EntityAction {
                id: item.id.clone(),
                reason: "Sem uso".into(),
            },
        )
        .await
        .unwrap();
        assert!(archived.archived_at.is_some());
        let active = reactivate_meliponary(
            &pool,
            EntityAction {
                id: item.id.clone(),
                reason: "Retorno".into(),
            },
        )
        .await
        .unwrap();
        assert!(active.archived_at.is_none());
        delete_meliponary(
            &pool,
            EntityAction {
                id: item.id.clone(),
                reason: "Cadastro de teste".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meliponaries WHERE id=?")
                .bind(item.id)
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn used_meliponary_cannot_be_deleted() {
        let pool = pool().await;
        let mel = repository::create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: mel.id.clone(),
                code: "CX-001".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        assert!(delete_meliponary(
            &pool,
            EntityAction {
                id: mel.id,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn species_used_by_colony_cannot_be_deleted_but_empty_species_can() {
        let pool = pool().await;
        let mel = repository::create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let used = repository::create_species(
            &pool,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let empty = repository::create_species(
            &pool,
            CreateSpecies {
                common_name: "Mandaçaia".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::create_colony(
            &pool,
            CreateColony {
                meliponary_id: mel.id,
                species_id: used.id.clone(),
                code: "JAT-001".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: None,
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        assert!(delete_species(
            &pool,
            EntityAction {
                id: used.id,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
        delete_species(
            &pool,
            EntityAction {
                id: empty.id.clone(),
                reason: "Cadastro duplicado".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM species WHERE id=?")
                .bind(empty.id)
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn unused_box_and_colony_can_be_deleted_but_history_blocks_delete() {
        let pool = pool().await;
        let mel = repository::create_meliponary(
            &pool,
            CreateMeliponary {
                name: "Principal".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let species = repository::create_species(
            &pool,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let empty_box = repository::create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: mel.id.clone(),
                code: "CX-001".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let empty_colony = repository::create_colony(
            &pool,
            CreateColony {
                meliponary_id: mel.id.clone(),
                species_id: species.id.clone(),
                code: "JAT-001".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: None,
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        delete_box(
            &pool,
            EntityAction {
                id: empty_box.id,
                reason: "Nunca usada".into(),
            },
        )
        .await
        .unwrap();
        delete_colony(
            &pool,
            EntityAction {
                id: empty_colony.id,
                reason: "Nunca usada".into(),
            },
        )
        .await
        .unwrap();

        let used_box = repository::create_box(
            &pool,
            CreateHiveBox {
                meliponary_id: mel.id.clone(),
                code: "CX-002".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let used_colony = repository::create_colony(
            &pool,
            CreateColony {
                meliponary_id: mel.id,
                species_id: species.id,
                code: "JAT-002".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: None,
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at) VALUES('o1',?,?,datetime('now','localtime'))")
            .bind(&used_colony.id).bind(&used_box.id).execute(&pool).await.unwrap();
        assert!(delete_box(
            &pool,
            EntityAction {
                id: used_box.id,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
        assert!(delete_colony(
            &pool,
            EntityAction {
                id: used_colony.id,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
    }
}
