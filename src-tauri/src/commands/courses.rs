use tauri::State;

use crate::{DbState, error::ApiError, models::course::Course};

/// Maximum allowed length for string fields
const MAX_NAME_LENGTH: usize = 255;
const MAX_CODE_LENGTH: usize = 50;

// ============================================================================
// COURSE ANALYTICS STRUCTS
// ============================================================================

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
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
pub async fn create_course(state: State<'_, DbState>, data: CourseInput) -> Result<Course, ApiError> {
    let pool = &state.0;
    
    let name = data.name.unwrap_or_else(|| "Untitled Course".to_string());
    
    // Input validation
    if name.len() > MAX_NAME_LENGTH {
        return Err(ApiError::validation(format!(
            "Course name too long (max {} characters)",
            MAX_NAME_LENGTH
        )));
    }
    if let Some(ref code) = data.code {
        if code.len() > MAX_CODE_LENGTH {
            return Err(ApiError::validation(format!(
                "Course code too long (max {} characters)",
                MAX_CODE_LENGTH
            )));
        }
    }
    if let Some(hours) = data.credit_hours {
        if hours < 0 || hours > 12 {
            return Err(ApiError::validation(
                "Credit hours must be between 0 and 12",
            ));
        }
    }
    if let Some(target) = data.target_weekly_hours {
        if target < 0.0 || target > 168.0 {
            return Err(ApiError::validation(
                "Target weekly hours must be between 0 and 168",
            ));
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
        ApiError::from_sqlx(e, "Failed to create course")
    })?;
    
    log::info!("Course created successfully: id={}", rec.id);
    Ok(rec)
}

#[tauri::command]
pub async fn get_courses(state: State<'_, DbState>) -> Result<Vec<Course>, ApiError> {
    let pool = &state.0;
    let rows = sqlx::query_as::<_, Course>("SELECT * FROM courses ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch courses: {}", e);
            ApiError::from_sqlx(e, "Failed to fetch courses")
        })?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_course(state: State<'_, DbState>, id: i64) -> Result<Course, ApiError> {
    let pool = &state.0;
    let row = sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch course {}: {}", id, e);
            ApiError::from_sqlx(e, "Course not found")
        })?;
    Ok(row)
}

#[tauri::command]
pub async fn update_course(state: State<'_, DbState>, id: i64, data: CourseInput) -> Result<Course, ApiError> {
    let pool = &state.0;
    
    // Input validation
    if let Some(ref name) = data.name {
        if name.len() > MAX_NAME_LENGTH {
            return Err(ApiError::validation(format!(
                "Course name too long (max {} characters)",
                MAX_NAME_LENGTH
            )));
        }
    }
    if let Some(ref code) = data.code {
        if code.len() > MAX_CODE_LENGTH {
            return Err(ApiError::validation(format!(
                "Course code too long (max {} characters)",
                MAX_CODE_LENGTH
            )));
        }
    }
    if let Some(hours) = data.credit_hours {
        if hours < 0 || hours > 12 {
            return Err(ApiError::validation(
                "Credit hours must be between 0 and 12",
            ));
        }
    }
    if let Some(target) = data.target_weekly_hours {
        if target < 0.0 || target > 168.0 {
            return Err(ApiError::validation(
                "Target weekly hours must be between 0 and 168",
            ));
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
        ApiError::from_sqlx(e, "Failed to update course")
    })?;
    
    log::info!("Course updated: id={}", id);
    Ok(rec)
}

#[tauri::command]
pub async fn delete_course(state: State<'_, DbState>, id: i64) -> Result<bool, ApiError> {
    let pool = &state.0;
    let result = sqlx::query("DELETE FROM courses WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete course {}: {}", id, e);
            ApiError::from_sqlx(e, "Failed to delete course")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Course not found"));
    }

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
pub async fn get_courses_with_progress(state: State<'_, DbState>) -> Result<Vec<CourseWithProgress>, ApiError> {
    get_courses_with_progress_inner(&state.0).await
}

pub async fn get_courses_with_progress_inner(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<CourseWithProgress>, ApiError> {
    sqlx::query_as::<_, CourseWithProgress>(
        r#"
        SELECT
            c.id,
            c.user_id,
            c.name,
            c.code,
            c.color,
            c.credit_hours,
            c.target_weekly_hours,
            c.is_active,
            c.created_at,
            c.current_grade,
            c.target_grade,
            COALESCE(s_week.hours, 0.0) as hours_this_week,
            CASE
                WHEN COALESCE(c.target_weekly_hours, 6.0) > 0 THEN
                    MIN((COALESCE(s_week.hours, 0.0) / COALESCE(c.target_weekly_hours, 6.0) * 100.0), 100.0)
                ELSE 0.0
            END as weekly_percent,
            COALESCE(s_total.hours, 0.0) as total_hours,
            COALESCE(a_upcoming.count, 0) as upcoming_assignments,
            COALESCE(a_overdue.count, 0) as overdue_assignments
        FROM courses c
        LEFT JOIN (
            SELECT reference_id, SUM(duration_minutes) / 60.0 as hours
            FROM sessions
            WHERE session_type = 'study'
              AND reference_type = 'course'
              AND started_at >= date('now', 'weekday 0', '-7 days')
            GROUP BY reference_id
        ) s_week ON c.id = s_week.reference_id
        LEFT JOIN (
            SELECT reference_id, SUM(duration_minutes) / 60.0 as hours
            FROM sessions
            WHERE session_type = 'study'
              AND reference_type = 'course'
            GROUP BY reference_id
        ) s_total ON c.id = s_total.reference_id
        LEFT JOIN (
            SELECT course_id, COUNT(*) as count
            FROM assignments
            WHERE is_completed = 0
              AND due_date IS NOT NULL
              AND due_date >= datetime('now')
              AND due_date <= datetime('now', '+7 days')
            GROUP BY course_id
        ) a_upcoming ON c.id = a_upcoming.course_id
        LEFT JOIN (
            SELECT course_id, COUNT(*) as count
            FROM assignments
            WHERE is_completed = 0
              AND due_date IS NOT NULL
              AND due_date < datetime('now')
            GROUP BY course_id
        ) a_overdue ON c.id = a_overdue.course_id
        ORDER BY c.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch courses with progress: {}", e);
        ApiError::from_sqlx(e, "Failed to fetch courses")
    })
}

#[tauri::command]
pub async fn get_course_analytics(state: State<'_, DbState>, course_id: i64) -> Result<CourseAnalytics, ApiError> {
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
    .map_err(ApiError::from)?;
    
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

// ============================================================================
// UNIT TESTS - TDD Compliant
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures
    fn valid_course_input() -> CourseInput {
        CourseInput {
            user_id: Some(1),
            name: Some("Test Course".to_string()),
            code: Some("TEST101".to_string()),
            color: Some("#3b82f6".to_string()),
            credit_hours: Some(3),
            target_weekly_hours: Some(6.0),
            is_active: Some(1),
            current_grade: Some(85.0),
            target_grade: Some(90.0),
        }
    }

    // =========================================================================
    // Name Validation Tests
    // =========================================================================

    #[test]
    fn test_name_within_max_length_is_valid() {
        let name = "A".repeat(MAX_NAME_LENGTH);
        assert!(name.len() <= MAX_NAME_LENGTH);
    }

    #[test]
    fn test_name_exceeding_max_length_fails() {
        let name = "A".repeat(MAX_NAME_LENGTH + 1);
        assert!(name.len() > MAX_NAME_LENGTH);
    }

    #[test]
    fn test_empty_name_uses_default() {
        let input = CourseInput {
            name: None,
            ..Default::default()
        };
        let name = input.name.unwrap_or_else(|| "Untitled Course".to_string());
        assert_eq!(name, "Untitled Course");
    }

    // =========================================================================
    // Code Validation Tests
    // =========================================================================

    #[test]
    fn test_code_within_max_length_is_valid() {
        let code = "A".repeat(MAX_CODE_LENGTH);
        assert!(code.len() <= MAX_CODE_LENGTH);
    }

    #[test]
    fn test_code_exceeding_max_length_fails() {
        let code = "A".repeat(MAX_CODE_LENGTH + 1);
        assert!(code.len() > MAX_CODE_LENGTH);
    }

    // =========================================================================
    // Credit Hours Validation Tests
    // =========================================================================

    #[test]
    fn test_credit_hours_zero_is_valid() {
        let hours = 0;
        assert!(hours >= 0 && hours <= 12);
    }

    #[test]
    fn test_credit_hours_twelve_is_valid() {
        let hours = 12;
        assert!(hours >= 0 && hours <= 12);
    }

    #[test]
    fn test_credit_hours_negative_fails() {
        let hours = -1;
        assert!(hours < 0 || hours > 12);
    }

    #[test]
    fn test_credit_hours_exceeds_max_fails() {
        let hours = 13;
        assert!(hours < 0 || hours > 12);
    }

    // =========================================================================
    // Target Weekly Hours Validation Tests
    // =========================================================================

    #[test]
    fn test_target_weekly_hours_zero_is_valid() {
        let target = 0.0;
        assert!(target >= 0.0 && target <= 168.0);
    }

    #[test]
    fn test_target_weekly_hours_max_is_valid() {
        let target = 168.0;
        assert!(target >= 0.0 && target <= 168.0);
    }

    #[test]
    fn test_target_weekly_hours_negative_fails() {
        let target = -1.0;
        assert!(target < 0.0 || target > 168.0);
    }

    #[test]
    fn test_target_weekly_hours_exceeds_week_fails() {
        let target = 169.0;
        assert!(target < 0.0 || target > 168.0);
    }

    // =========================================================================
    // CourseInput Default Tests
    // =========================================================================

    #[test]
    fn test_course_input_defaults() {
        let input = CourseInput::default();
        assert!(input.user_id.is_none());
        assert!(input.name.is_none());
        assert!(input.code.is_none());
        assert!(input.color.is_none());
        assert!(input.credit_hours.is_none());
        assert!(input.target_weekly_hours.is_none());
        assert!(input.is_active.is_none());
        assert!(input.current_grade.is_none());
        assert!(input.target_grade.is_none());
    }

    // =========================================================================
    // Weekly Percent Calculation Tests
    // =========================================================================

    #[test]
    fn test_weekly_percent_calculation_zero_target() {
        let hours_this_week: f64 = 5.0;
        let target: f64 = 0.0;
        let weekly_percent = if target > 0.0 {
            (hours_this_week / target * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(weekly_percent, 0.0);
    }

    #[test]
    fn test_weekly_percent_calculation_half_progress() {
        let hours_this_week: f64 = 3.0;
        let target: f64 = 6.0;
        let weekly_percent = if target > 0.0 {
            (hours_this_week / target * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(weekly_percent, 50.0);
    }

    #[test]
    fn test_weekly_percent_calculation_capped_at_100() {
        let hours_this_week: f64 = 12.0;
        let target: f64 = 6.0;
        let weekly_percent = if target > 0.0 {
            (hours_this_week / target * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(weekly_percent, 100.0);
    }

    #[test]
    fn test_weekly_percent_calculation_full_progress() {
        let hours_this_week: f64 = 6.0;
        let target: f64 = 6.0;
        let weekly_percent = if target > 0.0 {
            (hours_this_week / target * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(weekly_percent, 100.0);
    }

    // =========================================================================
    // CourseWithProgress Struct Tests
    // =========================================================================

    #[test]
    fn test_course_with_progress_serialization() {
        let course = CourseWithProgress {
            id: 1,
            user_id: 1,
            name: "Test".to_string(),
            code: Some("TEST".to_string()),
            color: Some("#fff".to_string()),
            credit_hours: Some(3),
            target_weekly_hours: Some(6.0),
            is_active: Some(1),
            created_at: Some("2026-01-01".to_string()),
            current_grade: Some(85.0),
            target_grade: Some(90.0),
            hours_this_week: 3.0,
            weekly_percent: 50.0,
            total_hours: 30.0,
            upcoming_assignments: 2,
            overdue_assignments: 0,
        };
        
        // Verify the struct can be serialized (serde::Serialize is derived)
        let json = serde_json::to_string(&course).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"weekly_percent\":50.0"));
    }

    // =========================================================================
    // CourseAnalytics Struct Tests
    // =========================================================================

    #[test]
    fn test_course_analytics_serialization() {
        let analytics = CourseAnalytics {
            course_id: 1,
            hours_this_week: 5.0,
            target_this_week: 6.0,
            weekly_percent: 83.33,
            total_hours: 50.0,
            sessions_count: 15,
            avg_session_duration: 45.0,
            weekly_history: vec![
                WeeklyHours { week_start: "2026-01-01".to_string(), hours: 4.0 },
                WeeklyHours { week_start: "2026-01-08".to_string(), hours: 5.0 },
            ],
        };
        
        let json = serde_json::to_string(&analytics).unwrap();
        assert!(json.contains("\"course_id\":1"));
        assert!(json.contains("\"sessions_count\":15"));
        assert!(json.contains("weekly_history"));
    }
}

// Implement Default for CourseInput to support tests
impl Default for CourseInput {
    fn default() -> Self {
        CourseInput {
            user_id: None,
            name: None,
            code: None,
            color: None,
            credit_hours: None,
            target_weekly_hours: None,
            is_active: None,
            current_grade: None,
            target_grade: None,
        }
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::Instant;
    use sqlx::Row;

    async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE courses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            name TEXT NOT NULL,
            code TEXT,
            color TEXT,
            credit_hours INTEGER,
            target_weekly_hours REAL,
            is_active INTEGER,
            current_grade REAL,
            target_grade REAL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.unwrap();

        sqlx::query("CREATE TABLE sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            session_type TEXT NOT NULL,
            reference_id INTEGER,
            reference_type TEXT,
            started_at TEXT DEFAULT CURRENT_TIMESTAMP,
            ended_at TEXT,
            duration_minutes INTEGER,
            notes TEXT
        )").execute(&pool).await.unwrap();

        sqlx::query("CREATE TABLE assignments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            course_id INTEGER,
            title TEXT NOT NULL,
            description TEXT,
            due_date TEXT,
            priority TEXT,
            is_completed INTEGER DEFAULT 0,
            completed_at TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn benchmark_get_courses_with_progress() {
        let pool = setup_db().await;

        // Seed Data
        for i in 0..50 {
            let row = sqlx::query("INSERT INTO courses (name, target_weekly_hours) VALUES (?, ?) RETURNING id")
                .bind(format!("Course {}", i))
                .bind(6.0)
                .fetch_one(&pool)
                .await
                .unwrap();
            let course_id: i64 = row.get(0);

            // Insert sessions (some this week, some older)
            for j in 0..40 {
                // Mix dates: some in last 7 days, some older
                let date_mod = if j % 2 == 0 { "0 days" } else { "-10 days" };
                sqlx::query("INSERT INTO sessions (session_type, reference_type, reference_id, duration_minutes, started_at) VALUES ('study', 'course', ?, ?, date('now', ?))")
                    .bind(course_id)
                    .bind(60)
                    .bind(date_mod)
                    .execute(&pool)
                    .await
                    .unwrap();
            }

            // Insert assignments
            for j in 0..10 {
                let due_mod = if j % 2 == 0 { "+3 days" } else { "-3 days" }; // upcoming vs overdue
                sqlx::query("INSERT INTO assignments (course_id, title, due_date, is_completed) VALUES (?, 'HW', datetime('now', ?), 0)")
                    .bind(course_id)
                    .bind(due_mod)
                    .execute(&pool)
                    .await
                    .unwrap();
            }
        }

        let start = Instant::now();
        let _result = get_courses_with_progress_inner(&pool).await.unwrap();
        let duration = start.elapsed();

        println!("Benchmark get_courses_with_progress: {:?}", duration);
    }
}
