use tauri::State;

use crate::{DbState, models::course::Course};

/// Maximum allowed length for string fields
const MAX_NAME_LENGTH: usize = 255;
const MAX_CODE_LENGTH: usize = 50;

// ============================================================================
// COURSE ANALYTICS STRUCTS
// ============================================================================

#[derive(Debug, serde::Serialize)]
pub struct CourseWithProgress {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub code: Option<String>,
    pub color: Option<String>,
    pub credit_hours: Option<i64>,
    pub target_weekly_hours: Option<f64>,
    pub is_active: Option<i64>,
    pub created_at: Option<String>,
    pub current_grade: Option<f64>,
    pub target_grade: Option<f64>,
    // Progress fields
    pub hours_this_week: f64,
    pub weekly_percent: f64,
    pub total_hours: f64,
    pub upcoming_assignments: i64,
    pub overdue_assignments: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct CourseAnalytics {
    pub course_id: i64,
    pub hours_this_week: f64,
    pub target_this_week: f64,
    pub weekly_percent: f64,
    pub total_hours: f64,
    pub sessions_count: i64,
    pub avg_session_duration: f64,
    pub weekly_history: Vec<WeeklyHours>,
}

#[derive(Debug, serde::Serialize)]
pub struct WeeklyHours {
    pub week_start: String,
    pub hours: f64,
}

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
        "INSERT INTO courses (user_id, name, code, color, credit_hours, target_weekly_hours, is_active, current_grade, target_grade) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) 
         RETURNING id, user_id, name, code, color, credit_hours, target_weekly_hours, is_active, created_at, current_grade, target_grade"
    )
    .bind(data.user_id.unwrap_or(1))
    .bind(&name)
    .bind(&data.code)
    .bind(data.color.unwrap_or_else(|| "#3b82f6".to_string()))
    .bind(data.credit_hours.unwrap_or(3))
    .bind(data.target_weekly_hours.unwrap_or(6.0))
    .bind(data.is_active.unwrap_or(1))
    .bind(data.current_grade)
    .bind(data.target_grade.unwrap_or(90.0))
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
        "UPDATE courses SET name = COALESCE(?, name), code = COALESCE(?, code), color = COALESCE(?, color), credit_hours = COALESCE(?, credit_hours), target_weekly_hours = COALESCE(?, target_weekly_hours), is_active = COALESCE(?, is_active), current_grade = COALESCE(?, current_grade), target_grade = COALESCE(?, target_grade) WHERE id = ? RETURNING id, user_id, name, code, color, credit_hours, target_weekly_hours, is_active, created_at, current_grade, target_grade"
    )
    .bind(&data.name)
    .bind(&data.code)
    .bind(&data.color)
    .bind(data.credit_hours)
    .bind(data.target_weekly_hours)
    .bind(data.is_active)
    .bind(data.current_grade)
    .bind(data.target_grade)
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
    #[serde(default)]
    pub current_grade: Option<f64>,
    #[serde(default)]
    pub target_grade: Option<f64>,
}

// ============================================================================
// COURSE ANALYTICS COMMANDS
// ============================================================================

#[tauri::command]
pub async fn get_courses_with_progress(state: State<'_, DbState>) -> Result<Vec<CourseWithProgress>, String> {
    let pool = &state.0;
    
    let courses = sqlx::query_as::<_, Course>("SELECT * FROM courses ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch courses: {}", e);
            "Failed to fetch courses".to_string()
        })?;
    
    let mut result: Vec<CourseWithProgress> = Vec::new();
    
    for course in courses {
        // Get hours this week
        let hours_this_week: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(duration_minutes), 0) / 60.0
            FROM sessions
            WHERE session_type = 'study'
              AND reference_type = 'course'
              AND reference_id = ?
              AND started_at >= date('now', 'weekday 0', '-7 days')
            "#
        )
        .bind(course.id)
        .fetch_one(pool)
        .await
        .unwrap_or(0.0);
        
        // Get total hours
        let total_hours: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(duration_minutes), 0) / 60.0
            FROM sessions
            WHERE session_type = 'study'
              AND reference_type = 'course'
              AND reference_id = ?
            "#
        )
        .bind(course.id)
        .fetch_one(pool)
        .await
        .unwrap_or(0.0);
        
        // Get upcoming assignments (due in next 7 days)
        let upcoming_assignments: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM assignments
            WHERE course_id = ?
              AND is_completed = 0
              AND due_date IS NOT NULL
              AND due_date >= datetime('now')
              AND due_date <= datetime('now', '+7 days')
            "#
        )
        .bind(course.id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        
        // Get overdue assignments
        let overdue_assignments: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM assignments
            WHERE course_id = ?
              AND is_completed = 0
              AND due_date IS NOT NULL
              AND due_date < datetime('now')
            "#
        )
        .bind(course.id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        
        let target = course.target_weekly_hours.unwrap_or(6.0);
        let weekly_percent = if target > 0.0 {
            (hours_this_week / target * 100.0).min(100.0)
        } else {
            0.0
        };
        
        result.push(CourseWithProgress {
            id: course.id,
            user_id: course.user_id,
            name: course.name,
            code: course.code,
            color: course.color,
            credit_hours: course.credit_hours,
            target_weekly_hours: course.target_weekly_hours,
            is_active: course.is_active,
            created_at: course.created_at,
            current_grade: course.current_grade,
            target_grade: course.target_grade,
            hours_this_week,
            weekly_percent,
            total_hours,
            upcoming_assignments,
            overdue_assignments,
        });
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn get_course_analytics(state: State<'_, DbState>, course_id: i64) -> Result<CourseAnalytics, String> {
    let pool = &state.0;
    
    // Get course target
    let target: f64 = sqlx::query_scalar(
        "SELECT COALESCE(target_weekly_hours, 6.0) FROM courses WHERE id = ?"
    )
    .bind(course_id)
    .fetch_one(pool)
    .await
    .unwrap_or(6.0);
    
    // Get hours this week
    let hours_this_week: f64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(duration_minutes), 0) / 60.0
        FROM sessions
        WHERE session_type = 'study'
          AND reference_type = 'course'
          AND reference_id = ?
          AND started_at >= date('now', 'weekday 0', '-7 days')
        "#
    )
    .bind(course_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0.0);
    
    // Get total hours
    let total_hours: f64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(duration_minutes), 0) / 60.0
        FROM sessions
        WHERE session_type = 'study'
          AND reference_type = 'course'
          AND reference_id = ?
        "#
    )
    .bind(course_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0.0);
    
    // Get session count
    let sessions_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE session_type = 'study'
          AND reference_type = 'course'
          AND reference_id = ?
        "#
    )
    .bind(course_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    
    // Get average session duration
    let avg_session_duration: f64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(AVG(duration_minutes), 0)
        FROM sessions
        WHERE session_type = 'study'
          AND reference_type = 'course'
          AND reference_id = ?
          AND duration_minutes IS NOT NULL
        "#
    )
    .bind(course_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0.0);
    
    // Get weekly history (last 8 weeks)
    let weekly_rows = sqlx::query_as::<_, (String, f64)>(
        r#"
        WITH weeks AS (
            SELECT date('now', 'weekday 0', (-7 * n) || ' days') as week_start
            FROM (SELECT 0 as n UNION SELECT 1 UNION SELECT 2 UNION SELECT 3 
                  UNION SELECT 4 UNION SELECT 5 UNION SELECT 6 UNION SELECT 7)
        )
        SELECT 
            w.week_start,
            COALESCE(SUM(s.duration_minutes), 0) / 60.0 as hours
        FROM weeks w
        LEFT JOIN sessions s ON date(s.started_at) >= w.week_start 
            AND date(s.started_at) < date(w.week_start, '+7 days')
            AND s.session_type = 'study'
            AND s.reference_type = 'course'
            AND s.reference_id = ?
        GROUP BY w.week_start
        ORDER BY w.week_start
        "#
    )
    .bind(course_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    let weekly_history: Vec<WeeklyHours> = weekly_rows
        .into_iter()
        .map(|(week_start, hours)| WeeklyHours { week_start, hours })
        .collect();
    
    let weekly_percent = if target > 0.0 {
        (hours_this_week / target * 100.0).min(100.0)
    } else {
        0.0
    };
    
    Ok(CourseAnalytics {
        course_id,
        hours_this_week,
        target_this_week: target,
        weekly_percent,
        total_hours,
        sessions_count,
        avg_session_duration,
        weekly_history,
    })
}

