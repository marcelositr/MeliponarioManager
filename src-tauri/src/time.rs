use crate::repository::AppError;
use sqlx::SqlitePool;

fn parse_component(value: &str, field: &str) -> Result<u32, AppError> {
    value.parse::<u32>().map_err(|_| {
        AppError::Validation(format!("Timestamp inválido: componente {field} inválido."))
    })
}

fn leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn validate_date(date: &str) -> Result<(), AppError> {
    if date.len() != 10 || &date[4..5] != "-" || &date[7..8] != "-" {
        return Err(AppError::Validation(
            "Data inválida. Use YYYY-MM-DD.".to_owned(),
        ));
    }

    let year = parse_component(&date[0..4], "ano")?;
    let month = parse_component(&date[5..7], "mês")?;
    let day = parse_component(&date[8..10], "dia")?;
    if year == 0 || !(1..=12).contains(&month) {
        return Err(AppError::Validation("Data inválida.".to_owned()));
    }

    let max_day = match month {
        2 if leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if day == 0 || day > max_day {
        return Err(AppError::Validation("Data inválida.".to_owned()));
    }
    Ok(())
}

fn validate_time(time: &str) -> Result<(), AppError> {
    if time.len() != 8 || &time[2..3] != ":" || &time[5..6] != ":" {
        return Err(AppError::Validation(
            "Horário inválido. Use HH:MM:SS.".to_owned(),
        ));
    }
    let hour = parse_component(&time[0..2], "hora")?;
    let minute = parse_component(&time[3..5], "minuto")?;
    let second = parse_component(&time[6..8], "segundo")?;
    if hour > 23 || minute > 59 || second > 59 {
        return Err(AppError::Validation("Horário inválido.".to_owned()));
    }
    Ok(())
}

pub fn normalize(value: &str, allow_date: bool) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation("Timestamp vazio.".to_owned()));
    }

    if value.len() == 10 {
        if !allow_date {
            return Err(AppError::Validation(
                "Informe data e horário para este registro.".to_owned(),
            ));
        }
        validate_date(value)?;
        return Ok(format!("{value} 00:00:00"));
    }

    let normalized = match value.len() {
        16 if &value[10..11] == "T" || &value[10..11] == " " => {
            format!("{} {}:00", &value[0..10], &value[11..16])
        }
        19 if &value[10..11] == "T" || &value[10..11] == " " => {
            format!("{} {}", &value[0..10], &value[11..19])
        }
        _ => {
            return Err(AppError::Validation(
                "Timestamp inválido. Use YYYY-MM-DDTHH:MM, YYYY-MM-DDTHH:MM:SS ou YYYY-MM-DD HH:MM:SS."
                    .to_owned(),
            ));
        }
    };

    validate_date(&normalized[0..10])?;
    validate_time(&normalized[11..19])?;
    Ok(normalized)
}

pub fn normalize_optional(
    value: &Option<String>,
    allow_date: bool,
) -> Result<Option<String>, AppError> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| normalize(value, allow_date))
        .transpose()
}

pub async fn local_now(pool: &SqlitePool) -> Result<String, AppError> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT datetime('now', 'localtime')")
            .fetch_one(pool)
            .await?,
    )
}

pub async fn normalize_or_now(
    pool: &SqlitePool,
    value: &Option<String>,
    allow_date: bool,
) -> Result<String, AppError> {
    match normalize_optional(value, allow_date)? {
        Some(value) => Ok(value),
        None => local_now(pool).await,
    }
}

pub fn ensure_not_before(
    candidate: &Option<String>,
    origin: &str,
    message: &str,
) -> Result<(), AppError> {
    if candidate.as_deref().is_some_and(|value| value < origin) {
        return Err(AppError::Validation(message.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn normalizes_supported_formats() {
        assert_eq!(
            normalize("2026-08-27T17:42", false).unwrap(),
            "2026-08-27 17:42:00"
        );
        assert_eq!(
            normalize("2026-08-27T17:42:31", false).unwrap(),
            "2026-08-27 17:42:31"
        );
        assert_eq!(
            normalize("2026-08-27 17:42:31", false).unwrap(),
            "2026-08-27 17:42:31"
        );
        assert_eq!(
            normalize("2026-08-27", true).unwrap(),
            "2026-08-27 00:00:00"
        );
    }

    #[test]
    fn rejects_invalid_timestamp() {
        assert!(normalize("2026-02-30T10:00", false).is_err());
        assert!(normalize("2026-08-27T25:00", false).is_err());
        assert!(normalize("banana", false).is_err());
    }

    #[tokio::test]
    async fn local_now_uses_canonical_shape() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let now = local_now(&pool).await.unwrap();
        assert_eq!(now.len(), 19);
        assert_eq!(normalize(&now, false).unwrap(), now);
    }
}
