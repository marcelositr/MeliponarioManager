use crate::{
    history::{self, TimelineEntry},
    lifecycle, maintenance,
    repository::AppError,
};
use sqlx::SqlitePool;

pub async fn by_colony(
    pool: &SqlitePool,
    colony_id: &str,
) -> Result<Vec<TimelineEntry>, AppError> {
    let mut entries = history::timeline_by_colony(pool, colony_id).await?;
    entries.extend(maintenance::timeline_entries_by_colony(pool, colony_id).await?);
    entries.extend(lifecycle::timeline_entries(pool, colony_id).await?);

    entries.sort_by(|left, right| {
        right
            .occurred_at
            .cmp(&left.occurred_at)
            .then_with(|| right.source_id.cmp(&left.source_id))
    });

    Ok(entries)
}
