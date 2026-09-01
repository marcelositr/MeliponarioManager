use crate::repository::AppError;
use sqlx::SqlitePool;

pub(crate) fn text_key(value: &str) -> String {
    value.trim().to_lowercase()
}

pub(crate) fn species_key(
    common_name: &str,
    scientific_name: Option<&str>,
    genus: Option<&str>,
) -> String {
    if let Some(scientific_name) = scientific_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("scientific:{}", text_key(scientific_name));
    }

    format!(
        "common:{}|genus:{}",
        text_key(common_name),
        text_key(genus.unwrap_or_default())
    )
}

pub(crate) async fn ensure_meliponary_name_available(
    pool: &SqlitePool,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<(), AppError> {
    let key = text_key(name);
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT id, name FROM meliponaries")
        .fetch_all(pool)
        .await?;
    if rows
        .iter()
        .any(|(id, value)| Some(id.as_str()) != exclude_id && text_key(value) == key)
    {
        return Err(AppError::Validation(
            "Já existe outro meliponário com este nome.".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) async fn ensure_box_code_available(
    pool: &SqlitePool,
    meliponary_id: &str,
    code: &str,
    exclude_id: Option<&str>,
) -> Result<(), AppError> {
    let key = text_key(code);
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, code FROM boxes WHERE meliponary_id = ?")
            .bind(meliponary_id)
            .fetch_all(pool)
            .await?;
    if rows
        .iter()
        .any(|(id, value)| Some(id.as_str()) != exclude_id && text_key(value) == key)
    {
        return Err(AppError::Validation(
            "Já existe uma caixa com esta identificação no meliponário.".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) async fn ensure_colony_code_available(
    pool: &SqlitePool,
    meliponary_id: &str,
    code: &str,
    exclude_id: Option<&str>,
) -> Result<(), AppError> {
    let key = text_key(code);
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, code FROM colonies WHERE meliponary_id = ?")
            .bind(meliponary_id)
            .fetch_all(pool)
            .await?;
    if rows
        .iter()
        .any(|(id, value)| Some(id.as_str()) != exclude_id && text_key(value) == key)
    {
        return Err(AppError::Validation(
            "Já existe uma colônia com esta identificação no meliponário.".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) async fn ensure_species_identity_available(
    pool: &SqlitePool,
    common_name: &str,
    scientific_name: Option<&str>,
    genus: Option<&str>,
    exclude_id: Option<&str>,
) -> Result<(), AppError> {
    type SpeciesIdentityRow = (String, String, Option<String>, Option<String>);
    let key = species_key(common_name, scientific_name, genus);
    let rows: Vec<SpeciesIdentityRow> =
        sqlx::query_as("SELECT id, common_name, scientific_name, genus FROM species")
            .fetch_all(pool)
            .await?;
    if rows.iter().any(|(id, common, scientific, genus)| {
        Some(id.as_str()) != exclude_id
            && species_key(common, scientific.as_deref(), genus.as_deref()) == key
    }) {
        return Err(AppError::Validation(
            "Já existe uma espécie com esta identidade cadastral.".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_identity_trims_and_uses_unicode_lowercase() {
        assert_eq!(text_key("  JATAÍ  "), "jataí");
    }

    #[test]
    fn species_identity_prefers_scientific_name_and_falls_back_to_common_name_and_genus() {
        assert_eq!(
            species_key(
                "Jataí",
                Some(" Tetragonisca Angustula "),
                Some("Tetragonisca")
            ),
            "scientific:tetragonisca angustula"
        );
        assert_eq!(
            species_key(" JATAÍ ", None, Some(" Tetragonisca ")),
            "common:jataí|genus:tetragonisca"
        );
    }
}
