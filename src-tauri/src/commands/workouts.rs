use tauri::State;

use crate::{DbState, error::ApiError, models::workout::Workout};

#[derive(Debug, serde::Deserialize)]
pub struct WorkoutInput {
    pub user_id: Option<i64>,
    pub name: Option<String>,
    pub duration_minutes: Option<i64>,
    pub notes: Option<String>,
    pub logged_at: Option<String>,
}

#[tauri::command]
pub async fn create_workout(state: State<'_, DbState>, data: WorkoutInput) -> Result<Workout, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Workout>(
        "INSERT INTO workouts (user_id, name, duration_minutes, notes, logged_at) VALUES (?, ?, ?, ?, COALESCE(?, CURRENT_TIMESTAMP)) RETURNING id, user_id, name, duration_minutes, notes, logged_at"
    )
    .bind(data.user_id.unwrap_or(1))
    .bind(&data.name)
    .bind(data.duration_minutes)
    .bind(&data.notes)
    .bind(&data.logged_at)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn get_workouts(state: State<'_, DbState>) -> Result<Vec<Workout>, ApiError> {
    let pool = &state.0;
    let rows = sqlx::query_as::<_, Workout>("SELECT id, user_id, name, duration_minutes, notes, logged_at FROM workouts ORDER BY logged_at DESC")
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_workout(state: State<'_, DbState>, id: i64) -> Result<Workout, ApiError> {
    let pool = &state.0;
    let row = sqlx::query_as::<_, Workout>("SELECT id, user_id, name, duration_minutes, notes, logged_at FROM workouts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(ApiError::from)?;
    Ok(row)
}

#[tauri::command]
pub async fn delete_workout(state: State<'_, DbState>, id: i64) -> Result<bool, ApiError> {
    let pool = &state.0;
    let result = sqlx::query("DELETE FROM workouts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ApiError::from)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Workout not found"));
    }

    Ok(true)
}

#[tauri::command]
pub async fn update_workout(
    state: State<'_, DbState>,
    id: i64,
    data: WorkoutInput,
) -> Result<Workout, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Workout>(
        "UPDATE workouts SET 
            name = COALESCE(?, name),
            duration_minutes = COALESCE(?, duration_minutes),
            notes = COALESCE(?, notes),
            logged_at = COALESCE(?, logged_at)
         WHERE id = ? 
         RETURNING id, user_id, name, duration_minutes, notes, logged_at"
    )
    .bind(&data.name)
    .bind(data.duration_minutes)
    .bind(&data.notes)
    .bind(&data.logged_at)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}
