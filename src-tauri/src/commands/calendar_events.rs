use tauri::State;
use crate::{
    DbState,
    error::ApiError,
    models::calendar_event::CalendarEvent,
    utils::{is_valid_time, parse_datetime_to_rfc3339},
};

#[cfg(test)]
use sqlx::sqlite::SqlitePoolOptions;

#[derive(Debug, serde::Deserialize)]
pub struct CalendarEventInput {
    #[serde(default)]
    pub user_id: Option<i64>,
    pub title: String,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub rrule: Option<String>,
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub linked_id: Option<i64>,
    #[serde(default)]
    pub locked: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn create_calendar_event(
    state: State<'_, DbState>,
    data: CalendarEventInput,
) -> Result<CalendarEvent, ApiError> {
    create_calendar_event_for_test(&state.0, data).await
}

async fn create_calendar_event_for_test(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    data: CalendarEventInput,
) -> Result<CalendarEvent, ApiError> {

    // Validate: either (rrule + start_time + end_time) OR (start_at + end_at)
    let has_recurring = data.rrule.is_some() && data.start_time.is_some() && data.end_time.is_some();
    let has_single = data.start_at.is_some() && data.end_at.is_some();

    if !has_recurring && !has_single {
        return Err(ApiError::validation(
            "Must provide either (rrule + start_time + end_time) for recurring events OR (start_at + end_at) for single events",
        ));
    }

    // Validate time format if provided
    if let Some(ref start_time) = data.start_time {
        if !is_valid_time(start_time) {
            return Err(ApiError::validation(
                "Invalid start_time format. Use HH:MM (24-hour)",
            ));
        }
    }
    if let Some(ref end_time) = data.end_time {
        if !is_valid_time(end_time) {
            return Err(ApiError::validation(
                "Invalid end_time format. Use HH:MM (24-hour)",
            ));
        }
    }

    // Validate start_time < end_time if both provided
    if let (Some(ref start), Some(ref end)) = (&data.start_time, &data.end_time) {
        if start >= end {
            return Err(ApiError::validation("start_time must be before end_time"));
        }
    }

    let normalized_start_at = data
        .start_at
        .as_deref()
        .and_then(parse_datetime_to_rfc3339)
        .or_else(|| data.start_at.clone());
    let normalized_end_at = data
        .end_at
        .as_deref()
        .and_then(parse_datetime_to_rfc3339)
        .or_else(|| data.end_at.clone());

    let user_id = data.user_id.unwrap_or(1);
    let category = data.category.unwrap_or_else(|| "general".to_string());

    let rec = sqlx::query_as::<_, CalendarEvent>(
        r#"INSERT INTO calendar_events (user_id, title, start_at, end_at, rrule, start_time, end_time, category, domain, linked_id, locked, notes)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING *"#
    )
    .bind(user_id)
    .bind(&data.title)
    .bind(&normalized_start_at)
    .bind(&normalized_end_at)
    .bind(&data.rrule)
    .bind(&data.start_time)
    .bind(&data.end_time)
    .bind(&category)
    .bind(&data.domain)
    .bind(&data.linked_id)
    .bind(&data.locked)
    .bind(&data.notes)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to create calendar event: {}", e);
        ApiError::from_sqlx(e, "Failed to create calendar event")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn get_calendar_events(
    state: State<'_, DbState>,
    category: Option<String>,
) -> Result<Vec<CalendarEvent>, ApiError> {
    let pool = &state.0;

    let events = if let Some(cat) = category {
        sqlx::query_as::<_, CalendarEvent>(
            "SELECT * FROM calendar_events WHERE category = ? ORDER BY start_at, start_time"
        )
        .bind(cat)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, CalendarEvent>(
            "SELECT * FROM calendar_events ORDER BY start_at, start_time"
        )
        .fetch_all(pool)
        .await
    };

    events.map_err(|e| {
        log::error!("Failed to fetch calendar events: {}", e);
        ApiError::from_sqlx(e, "Failed to fetch calendar events")
    })
}

#[tauri::command]
pub async fn get_calendar_event(
    state: State<'_, DbState>,
    id: i64,
) -> Result<CalendarEvent, ApiError> {
    let pool = &state.0;

    sqlx::query_as::<_, CalendarEvent>(
        "SELECT * FROM calendar_events WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch calendar event {}: {}", id, e);
        ApiError::from_sqlx(e, "Calendar event not found")
    })
}

#[tauri::command]
pub async fn update_calendar_event(
    state: State<'_, DbState>,
    id: i64,
    data: CalendarEventInput,
) -> Result<CalendarEvent, ApiError> {
    update_calendar_event_for_test(&state.0, id, data).await
}

async fn update_calendar_event_for_test(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    id: i64,
    data: CalendarEventInput,
) -> Result<CalendarEvent, ApiError> {

    // Validate time format if provided
    if let Some(ref start_time) = data.start_time {
        if !is_valid_time(start_time) {
            return Err(ApiError::validation(
                "Invalid start_time format. Use HH:MM (24-hour)",
            ));
        }
    }
    if let Some(ref end_time) = data.end_time {
        if !is_valid_time(end_time) {
            return Err(ApiError::validation(
                "Invalid end_time format. Use HH:MM (24-hour)",
            ));
        }
    }

    // Validate start_time < end_time if both provided
    if let (Some(ref start), Some(ref end)) = (&data.start_time, &data.end_time) {
        if start >= end {
            return Err(ApiError::validation("start_time must be before end_time"));
        }
    }

    let normalized_start_at = data
        .start_at
        .as_deref()
        .and_then(parse_datetime_to_rfc3339)
        .or_else(|| data.start_at.clone());
    let normalized_end_at = data
        .end_at
        .as_deref()
        .and_then(parse_datetime_to_rfc3339)
        .or_else(|| data.end_at.clone());

    let rec = sqlx::query_as::<_, CalendarEvent>(
        r#"UPDATE calendar_events
           SET user_id = COALESCE(?, user_id),
               title = COALESCE(?, title),
               start_at = COALESCE(?, start_at),
               end_at = COALESCE(?, end_at),
               rrule = COALESCE(?, rrule),
               start_time = COALESCE(?, start_time),
               end_time = COALESCE(?, end_time),
               category = COALESCE(?, category),
               domain = COALESCE(?, domain),
               linked_id = COALESCE(?, linked_id),
               locked = COALESCE(?, locked),
               notes = COALESCE(?, notes)
           WHERE id = ?
           RETURNING *"#
    )
    .bind(&data.user_id)
    .bind(&data.title)
    .bind(&normalized_start_at)
    .bind(&normalized_end_at)
    .bind(&data.rrule)
    .bind(&data.start_time)
    .bind(&data.end_time)
    .bind(&data.category)
    .bind(&data.domain)
    .bind(&data.linked_id)
    .bind(&data.locked)
    .bind(&data.notes)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to update calendar event {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to update calendar event")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn delete_calendar_event(
    state: State<'_, DbState>,
    id: i64,
) -> Result<bool, ApiError> {
    let pool = &state.0;

    let result = sqlx::query("DELETE FROM calendar_events WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete calendar event {}: {}", id, e);
            ApiError::from_sqlx(e, "Failed to delete calendar event")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Calendar event not found"));
    }

    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;

        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(false);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to connect to in-memory DB");

        crate::db::migrations::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn calendar_event_input_normalizes_rfc3339() {
        let pool = setup_db().await;

        let input = CalendarEventInput {
            user_id: Some(1),
            title: "Normalized Event".to_string(),
            start_at: Some("2026-02-07T09:30:00".to_string()),
            end_at: Some("2026-02-07T10:30:00".to_string()),
            rrule: None,
            start_time: None,
            end_time: None,
            category: Some("busy".to_string()),
            domain: None,
            linked_id: None,
            locked: None,
            notes: None,
        };

        let created = create_calendar_event_for_test(&pool, input).await.unwrap();

        let start_at = created.start_at.expect("start_at missing");
        let end_at = created.end_at.expect("end_at missing");

        assert!(start_at.contains('Z') || start_at.contains('+'));
        assert!(end_at.contains('Z') || end_at.contains('+'));
    }
}
