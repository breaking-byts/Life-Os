use tauri::State;

use crate::{DbState, models::assignment::Assignment};

#[derive(Debug, serde::Deserialize)]
pub struct AssignmentInput {
    pub course_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<String>,
}

#[tauri::command]
pub async fn create_assignment(state: State<'_, DbState>, data: AssignmentInput) -> Result<Assignment, String> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Assignment>(
        "INSERT INTO assignments (course_id, title, description, due_date, priority) VALUES (?, ?, ?, ?, ?) RETURNING id, course_id, title, description, due_date, priority, is_completed, completed_at, created_at"
    )
    .bind(data.course_id)
    .bind(&data.title)
    .bind(&data.description)
    .bind(&data.due_date)
    .bind(&data.priority)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}

#[tauri::command]
pub async fn get_assignments(state: State<'_, DbState>, course_id: Option<i64>) -> Result<Vec<Assignment>, String> {
    let pool = &state.0;
    let rows = if let Some(course_id) = course_id {
        sqlx::query_as::<_, Assignment>("SELECT * FROM assignments WHERE course_id = ? ORDER BY due_date IS NULL, due_date")
            .bind(course_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as::<_, Assignment>("SELECT * FROM assignments ORDER BY due_date IS NULL, due_date")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
    };
    Ok(rows)
}

#[tauri::command]
pub async fn update_assignment(state: State<'_, DbState>, id: i64, data: AssignmentInput) -> Result<Assignment, String> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Assignment>(
        "UPDATE assignments SET course_id = COALESCE(?, course_id), title = COALESCE(?, title), description = COALESCE(?, description), due_date = COALESCE(?, due_date), priority = COALESCE(?, priority) WHERE id = ? RETURNING id, course_id, title, description, due_date, priority, is_completed, completed_at, created_at"
    )
    .bind(Some(data.course_id))
    .bind(Some(&data.title))
    .bind(&data.description)
    .bind(&data.due_date)
    .bind(&data.priority)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}

#[tauri::command]
pub async fn delete_assignment(state: State<'_, DbState>, id: i64) -> Result<bool, String> {
    let pool = &state.0;
    let result = sqlx::query("DELETE FROM assignments WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("Assignment not found".to_string());
    }

    Ok(true)
}

#[tauri::command]
pub async fn toggle_assignment(state: State<'_, DbState>, id: i64) -> Result<Assignment, String> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, Assignment>(
        "UPDATE assignments SET is_completed = CASE WHEN is_completed = 1 THEN 0 ELSE 1 END, completed_at = CASE WHEN is_completed = 1 THEN NULL ELSE CURRENT_TIMESTAMP END WHERE id = ? RETURNING id, course_id, title, description, due_date, priority, is_completed, completed_at, created_at"
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}

