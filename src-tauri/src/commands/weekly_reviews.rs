use tauri::State;

use crate::{DbState, error::ApiError};

#[derive(Debug, serde::Deserialize)]
pub struct WeeklyReviewInput {
    pub user_id: Option<i64>,
    pub week_start: String,
    pub wins: Option<String>,
    pub improvements: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct WeeklyReview {
    pub id: i64,
    pub user_id: i64,
    pub week_start: String,
    pub wins: Option<String>,
    pub improvements: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[tauri::command]
pub async fn create_weekly_review(state: State<'_, DbState>, data: WeeklyReviewInput) -> Result<WeeklyReview, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, WeeklyReview>(
        "INSERT INTO weekly_reviews (user_id, week_start, wins, improvements, notes) VALUES (?, ?, ?, ?, ?) RETURNING id, user_id, week_start, wins, improvements, notes, created_at"
    )
    .bind(data.user_id.unwrap_or(1))
    .bind(&data.week_start)
    .bind(&data.wins)
    .bind(&data.improvements)
    .bind(&data.notes)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn get_weekly_reviews(state: State<'_, DbState>) -> Result<Vec<WeeklyReview>, ApiError> {
    let pool = &state.0;
    let rows = sqlx::query_as::<_, WeeklyReview>("SELECT * FROM weekly_reviews ORDER BY week_start DESC")
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)?;
    Ok(rows)
}
