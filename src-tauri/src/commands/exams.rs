use tauri::State;

use crate::{DbState, error::ApiError, models::exam::Exam};

#[derive(Debug, serde::Deserialize)]
pub struct ExamInput {
    #[serde(default)]
    pub course_id: Option<i64>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub exam_date: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub duration_minutes: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub grade: Option<f64>,
    #[serde(default)]
    pub weight: Option<f64>,
}

#[tauri::command]
pub async fn create_exam(state: State<'_, DbState>, data: ExamInput) -> Result<Exam, ApiError> {
    let pool = &state.0;
    
    let course_id = data
        .course_id
        .ok_or_else(|| ApiError::validation("course_id is required"))?;
    let title = data.title.unwrap_or_else(|| "Untitled Exam".to_string());
    
    let rec = sqlx::query_as::<_, Exam>(
        r#"
        INSERT INTO exams (course_id, title, exam_date, location, duration_minutes, notes, grade, weight)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id, course_id, title, exam_date, location, duration_minutes, notes, grade, weight, created_at
        "#
    )
    .bind(course_id)
    .bind(&title)
    .bind(&data.exam_date)
    .bind(&data.location)
    .bind(data.duration_minutes)
    .bind(&data.notes)
    .bind(data.grade)
    .bind(data.weight)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to create exam: {}", e);
        ApiError::from_sqlx(e, "Failed to create exam")
    })?;
    
    log::info!("Exam created: id={}", rec.id);
    Ok(rec)
}

#[tauri::command]
pub async fn get_exams(state: State<'_, DbState>, course_id: Option<i64>) -> Result<Vec<Exam>, ApiError> {
    let pool = &state.0;
    
    let exams = if let Some(cid) = course_id {
        sqlx::query_as::<_, Exam>(
            "SELECT * FROM exams WHERE course_id = ? ORDER BY exam_date ASC, created_at DESC"
        )
        .bind(cid)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Exam>(
            "SELECT * FROM exams ORDER BY exam_date ASC, created_at DESC"
        )
        .fetch_all(pool)
        .await
    }
    .map_err(|e| {
        log::error!("Failed to fetch exams: {}", e);
        ApiError::from_sqlx(e, "Failed to fetch exams")
    })?;
    
    Ok(exams)
}

#[tauri::command]
pub async fn get_exam(state: State<'_, DbState>, id: i64) -> Result<Exam, ApiError> {
    let pool = &state.0;
    
    let exam = sqlx::query_as::<_, Exam>("SELECT * FROM exams WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch exam {}: {}", id, e);
            ApiError::from_sqlx(e, "Exam not found")
        })?;
    
    Ok(exam)
}

#[tauri::command]
pub async fn update_exam(state: State<'_, DbState>, id: i64, data: ExamInput) -> Result<Exam, ApiError> {
    let pool = &state.0;
    
    let rec = sqlx::query_as::<_, Exam>(
        r#"
        UPDATE exams SET 
            title = COALESCE(?, title),
            exam_date = COALESCE(?, exam_date),
            location = COALESCE(?, location),
            duration_minutes = COALESCE(?, duration_minutes),
            notes = COALESCE(?, notes),
            grade = COALESCE(?, grade),
            weight = COALESCE(?, weight)
        WHERE id = ?
        RETURNING id, course_id, title, exam_date, location, duration_minutes, notes, grade, weight, created_at
        "#
    )
    .bind(&data.title)
    .bind(&data.exam_date)
    .bind(&data.location)
    .bind(data.duration_minutes)
    .bind(&data.notes)
    .bind(data.grade)
    .bind(data.weight)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to update exam {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to update exam")
    })?;
    
    log::info!("Exam updated: id={}", id);
    Ok(rec)
}

#[tauri::command]
pub async fn delete_exam(state: State<'_, DbState>, id: i64) -> Result<bool, ApiError> {
    let pool = &state.0;
    
    let result = sqlx::query("DELETE FROM exams WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete exam {}: {}", id, e);
            ApiError::from_sqlx(e, "Failed to delete exam")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Exam not found"));
    }

    log::info!("Exam deleted: id={}", id);
    Ok(true)
}

#[tauri::command]
pub async fn get_upcoming_exams(state: State<'_, DbState>, days: i64) -> Result<Vec<Exam>, ApiError> {
    let pool = &state.0;
    
    let exams = sqlx::query_as::<_, Exam>(
        r#"
        SELECT * FROM exams 
        WHERE exam_date IS NOT NULL 
          AND exam_date >= datetime('now')
          AND exam_date <= datetime('now', ? || ' days')
        ORDER BY exam_date ASC
        "#
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch upcoming exams: {}", e);
        ApiError::from_sqlx(e, "Failed to fetch upcoming exams")
    })?;
    
    Ok(exams)
}
