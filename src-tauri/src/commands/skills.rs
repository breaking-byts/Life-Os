use tauri::State;

use crate::{DbState, error::ApiError, models::skill::Skill};

#[derive(Debug, serde::Deserialize)]
pub struct SkillInput {
    pub user_id: Option<i64>,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub target_weekly_hours: Option<f64>,
}

#[tauri::command]
pub async fn create_skill(state: State<'_, DbState>, data: SkillInput) -> Result<Skill, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Skill>(
        "INSERT INTO skills (user_id, name, category, description, target_weekly_hours) VALUES (?, ?, ?, ?, ?) RETURNING id, user_id, name, category, description, target_weekly_hours, current_level, total_hours, created_at"
    )
    .bind(data.user_id.unwrap_or(1))
    .bind(&data.name)
    .bind(&data.category)
    .bind(&data.description)
    .bind(data.target_weekly_hours)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn get_skills(state: State<'_, DbState>) -> Result<Vec<Skill>, ApiError> {
    let pool = &state.0;
    let rows = sqlx::query_as::<_, Skill>("SELECT * FROM skills ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)?;
    Ok(rows)
}

#[tauri::command]
pub async fn update_skill(state: State<'_, DbState>, id: i64, data: SkillInput) -> Result<Skill, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Skill>(
        "UPDATE skills SET name = COALESCE(?, name), category = COALESCE(?, category), description = COALESCE(?, description), target_weekly_hours = COALESCE(?, target_weekly_hours) WHERE id = ? RETURNING id, user_id, name, category, description, target_weekly_hours, current_level, total_hours, created_at"
    )
    .bind(&data.name)
    .bind(&data.category)
    .bind(&data.description)
    .bind(data.target_weekly_hours)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn delete_skill(state: State<'_, DbState>, id: i64) -> Result<bool, ApiError> {
    let pool = &state.0;
    let result = sqlx::query("DELETE FROM skills WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ApiError::from)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Skill not found"));
    }

    Ok(true)
}
