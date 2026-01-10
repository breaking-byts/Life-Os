use tauri::State;

use crate::{DbState, models::checkin::CheckIn};

#[derive(Debug, serde::Deserialize)]
pub struct CheckInInput {
    pub user_id: Option<i64>,
    pub mood: Option<i64>,
    pub energy: Option<i64>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn create_checkin(state: State<'_, DbState>, data: CheckInInput) -> Result<CheckIn, String> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, CheckIn>(
        "INSERT INTO check_ins (user_id, mood, energy, notes) VALUES (?, ?, ?, ?) RETURNING id, user_id, mood, energy, notes, checked_in_at"
    )
    .bind(data.user_id.unwrap_or(1))
    .bind(data.mood)
    .bind(data.energy)
    .bind(&data.notes)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}

#[tauri::command]
pub async fn get_today_checkin(state: State<'_, DbState>) -> Result<Option<CheckIn>, String> {
    let pool = &state.0;
    let row = sqlx::query_as::<_, CheckIn>(
        "SELECT * FROM check_ins WHERE date(checked_in_at) = date('now') ORDER BY checked_in_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row)
}

#[tauri::command]
pub async fn get_checkins(state: State<'_, DbState>) -> Result<Vec<CheckIn>, String> {
    let pool = &state.0;
    let rows = sqlx::query_as::<_, CheckIn>("SELECT * FROM check_ins ORDER BY checked_in_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

