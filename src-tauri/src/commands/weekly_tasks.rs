use tauri::State;
use crate::{DbState, error::ApiError, models::weekly_task::WeeklyTask};

#[derive(Debug, serde::Deserialize)]
pub struct WeeklyTaskInput {
    #[serde(default)]
    pub user_id: Option<i64>,
    pub week_start_date: String,
    pub title: String,
    #[serde(default)]
    pub course_id: Option<i64>,
    #[serde(default)]
    pub duration_minutes: Option<i64>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub completed: Option<i64>,
}

fn get_current_week_start() -> String {
    use chrono::{Datelike, Local};
    let today = Local::now().date_naive();
    let days_since_monday = today.weekday().num_days_from_monday();
    let monday = today - chrono::Duration::days(days_since_monday as i64);
    monday.format("%Y-%m-%d").to_string()
}

fn is_valid_priority(priority: &str) -> bool {
    matches!(priority, "low" | "medium" | "high")
}

#[tauri::command]
pub async fn create_weekly_task(
    state: State<'_, DbState>,
    data: WeeklyTaskInput,
) -> Result<WeeklyTask, ApiError> {
    let pool = &state.0;

    // Validate title not empty
    if data.title.trim().is_empty() {
        return Err(ApiError::validation("Title cannot be empty"));
    }

    // Validate priority if provided
    if let Some(ref priority) = data.priority {
        if !is_valid_priority(priority) {
            return Err(ApiError::validation(
                "Priority must be 'low', 'medium', or 'high'",
            ));
        }
    }

    let user_id = data.user_id.unwrap_or(1);
    let completed = data.completed.unwrap_or(0);

    let rec = sqlx::query_as::<_, WeeklyTask>(
        r#"INSERT INTO weekly_tasks (user_id, week_start_date, title, course_id, duration_minutes, priority, notes, completed)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING *"#
    )
    .bind(user_id)
    .bind(&data.week_start_date)
    .bind(&data.title)
    .bind(&data.course_id)
    .bind(&data.duration_minutes)
    .bind(&data.priority)
    .bind(&data.notes)
    .bind(completed)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to create weekly task: {}", e);
        ApiError::from_sqlx(e, "Failed to create weekly task")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn get_weekly_tasks(
    state: State<'_, DbState>,
    week_start_date: Option<String>,
) -> Result<Vec<WeeklyTask>, ApiError> {
    let pool = &state.0;

    let week_date = week_start_date.unwrap_or_else(get_current_week_start);

    let tasks = sqlx::query_as::<_, WeeklyTask>(
        "SELECT * FROM weekly_tasks WHERE week_start_date = ? ORDER BY priority DESC, created_at ASC"
    )
    .bind(&week_date)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch weekly tasks: {}", e);
        ApiError::from_sqlx(e, "Failed to fetch weekly tasks")
    })?;

    Ok(tasks)
}

#[tauri::command]
pub async fn update_weekly_task(
    state: State<'_, DbState>,
    id: i64,
    data: WeeklyTaskInput,
) -> Result<WeeklyTask, ApiError> {
    let pool = &state.0;

    // Validate title not empty
    if data.title.trim().is_empty() {
        return Err(ApiError::validation("Title cannot be empty"));
    }

    // Validate priority if provided
    if let Some(ref priority) = data.priority {
        if !is_valid_priority(priority) {
            return Err(ApiError::validation(
                "Priority must be 'low', 'medium', or 'high'",
            ));
        }
    }

    let rec = sqlx::query_as::<_, WeeklyTask>(
        r#"UPDATE weekly_tasks
           SET user_id = COALESCE(?, user_id),
               week_start_date = COALESCE(?, week_start_date),
               title = COALESCE(?, title),
               course_id = COALESCE(?, course_id),
               duration_minutes = COALESCE(?, duration_minutes),
               priority = COALESCE(?, priority),
               notes = COALESCE(?, notes),
               completed = COALESCE(?, completed)
           WHERE id = ?
           RETURNING *"#
    )
    .bind(&data.user_id)
    .bind(&data.week_start_date)
    .bind(&data.title)
    .bind(&data.course_id)
    .bind(&data.duration_minutes)
    .bind(&data.priority)
    .bind(&data.notes)
    .bind(&data.completed)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to update weekly task {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to update weekly task")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn toggle_weekly_task(
    state: State<'_, DbState>,
    id: i64,
) -> Result<WeeklyTask, ApiError> {
    let pool = &state.0;

    let rec = sqlx::query_as::<_, WeeklyTask>(
        r#"UPDATE weekly_tasks
           SET completed = CASE WHEN completed = 1 THEN 0 ELSE 1 END
           WHERE id = ?
           RETURNING *"#
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to toggle weekly task {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to toggle weekly task")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn delete_weekly_task(
    state: State<'_, DbState>,
    id: i64,
) -> Result<bool, ApiError> {
    let pool = &state.0;

    let result = sqlx::query("DELETE FROM weekly_tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete weekly task {}: {}", id, e);
            ApiError::from_sqlx(e, "Failed to delete weekly task")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Weekly task not found"));
    }

    Ok(true)
}
