use tauri::State;

use crate::DbState;

#[derive(Debug, serde::Deserialize)]
pub struct PracticeInput {
    pub skill_id: i64,
    pub duration_minutes: i64,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct PracticeLog {
    pub id: i64,
    pub skill_id: i64,
    pub duration_minutes: i64,
    pub notes: Option<String>,
    pub logged_at: Option<String>,
}

#[tauri::command]
pub async fn log_practice(state: State<'_, DbState>, data: PracticeInput) -> Result<PracticeLog, String> {
    let pool = &state.0;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let rec = sqlx::query_as::<_, PracticeLog>(
        "INSERT INTO practice_logs (skill_id, duration_minutes, notes) VALUES (?, ?, ?) RETURNING id, skill_id, duration_minutes, notes, logged_at"
    )
    .bind(data.skill_id)
    .bind(data.duration_minutes)
    .bind(&data.notes)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE skills SET total_hours = COALESCE(total_hours, 0) + (? / 60.0) WHERE id = ?")
        .bind(data.duration_minutes)
        .bind(data.skill_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(rec)
}

#[tauri::command]
pub async fn get_practice_logs(state: State<'_, DbState>, skill_id: Option<i64>) -> Result<Vec<PracticeLog>, String> {
    let pool = &state.0;
    let rows = match skill_id {
        Some(id) => sqlx::query_as::<_, PracticeLog>("SELECT * FROM practice_logs WHERE skill_id = ? ORDER BY logged_at DESC")
            .bind(id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?,
        None => sqlx::query_as::<_, PracticeLog>("SELECT * FROM practice_logs ORDER BY logged_at DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?,
    };
    Ok(rows)
}

