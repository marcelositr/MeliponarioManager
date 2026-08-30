use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Meliponary {
    pub id: String,
    pub name: String,
    pub responsible_name: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub archived_at: Option<String>,
    pub archive_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMeliponary {
    pub name: String,
    pub responsible_name: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Species {
    pub id: String,
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub genus: Option<String>,
    pub notes: Option<String>,
    pub archived_at: Option<String>,
    pub archive_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpecies {
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub genus: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HiveBox {
    pub id: String,
    pub meliponary_id: String,
    pub code: String,
    pub model: Option<String>,
    pub material: Option<String>,
    pub location_note: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub current_colony_code: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHiveBox {
    pub meliponary_id: String,
    pub code: String,
    pub model: Option<String>,
    pub material: Option<String>,
    pub location_note: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Colony {
    pub id: String,
    pub meliponary_id: String,
    pub species_id: String,
    pub code: String,
    pub origin_type: String,
    pub origin_notes: Option<String>,
    pub installed_at: Option<String>,
    pub status: String,
    pub mother_colony_id: Option<String>,
    pub notes: Option<String>,
    pub current_box_code: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateColony {
    pub meliponary_id: String,
    pub species_id: String,
    pub code: String,
    pub origin_type: Option<String>,
    pub origin_notes: Option<String>,
    pub installed_at: Option<String>,
    pub mother_colony_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceColony {
    pub colony_id: String,
    pub box_id: String,
    pub started_at: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ColonyBoxOccupancy {
    pub id: String,
    pub colony_id: String,
    pub box_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CoreSummary {
    pub meliponaries: i64,
    pub species: i64,
    pub colonies: i64,
    pub boxes: i64,
}
