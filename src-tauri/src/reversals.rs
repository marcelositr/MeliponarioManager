use crate::{audit, operational, repository::AppError};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseRecord {
    pub id: String,
    pub reason: String,
}
fn req(v: &str, f: &str) -> Result<String, AppError> {
    let v = v.trim();
    if v.is_empty() {
        Err(AppError::Validation(format!("{f} é obrigatório.")))
    } else {
        Ok(v.to_owned())
    }
}
async fn now(tx: &mut Transaction<'_, Sqlite>) -> Result<String, AppError> {
    Ok(sqlx::query_scalar("SELECT datetime('now','localtime')")
        .fetch_one(&mut **tx)
        .await?)
}
async fn later_facts(
    p: &SqlitePool,
    c: &str,
    after: &str,
    except_lifecycle: Option<&str>,
    except_movement: Option<&str>,
) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM inspections WHERE colony_id=? AND inspected_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM feedings WHERE colony_id=? AND fed_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM production_records WHERE colony_id=? AND harvested_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM colony_events WHERE colony_id=? AND occurred_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM colony_divisions WHERE (parent_colony_id=? OR daughter_colony_id=?) AND performed_at>? AND voided_at IS NULL) OR EXISTS(SELECT 1 FROM colony_movements WHERE colony_id=? AND moved_at>? AND voided_at IS NULL AND reversed_at IS NULL AND (? IS NULL OR id<>?)) OR EXISTS(SELECT 1 FROM colony_lifecycle_records WHERE colony_id=? AND occurred_at>? AND reversed_at IS NULL AND (? IS NULL OR id<>?))")
.bind(c).bind(after).bind(c).bind(after).bind(c).bind(after).bind(c).bind(after).bind(c).bind(c).bind(after).bind(c).bind(after).bind(except_movement).bind(except_movement).bind(c).bind(after).bind(except_lifecycle).bind(except_lifecycle).fetch_one(p).await?)
}
async fn box_free(tx: &mut Transaction<'_, Sqlite>, b: &str) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM boxes WHERE id=? AND status='active') AND NOT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE box_id=? AND ended_at IS NULL)").bind(b).bind(b).fetch_one(&mut**tx).await?)
}
async fn restore_box(
    tx: &mut Transaction<'_, Sqlite>,
    c: &str,
    b: Option<&str>,
    at: &str,
    reason: &str,
) -> Result<(), AppError> {
    let Some(b) = b else { return Ok(()) };
    if !box_free(tx, b).await? {
        return Err(AppError::Validation(
            "A caixa anterior não está ativa e livre; a reversão automática foi bloqueada.".into(),
        ));
    }
    let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL)").bind(c).fetch_one(&mut**tx).await?;
    if active {
        return Err(AppError::Validation(
            "A colônia já possui uma ocupação ativa; a reversão não pode criar duas ocupações."
                .into(),
        ));
    }
    sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at,reason,notes)VALUES(?,?,?,?,?,?)").bind(Uuid::new_v4().to_string()).bind(c).bind(b).bind(at).bind("Retificação: restauração de ocupação").bind(Some(reason)).execute(&mut**tx).await?;
    Ok(())
}

pub async fn reverse_lifecycle(p: &SqlitePool, input: ReverseRecord) -> Result<(), AppError> {
    let id = req(&input.id, "Transição")?;
    let reason = req(&input.reason, "Motivo da reversão")?;
    let row:Option<(String,String,String,String,String,Option<String>,Option<String>)>=sqlx::query_as("SELECT colony_id,action,occurred_at,previous_status,new_status,box_id,reversed_at FROM colony_lifecycle_records WHERE id=?").bind(&id).fetch_optional(p).await?;
    let (c, action, at, previous, new_status, box_id, reversed) =
        row.ok_or_else(|| AppError::NotFound("Transição não encontrada.".into()))?;
    if reversed.is_some() {
        return Err(AppError::Validation(
            "Esta transição já foi revertida.".into(),
        ));
    }
    let latest:Option<String>=sqlx::query_scalar("SELECT id FROM colony_lifecycle_records WHERE colony_id=? AND reversed_at IS NULL ORDER BY occurred_at DESC,created_at DESC,id DESC LIMIT 1").bind(&c).fetch_optional(p).await?;
    if latest.as_deref() != Some(id.as_str()) {
        return Err(AppError::Validation(
            "Somente a transição efetiva mais recente pode ser revertida automaticamente.".into(),
        ));
    }
    if later_facts(p, &c, &at, Some(&id), None).await? {
        return Err(AppError::Validation(
            "Existem fatos posteriores incompatíveis com a reversão. Corrija esses fatos primeiro."
                .into(),
        ));
    }
    let current: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
        .bind(&c)
        .fetch_one(p)
        .await?;
    if current != new_status {
        return Err(AppError::Validation(
            "O estado atual da colônia já não corresponde ao resultado desta transição.".into(),
        ));
    }
    let mut tx = p.begin().await?;
    let reversed_at = now(&mut tx).await?;
    let before = json!({"id":id,"colony_id":c,"action":action,"occurred_at":at,"previous_status":previous,"new_status":new_status,"box_id":box_id});
    match action.as_str() {
        "loss" | "deactivate" => {
            sqlx::query("UPDATE colonies SET status=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                .bind(&previous)
                .bind(&c)
                .execute(&mut *tx)
                .await?;
            restore_box(&mut tx, &c, box_id.as_deref(), &reversed_at, &reason).await?
        }
        "reactivate" => {
            let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL)").bind(&c).fetch_one(&mut*tx).await?;
            if active {
                return Err(AppError::Validation("A colônia recebeu uma caixa após a reativação; a reversão automática foi bloqueada.".into()));
            }
            sqlx::query("UPDATE colonies SET status=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                .bind(&previous)
                .bind(&c)
                .execute(&mut *tx)
                .await?
        }
        _ => {
            return Err(AppError::Validation(
                "Ação de ciclo de vida sem reversão automática.".into(),
            ))
        }
    };
    sqlx::query("UPDATE colony_lifecycle_records SET reversed_at=?,reversal_reason=? WHERE id=?")
        .bind(&reversed_at)
        .bind(&reason)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "lifecycle",
        &id,
        "reverse",
        &reason,
        Some(before),
        Some(json!({"reversed_at":reversed_at,"reversal_reason":reason})),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn reverse_movement(p: &SqlitePool, input: ReverseRecord) -> Result<(), AppError> {
    let id = req(&input.id, "Movimentação")?;
    let reason = req(&input.reason, "Motivo da reversão")?;
    let row:Option<(String,String,String,String,Option<String>,Option<String>,Option<String>,Option<String>)>=sqlx::query_as("SELECT colony_id,movement_type,moved_at,from_meliponary_id,to_meliponary_id,from_box_id,to_box_id,reversed_at FROM colony_movements WHERE id=? AND voided_at IS NULL").bind(&id).fetch_optional(p).await?;
    let (c, kind, at, from_m, to_m, from_b, to_b, reversed) =
        row.ok_or_else(|| AppError::NotFound("Movimentação não encontrada.".into()))?;
    if reversed.is_some() {
        return Err(AppError::Validation(
            "Esta movimentação já foi revertida.".into(),
        ));
    }
    if kind == "transport" {
        return Err(AppError::Validation(
            "Transporte sem consequência deve ser anulado, não revertido.".into(),
        ));
    }
    let latest:Option<String>=sqlx::query_scalar("SELECT id FROM colony_movements WHERE colony_id=? AND movement_type IN('internal_transfer','external_transfer') AND voided_at IS NULL AND reversed_at IS NULL ORDER BY moved_at DESC,created_at DESC,id DESC LIMIT 1").bind(&c).fetch_optional(p).await?;
    if latest.as_deref() != Some(id.as_str()) {
        return Err(AppError::Validation(
            "Somente a transferência efetiva mais recente pode ser revertida automaticamente."
                .into(),
        ));
    }
    if later_facts(p, &c, &at, None, Some(&id)).await? {
        return Err(AppError::Validation(
            "Existem fatos posteriores à transferência; a reversão automática foi bloqueada."
                .into(),
        ));
    }
    let mut tx = p.begin().await?;
    let reversed_at = now(&mut tx).await?;
    let before = json!({"id":id,"colony_id":c,"movement_type":kind,"moved_at":at,"from_meliponary_id":from_m,"to_meliponary_id":to_m,"from_box_id":from_b,"to_box_id":to_b});
    match kind.as_str() {
        "internal_transfer" => {
            let current_m: String =
                sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id=?")
                    .bind(&c)
                    .fetch_one(&mut *tx)
                    .await?;
            if Some(current_m.as_str()) != to_m.as_deref() {
                return Err(AppError::Validation(
                    "A colônia já não está no destino registrado por esta transferência.".into(),
                ));
            }
            if let Some(tb) = &to_b {
                let occ:Option<(String,String)>=sqlx::query_as("SELECT id,started_at FROM colony_box_occupancies WHERE colony_id=? AND box_id=? AND ended_at IS NULL").bind(&c).bind(tb).fetch_optional(&mut*tx).await?;
                let (oid, start) = occ.ok_or_else(|| {
                    AppError::Validation(
                        "A ocupação criada pela transferência já não é a ocupação atual.".into(),
                    )
                })?;
                if start != at {
                    return Err(AppError::Validation(
                        "A ocupação atual não corresponde exatamente à transferência original."
                            .into(),
                    ));
                }
                sqlx::query("UPDATE colony_box_occupancies SET ended_at=? WHERE id=?")
                    .bind(&reversed_at)
                    .bind(oid)
                    .execute(&mut *tx)
                    .await?;
            } else {
                let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL)").bind(&c).fetch_one(&mut*tx).await?;
                if active {
                    return Err(AppError::Validation("A colônia recebeu uma caixa após a transferência; a reversão automática foi bloqueada.".into()));
                }
            }
            sqlx::query(
                "UPDATE colonies SET meliponary_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?",
            )
            .bind(&from_m)
            .bind(&c)
            .execute(&mut *tx)
            .await?;
            restore_box(&mut tx, &c, from_b.as_deref(), &reversed_at, &reason).await?
        }
        "external_transfer" => {
            let status: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
                .bind(&c)
                .fetch_one(&mut *tx)
                .await?;
            if status != "transferred" {
                return Err(AppError::Validation(
                    "A colônia já não está marcada como transferida.".into(),
                ));
            }
            let previous:Option<String>=sqlx::query_scalar("SELECT new_status FROM colony_lifecycle_records WHERE colony_id=? AND occurred_at<? AND reversed_at IS NULL ORDER BY occurred_at DESC,created_at DESC,id DESC LIMIT 1").bind(&c).bind(&at).fetch_optional(&mut*tx).await?;
            let previous=previous.ok_or_else(||AppError::Validation("O estado anterior à transferência externa não está historicamente comprovado; a reversão automática foi bloqueada.".into()))?;
            if !operational::is_manageable_status(&previous) {
                return Err(AppError::Validation(
                    "O estado anterior não permite restauração automática ao plantel.".into(),
                ));
            }
            sqlx::query("UPDATE colonies SET status=?,meliponary_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(previous).bind(&from_m).bind(&c).execute(&mut*tx).await?;
            restore_box(&mut tx, &c, from_b.as_deref(), &reversed_at, &reason).await?
        }
        _ => {
            return Err(AppError::Validation(
                "Tipo de movimentação sem reversão automática.".into(),
            ))
        }
    };
    sqlx::query("UPDATE colony_movements SET reversed_at=?,reversal_reason=? WHERE id=?")
        .bind(&reversed_at)
        .bind(&reason)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    audit::record_tx(
        &mut tx,
        "movement",
        &id,
        "reverse",
        &reason,
        Some(before),
        Some(json!({"reversed_at":reversed_at,"reversal_reason":reason})),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreateColony, CreateHiveBox, CreateMeliponary, CreateSpecies, PlaceColony},
        lifecycle::{self, ChangeColonyLifecycle},
        movements::{self, CreateMovement},
        repository,
    };
    use sqlx::sqlite::SqlitePoolOptions;
    struct S {
        p: SqlitePool,
        c: String,
        sm: String,
        tm: String,
        sb: String,
        tb: String,
    }
    async fn seed() -> S {
        let p = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../migrations").run(&p).await.unwrap();
        let sm = repository::create_meliponary(
            &p,
            CreateMeliponary {
                name: "Origem".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let tm = repository::create_meliponary(
            &p,
            CreateMeliponary {
                name: "Destino".into(),
                responsible_name: None,
                location: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let sp = repository::create_species(
            &p,
            CreateSpecies {
                common_name: "Jataí".into(),
                scientific_name: None,
                genus: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let sb = repository::create_box(
            &p,
            CreateHiveBox {
                meliponary_id: sm.id.clone(),
                code: "A-1".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let tb = repository::create_box(
            &p,
            CreateHiveBox {
                meliponary_id: tm.id.clone(),
                code: "B-1".into(),
                model: None,
                material: None,
                location_note: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let c = repository::create_colony(
            &p,
            CreateColony {
                meliponary_id: sm.id.clone(),
                species_id: sp.id,
                code: "JAT-001".into(),
                origin_type: None,
                origin_notes: None,
                installed_at: Some("2026-01-01 09:00:00".into()),
                mother_colony_id: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        repository::place_colony(
            &p,
            PlaceColony {
                colony_id: c.id.clone(),
                box_id: sb.id.clone(),
                started_at: Some("2026-01-01 09:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        S {
            p,
            c: c.id,
            sm: sm.id,
            tm: tm.id,
            sb: sb.id,
            tb: tb.id,
        }
    }
    #[tokio::test]
    async fn safe_lifecycle_reversal_restores_status_and_box() {
        let s = seed().await;
        let r = lifecycle::change(
            &s.p,
            ChangeColonyLifecycle {
                colony_id: s.c.clone(),
                action: "loss".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: Some("Erro".into()),
                notes: None,
            },
        )
        .await
        .unwrap();
        reverse_lifecycle(
            &s.p,
            ReverseRecord {
                id: r.id.clone(),
                reason: "Engano".into(),
            },
        )
        .await
        .unwrap();
        let st: String = sqlx::query_scalar("SELECT status FROM colonies WHERE id=?")
            .bind(&s.c)
            .fetch_one(&s.p)
            .await
            .unwrap();
        let b: String = sqlx::query_scalar(
            "SELECT box_id FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL",
        )
        .bind(&s.c)
        .fetch_one(&s.p)
        .await
        .unwrap();
        assert_eq!(st, "active");
        assert_eq!(b, s.sb);
        assert!(sqlx::query_scalar::<_, Option<String>>(
            "SELECT reversed_at FROM colony_lifecycle_records WHERE id=?"
        )
        .bind(r.id)
        .fetch_one(&s.p)
        .await
        .unwrap()
        .is_some());
    }
    #[tokio::test]
    async fn conflicting_lifecycle_reversal_is_rejected() {
        let s = seed().await;
        let r = lifecycle::change(
            &s.p,
            ChangeColonyLifecycle {
                colony_id: s.c.clone(),
                action: "deactivate".into(),
                occurred_at: Some("2026-02-01 10:00:00".into()),
                reason: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("INSERT INTO colony_events(id,colony_id,event_type,occurred_at,severity)VALUES('e1',?,'observation','2026-02-02 10:00:00','info')").bind(&s.c).execute(&s.p).await.unwrap();
        assert!(reverse_lifecycle(
            &s.p,
            ReverseRecord {
                id: r.id,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
    }
    #[tokio::test]
    async fn internal_transfer_reversal_is_transactional_and_preserves_original() {
        let s = seed().await;
        let m = movements::create(
            &s.p,
            CreateMovement {
                colony_id: s.c.clone(),
                movement_type: "internal_transfer".into(),
                moved_at: Some("2026-02-01 10:00:00".into()),
                to_meliponary_id: Some(s.tm),
                to_box_id: Some(s.tb),
                destination: None,
                document_reference: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        reverse_movement(
            &s.p,
            ReverseRecord {
                id: m.id.clone(),
                reason: "Destino incorreto".into(),
            },
        )
        .await
        .unwrap();
        let mel: String = sqlx::query_scalar("SELECT meliponary_id FROM colonies WHERE id=?")
            .bind(&s.c)
            .fetch_one(&s.p)
            .await
            .unwrap();
        let b: String = sqlx::query_scalar(
            "SELECT box_id FROM colony_box_occupancies WHERE colony_id=? AND ended_at IS NULL",
        )
        .bind(&s.c)
        .fetch_one(&s.p)
        .await
        .unwrap();
        assert_eq!(mel, s.sm);
        assert_eq!(b, s.sb);
        let kept: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM colony_movements WHERE id=? AND reversed_at IS NOT NULL",
        )
        .bind(m.id)
        .fetch_one(&s.p)
        .await
        .unwrap();
        assert_eq!(kept, 1);
    }
    #[tokio::test]
    async fn movement_reversal_with_later_fact_is_rejected() {
        let s = seed().await;
        let m = movements::create(
            &s.p,
            CreateMovement {
                colony_id: s.c.clone(),
                movement_type: "internal_transfer".into(),
                moved_at: Some("2026-02-01 10:00:00".into()),
                to_meliponary_id: Some(s.tm),
                to_box_id: Some(s.tb),
                destination: None,
                document_reference: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("INSERT INTO colony_events(id,colony_id,event_type,occurred_at,severity)VALUES('e1',?,'observation','2026-02-02 10:00:00','info')").bind(&s.c).execute(&s.p).await.unwrap();
        assert!(reverse_movement(
            &s.p,
            ReverseRecord {
                id: m.id,
                reason: "Teste".into()
            }
        )
        .await
        .is_err());
    }
}
