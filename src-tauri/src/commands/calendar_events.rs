use tauri::State;
use crate::{DbState, models::calendar_event::CalendarEvent, utils::is_valid_time};

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
) -> Result<CalendarEvent, String> {
    let pool = &state.0;

    // Validate: either (rrule + start_time + end_time) OR (start_at + end_at)
    let has_recurring = data.rrule.is_some() && data.start_time.is_some() && data.end_time.is_some();
    let has_single = data.start_at.is_some() && data.end_at.is_some();

    if !has_recurring && !has_single {
        return Err("Must provide either (rrule + start_time + end_time) for recurring events OR (start_at + end_at) for single events".to_string());
    }

    // Validate time format if provided
    if let Some(ref start_time) = data.start_time {
        if !is_valid_time(start_time) {
            return Err("Invalid start_time format. Use HH:MM (24-hour)".to_string());
        }
    }
    if let Some(ref end_time) = data.end_time {
        if !is_valid_time(end_time) {
            return Err("Invalid end_time format. Use HH:MM (24-hour)".to_string());
        }
    }

    // Validate start_time < end_time if both provided
    if let (Some(ref start), Some(ref end)) = (&data.start_time, &data.end_time) {
        if start >= end {
            return Err("start_time must be before end_time".to_string());
        }
    }

    let user_id = data.user_id.unwrap_or(1);
    let category = data.category.unwrap_or_else(|| "general".to_string());

    let rec = sqlx::query_as::<_, CalendarEvent>(
        r#"INSERT INTO calendar_events (user_id, title, start_at, end_at, rrule, start_time, end_time, category, domain, linked_id, locked, notes)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING *"#
    )
    .bind(user_id)
    .bind(&data.title)
    .bind(&data.start_at)
    .bind(&data.end_at)
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
        "Failed to create calendar event".to_string()
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn get_calendar_events(
    state: State<'_, DbState>,
    category: Option<String>,
) -> Result<Vec<CalendarEvent>, String> {
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
        "Failed to fetch calendar events".to_string()
    })
}

#[tauri::command]
pub async fn get_calendar_event(
    state: State<'_, DbState>,
    id: i64,
) -> Result<CalendarEvent, String> {
    let pool = &state.0;

    sqlx::query_as::<_, CalendarEvent>(
        "SELECT * FROM calendar_events WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch calendar event {}: {}", id, e);
        "Calendar event not found".to_string()
    })
}

#[tauri::command]
pub async fn update_calendar_event(
    state: State<'_, DbState>,
    id: i64,
    data: CalendarEventInput,
) -> Result<CalendarEvent, String> {
    let pool = &state.0;

    // Validate time format if provided
    if let Some(ref start_time) = data.start_time {
        if !is_valid_time(start_time) {
            return Err("Invalid start_time format. Use HH:MM (24-hour)".to_string());
        }
    }
    if let Some(ref end_time) = data.end_time {
        if !is_valid_time(end_time) {
            return Err("Invalid end_time format. Use HH:MM (24-hour)".to_string());
        }
    }

    // Validate start_time < end_time if both provided
    if let (Some(ref start), Some(ref end)) = (&data.start_time, &data.end_time) {
        if start >= end {
            return Err("start_time must be before end_time".to_string());
        }
    }

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
    .bind(&data.start_at)
    .bind(&data.end_at)
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
        "Failed to update calendar event".to_string()
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn delete_calendar_event(
    state: State<'_, DbState>,
    id: i64,
) -> Result<bool, String> {
    let pool = &state.0;

    let result = sqlx::query("DELETE FROM calendar_events WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete calendar event {}: {}", id, e);
            "Failed to delete calendar event".to_string()
        })?;

    if result.rows_affected() == 0 {
        return Err("Calendar event not found".to_string());
    }

    Ok(true)
}

