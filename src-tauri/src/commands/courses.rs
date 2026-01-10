use tauri::State;

use crate::{DbState, models::course::Course};

/// Maximum allowed length for string fields
const MAX_NAME_LENGTH: usize = 255;
const MAX_CODE_LENGTH: usize = 50;

#[tauri::command]
pub async fn create_course(state: State<'_, DbState>, data: CourseInput) -> Result<Course, String> {
    let pool = &state.0;
    
    let name = data.name.unwrap_or_else(|| "Untitled Course".to_string());
    
    // Input validation
    if name.len() > MAX_NAME_LENGTH {
        return Err(format!("Course name too long (max {} characters)", MAX_NAME_LENGTH));
    }
    if let Some(ref code) = data.code {
        if code.len() > MAX_CODE_LENGTH {
            return Err(format!("Course code too long (max {} characters)", MAX_CODE_LENGTH));
        }
    }
    if let Some(hours) = data.credit_hours {
        if hours < 0 || hours > 12 {
            return Err("Credit hours must be between 0 and 12".to_string());
        }
    }
    if let Some(target) = data.target_weekly_hours {
        if target < 0.0 || target > 168.0 {
            return Err("Target weekly hours must be between 0 and 168".to_string());
        }
    }
    
    log::debug!("Creating course: {}", name);
    
    let rec = sqlx::query_as::<_, Course>(
        "INSERT INTO courses (user_id, name, code, color, credit_hours, target_weekly_hours, is_active) 
         VALUES (?, ?, ?, ?, ?, ?, ?) 
         RETURNING id, user_id, name, code, color, credit_hours, target_weekly_hours, is_active, created_at"
    )
    .bind(data.user_id.unwrap_or(1))
    .bind(&name)
    .bind(&data.code)
    .bind(data.color.unwrap_or_else(|| "#3b82f6".to_string()))
    .bind(data.credit_hours.unwrap_or(3))
    .bind(data.target_weekly_hours.unwrap_or(6.0))
    .bind(data.is_active.unwrap_or(1))
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to create course: {}", e);
        "Failed to create course".to_string()
    })?;
    
    log::info!("Course created successfully: id={}", rec.id);
    Ok(rec)
}

#[tauri::command]
pub async fn get_courses(state: State<'_, DbState>) -> Result<Vec<Course>, String> {
    let pool = &state.0;
    let rows = sqlx::query_as::<_, Course>("SELECT * FROM courses ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch courses: {}", e);
            "Failed to fetch courses".to_string()
        })?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_course(state: State<'_, DbState>, id: i64) -> Result<Course, String> {
    let pool = &state.0;
    let row = sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch course {}: {}", id, e);
            "Course not found".to_string()
        })?;
    Ok(row)
}

#[tauri::command]
pub async fn update_course(state: State<'_, DbState>, id: i64, data: CourseInput) -> Result<Course, String> {
    let pool = &state.0;
    
    // Input validation
    if let Some(ref name) = data.name {
        if name.len() > MAX_NAME_LENGTH {
            return Err(format!("Course name too long (max {} characters)", MAX_NAME_LENGTH));
        }
    }
    if let Some(ref code) = data.code {
        if code.len() > MAX_CODE_LENGTH {
            return Err(format!("Course code too long (max {} characters)", MAX_CODE_LENGTH));
        }
    }
    if let Some(hours) = data.credit_hours {
        if hours < 0 || hours > 12 {
            return Err("Credit hours must be between 0 and 12".to_string());
        }
    }
    if let Some(target) = data.target_weekly_hours {
        if target < 0.0 || target > 168.0 {
            return Err("Target weekly hours must be between 0 and 168".to_string());
        }
    }
    
    let rec = sqlx::query_as::<_, Course>(
        "UPDATE courses SET name = COALESCE(?, name), code = COALESCE(?, code), color = COALESCE(?, color), credit_hours = COALESCE(?, credit_hours), target_weekly_hours = COALESCE(?, target_weekly_hours), is_active = COALESCE(?, is_active) WHERE id = ? RETURNING id, user_id, name, code, color, credit_hours, target_weekly_hours, is_active, created_at"
    )
    .bind(&data.name)
    .bind(&data.code)
    .bind(&data.color)
    .bind(data.credit_hours)
    .bind(data.target_weekly_hours)
    .bind(data.is_active)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to update course {}: {}", id, e);
        "Failed to update course".to_string()
    })?;
    
    log::info!("Course updated: id={}", id);
    Ok(rec)
}

#[tauri::command]
pub async fn delete_course(state: State<'_, DbState>, id: i64) -> Result<bool, String> {
    let pool = &state.0;
    sqlx::query("DELETE FROM courses WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete course {}: {}", id, e);
            "Failed to delete course".to_string()
        })?;
    log::info!("Course deleted: id={}", id);
    Ok(true)
}

#[derive(Debug, serde::Deserialize)]
pub struct CourseInput {
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub credit_hours: Option<i64>,
    #[serde(default)]
    pub target_weekly_hours: Option<f64>,
    #[serde(default)]
    pub is_active: Option<i64>,
}

