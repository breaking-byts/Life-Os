use tauri::State;

use crate::{DbState, error::ApiError, models::session::Session};

#[derive(Debug, serde::Deserialize)]
pub struct SessionInput {
    pub user_id: Option<i64>,
    pub session_type: String,
    pub reference_id: Option<i64>,
    pub reference_type: Option<String>,
    pub started_at: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn start_session(state: State<'_, DbState>, data: SessionInput) -> Result<Session, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (user_id, session_type, reference_id, reference_type, started_at, notes) VALUES (?, ?, ?, ?, COALESCE(?, CURRENT_TIMESTAMP), ?) RETURNING id, user_id, session_type, reference_id, reference_type, started_at, ended_at, duration_minutes, notes"
    )
    .bind(data.user_id.unwrap_or(1))
    .bind(&data.session_type)
    .bind(data.reference_id)
    .bind(&data.reference_type)
    .bind(&data.started_at)
    .bind(&data.notes)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn end_session(state: State<'_, DbState>, id: i64) -> Result<Session, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Session>(
        "UPDATE sessions SET ended_at = COALESCE(ended_at, CURRENT_TIMESTAMP), duration_minutes = CAST((strftime('%s', COALESCE(ended_at, CURRENT_TIMESTAMP)) - strftime('%s', started_at)) / 60 AS INTEGER) WHERE id = ? RETURNING id, user_id, session_type, reference_id, reference_type, started_at, ended_at, duration_minutes, notes"
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn get_sessions(state: State<'_, DbState>, reference_id: Option<i64>, reference_type: Option<String>) -> Result<Vec<Session>, ApiError> {
    let pool = &state.0;
    let rows = match (reference_id, reference_type) {
        (Some(id), Some(rtype)) => sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE reference_id = ? AND reference_type = ? ORDER BY started_at DESC")
            .bind(id)
            .bind(&rtype)
            .fetch_all(pool)
            .await
            .map_err(ApiError::from)?,
        _ => sqlx::query_as::<_, Session>("SELECT * FROM sessions ORDER BY started_at DESC")
            .fetch_all(pool)
            .await
            .map_err(ApiError::from)?,
    };
    Ok(rows)
}
