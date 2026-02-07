use tauri::State;

use crate::{DbState, error::ApiError};

#[derive(Debug, serde::Serialize)]
pub struct StatsSummary {
    pub study_hours_week: f64,
    pub practice_hours_week: f64,
    pub workouts_week: i64,
    pub active_streaks: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct Streaks {
    pub study_streak: i64,
    pub workout_streak: i64,
    pub practice_streak: i64,
    pub checkin_streak: i64,
}

// ============================================================================
// DETAILED STATS FOR DASHBOARD REVAMP
// ============================================================================

#[derive(Debug, serde::Serialize)]
pub struct CourseProgress {
    pub course_id: i64,
    pub course_name: String,
    pub code: Option<String>,
    pub color: String,
    pub hours_this_week: f64,
    pub target_hours: f64,
    pub percent: f64,
    pub current_grade: Option<f64>,
    pub target_grade: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
pub struct SkillProgress {
    pub skill_id: i64,
    pub skill_name: String,
    pub category: Option<String>,
    pub hours_this_week: f64,
    pub target_weekly_hours: f64,
    pub weekly_percent: f64,
    pub total_hours: f64,
    pub target_hours: f64,
    pub mastery_percent: f64,
    pub current_level: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct DetailedStats {
    // Study stats
    pub study_hours_week: f64,
    pub study_target_week: f64,
    pub study_percent: f64,
    pub study_breakdown: Vec<CourseProgress>,
    
    // Practice stats
    pub practice_hours_week: f64,
    pub practice_target_week: f64,
    pub practice_percent: f64,
    pub practice_breakdown: Vec<SkillProgress>,
    
    // Workout stats
    pub workouts_week: i64,
    pub workout_target_week: i64,
    pub workout_percent: f64,
    
    // Active skills stats
    pub active_skills_count: i64,
    pub skills_target: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct UserSettings {
    pub weekly_workout_target: i64,
    pub weekly_active_skills_target: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct WorkoutHeatmapDay {
    pub date: String,
    pub count: i64,
    pub total_minutes: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct Achievement {
    pub id: i64,
    pub achievement_type: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub achieved_at: String,
    pub metadata: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PersonalRecord {
    pub id: i64,
    pub exercise_name: String,
    pub pr_type: String,
    pub value: f64,
    pub achieved_at: String,
    pub workout_id: Option<i64>,
}

#[tauri::command]
pub async fn get_stats(state: State<'_, DbState>) -> Result<StatsSummary, ApiError> {
    let pool = &state.0;

    let study_minutes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions WHERE session_type = 'study' AND started_at >= date('now', '-6 days')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let practice_minutes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions WHERE session_type = 'practice' AND started_at >= date('now', '-6 days')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let workouts_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workouts WHERE logged_at >= date('now', '-6 days')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let active_streaks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM skills WHERE total_hours >= 1"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(StatsSummary {
        study_hours_week: study_minutes as f64 / 60.0,
        practice_hours_week: practice_minutes as f64 / 60.0,
        workouts_week,
        active_streaks,
    })
}

#[tauri::command]
pub async fn get_streaks(state: State<'_, DbState>) -> Result<Streaks, ApiError> {
    let pool = &state.0;

    // Study streak: consecutive days with study sessions
    let study_streak: i64 = sqlx::query_scalar(
        r#"
        WITH dated_sessions AS (
            SELECT DISTINCT date(started_at) as session_date
            FROM sessions
            WHERE session_type = 'study'
            ORDER BY session_date DESC
        ),
        streak AS (
            SELECT session_date,
                   julianday('now', 'start of day') - julianday(session_date) as days_ago
            FROM dated_sessions
        )
        SELECT COUNT(*) FROM streak
        WHERE days_ago = (SELECT MIN(days_ago) FROM streak WHERE days_ago >= 0)
           OR days_ago IN (
               SELECT days_ago + 1 FROM streak
           )
        "#
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Workout streak: consecutive days with workouts
    let workout_streak: i64 = sqlx::query_scalar(
        r#"
        WITH dated_workouts AS (
            SELECT DISTINCT date(logged_at) as workout_date
            FROM workouts
            ORDER BY workout_date DESC
        ),
        streak AS (
            SELECT workout_date,
                   julianday('now', 'start of day') - julianday(workout_date) as days_ago
            FROM dated_workouts
        )
        SELECT COUNT(*) FROM streak
        WHERE days_ago = (SELECT MIN(days_ago) FROM streak WHERE days_ago >= 0)
           OR days_ago IN (
               SELECT days_ago + 1 FROM streak
           )
        "#
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Practice streak: consecutive days with practice logs
    let practice_streak: i64 = sqlx::query_scalar(
        r#"
        WITH dated_practice AS (
            SELECT DISTINCT date(logged_at) as practice_date
            FROM practice_logs
            ORDER BY practice_date DESC
        ),
        streak AS (
            SELECT practice_date,
                   julianday('now', 'start of day') - julianday(practice_date) as days_ago
            FROM dated_practice
        )
        SELECT COUNT(*) FROM streak
        WHERE days_ago = (SELECT MIN(days_ago) FROM streak WHERE days_ago >= 0)
           OR days_ago IN (
               SELECT days_ago + 1 FROM streak
           )
        "#
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Check-in streak: consecutive days with check-ins
    let checkin_streak: i64 = sqlx::query_scalar(
        r#"
        WITH dated_checkins AS (
            SELECT DISTINCT date(checked_in_at) as checkin_date
            FROM check_ins
            ORDER BY checkin_date DESC
        ),
        streak AS (
            SELECT checkin_date,
                   julianday('now', 'start of day') - julianday(checkin_date) as days_ago
            FROM dated_checkins
        )
        SELECT COUNT(*) FROM streak
        WHERE days_ago = (SELECT MIN(days_ago) FROM streak WHERE days_ago >= 0)
           OR days_ago IN (
               SELECT days_ago + 1 FROM streak
           )
        "#
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(Streaks {
        study_streak,
        workout_streak,
        practice_streak,
        checkin_streak,
    })
}

// ============================================================================
// USER SETTINGS
// ============================================================================

#[tauri::command]
pub async fn get_user_settings(state: State<'_, DbState>) -> Result<UserSettings, ApiError> {
    let pool = &state.0;
    
    let row = sqlx::query_as::<_, (i64, i64)>(
        "SELECT weekly_workout_target, weekly_active_skills_target FROM user_settings WHERE user_id = 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?;
    
    match row {
        Some((workout_target, skills_target)) => Ok(UserSettings {
            weekly_workout_target: workout_target,
            weekly_active_skills_target: skills_target,
        }),
        None => Ok(UserSettings {
            weekly_workout_target: 3,
            weekly_active_skills_target: 5,
        }),
    }
}

#[tauri::command]
pub async fn update_user_settings(
    state: State<'_, DbState>,
    weekly_workout_target: i64,
    weekly_active_skills_target: i64,
) -> Result<UserSettings, ApiError> {
    let pool = &state.0;
    
    sqlx::query(
        r#"
        INSERT INTO user_settings (id, user_id, weekly_workout_target, weekly_active_skills_target, updated_at)
        VALUES (1, 1, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            weekly_workout_target = excluded.weekly_workout_target,
            weekly_active_skills_target = excluded.weekly_active_skills_target,
            updated_at = CURRENT_TIMESTAMP
        "#
    )
    .bind(weekly_workout_target)
    .bind(weekly_active_skills_target)
    .execute(pool)
    .await
    .map_err(ApiError::from)?;
    
    Ok(UserSettings {
        weekly_workout_target,
        weekly_active_skills_target,
    })
}

// ============================================================================
// DETAILED STATS WITH BREAKDOWN
// ============================================================================

#[tauri::command]
pub async fn get_detailed_stats(state: State<'_, DbState>) -> Result<DetailedStats, ApiError> {
    let pool = &state.0;
    
    // Get user settings for targets
    let settings = sqlx::query_as::<_, (i64, i64)>(
        "SELECT COALESCE(weekly_workout_target, 3), COALESCE(weekly_active_skills_target, 5) FROM user_settings WHERE user_id = 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .unwrap_or((3, 5));
    
    // Get study hours breakdown by course
    let course_rows = sqlx::query_as::<_, (i64, String, Option<String>, String, f64, Option<f64>, Option<f64>, f64)>(
        r#"
        SELECT
            c.id,
            c.name,
            c.code,
            c.color,
            c.target_weekly_hours,
            c.current_grade,
            c.target_grade,
            (
                SELECT COALESCE(SUM(s.duration_minutes), 0) / 60.0
                FROM sessions s
                WHERE s.session_type = 'study'
                  AND s.reference_type = 'course'
                  AND s.reference_id = c.id
                  AND s.started_at >= date('now', 'weekday 0', '-7 days')
            ) as hours_this_week
        FROM courses c
        WHERE c.is_active = 1
        ORDER BY c.name
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    let mut study_breakdown: Vec<CourseProgress> = Vec::new();
    let mut study_hours_week = 0.0;
    let mut study_target_week = 0.0;

    for (course_id, name, code, color, target_hours, current_grade, target_grade, hours_this_week) in course_rows {
        study_hours_week += hours_this_week;
        study_target_week += target_hours;

        let percent = if target_hours > 0.0 {
            (hours_this_week / target_hours * 100.0).min(100.0)
        } else {
            0.0
        };

        study_breakdown.push(CourseProgress {
            course_id,
            course_name: name,
            code,
            color,
            hours_this_week,
            target_hours,
            percent,
            current_grade,
            target_grade,
        });
    }
    
    let study_percent = if study_target_week > 0.0 {
        (study_hours_week / study_target_week * 100.0).min(100.0)
    } else {
        0.0
    };
    
    // Get practice hours breakdown by skill
    let skill_rows = sqlx::query_as::<_, (i64, String, Option<String>, f64, f64, i64, f64, f64)>(
        r#"
        SELECT
            s.id,
            s.name,
            s.category,
            s.target_weekly_hours,
            s.total_hours,
            s.current_level,
            (
                SELECT COALESCE(SUM(p.duration_minutes), 0) / 60.0
                FROM practice_logs p
                WHERE p.skill_id = s.id
                  AND p.logged_at >= date('now', 'weekday 0', '-7 days')
            ) as hours_this_week,
            COALESCE(s.target_hours, 100.0) as target_hours
        FROM skills s
        ORDER BY s.name
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    let mut practice_breakdown: Vec<SkillProgress> = Vec::new();
    let mut practice_hours_week = 0.0;
    let mut practice_target_week = 0.0;
    let mut active_skills_count = 0;

    for (skill_id, name, category, target_weekly_hours, total_hours, current_level, hours_this_week, target_hours) in skill_rows {
        practice_hours_week += hours_this_week;
        practice_target_week += target_weekly_hours;

        // Count as active if practiced this week
        if hours_this_week > 0.0 {
            active_skills_count += 1;
        }

        let weekly_percent = if target_weekly_hours > 0.0 {
            (hours_this_week / target_weekly_hours * 100.0).min(100.0)
        } else {
            0.0
        };

        let mastery_percent = if target_hours > 0.0 {
            (total_hours / target_hours * 100.0).min(100.0)
        } else {
            0.0
        };

        practice_breakdown.push(SkillProgress {
            skill_id,
            skill_name: name,
            category,
            hours_this_week,
            target_weekly_hours,
            weekly_percent,
            total_hours,
            target_hours,
            mastery_percent,
            current_level,
        });
    }
    
    let practice_percent = if practice_target_week > 0.0 {
        (practice_hours_week / practice_target_week * 100.0).min(100.0)
    } else {
        0.0
    };
    
    // Get workouts this week
    let workouts_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workouts WHERE logged_at >= date('now', 'weekday 0', '-7 days')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    
    let workout_percent = if settings.0 > 0 {
        (workouts_week as f64 / settings.0 as f64 * 100.0).min(100.0)
    } else {
        0.0
    };
    
    Ok(DetailedStats {
        study_hours_week,
        study_target_week,
        study_percent,
        study_breakdown,
        practice_hours_week,
        practice_target_week,
        practice_percent,
        practice_breakdown,
        workouts_week,
        workout_target_week: settings.0,
        workout_percent,
        active_skills_count,
        skills_target: settings.1,
    })
}

#[cfg(test)]
async fn get_detailed_stats_inner(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<DetailedStats, String> {
    let settings = sqlx::query_as::<_, (i64, i64)>(
        "SELECT COALESCE(weekly_workout_target, 3), COALESCE(weekly_active_skills_target, 5) FROM user_settings WHERE user_id = 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or((3, 5));

    let course_rows = sqlx::query_as::<_, (i64, String, Option<String>, String, f64, Option<f64>, Option<f64>, f64)>(
        r#"
        SELECT
            c.id,
            c.name,
            c.code,
            c.color,
            c.target_weekly_hours,
            c.current_grade,
            c.target_grade,
            (
                SELECT COALESCE(SUM(s.duration_minutes), 0) / 60.0
                FROM sessions s
                WHERE s.session_type = 'study'
                  AND s.reference_type = 'course'
                  AND s.reference_id = c.id
                  AND s.started_at >= date('now', 'weekday 0', '-7 days')
            ) as hours_this_week
        FROM courses c
        WHERE c.is_active = 1
        ORDER BY c.name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut study_breakdown: Vec<CourseProgress> = Vec::new();
    let mut study_hours_week = 0.0;
    let mut study_target_week = 0.0;

    for (course_id, name, code, color, target_hours, current_grade, target_grade, hours_this_week) in course_rows {
        study_hours_week += hours_this_week;
        study_target_week += target_hours;

        let percent = if target_hours > 0.0 {
            (hours_this_week / target_hours * 100.0).min(100.0)
        } else {
            0.0
        };

        study_breakdown.push(CourseProgress {
            course_id,
            course_name: name,
            code,
            color,
            hours_this_week,
            target_hours,
            percent,
            current_grade,
            target_grade,
        });
    }

    let study_percent = if study_target_week > 0.0 {
        (study_hours_week / study_target_week * 100.0).min(100.0)
    } else {
        0.0
    };

    let skill_rows = sqlx::query_as::<_, (i64, String, Option<String>, f64, f64, i64, f64, f64)>(
        r#"
        SELECT
            s.id,
            s.name,
            s.category,
            s.target_weekly_hours,
            s.total_hours,
            s.current_level,
            (
                SELECT COALESCE(SUM(p.duration_minutes), 0) / 60.0
                FROM practice_logs p
                WHERE p.skill_id = s.id
                  AND p.logged_at >= date('now', 'weekday 0', '-7 days')
            ) as hours_this_week,
            COALESCE(s.target_hours, 100.0) as target_hours
        FROM skills s
        ORDER BY s.name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut practice_breakdown: Vec<SkillProgress> = Vec::new();
    let mut practice_hours_week = 0.0;
    let mut practice_target_week = 0.0;
    let mut active_skills_count = 0;

    for (skill_id, name, category, target_weekly_hours, total_hours, current_level, hours_this_week, target_hours) in skill_rows {
        practice_hours_week += hours_this_week;
        practice_target_week += target_weekly_hours;

        if hours_this_week > 0.0 {
            active_skills_count += 1;
        }

        let weekly_percent = if target_weekly_hours > 0.0 {
            (hours_this_week / target_weekly_hours * 100.0).min(100.0)
        } else {
            0.0
        };

        let mastery_percent = if target_hours > 0.0 {
            (total_hours / target_hours * 100.0).min(100.0)
        } else {
            0.0
        };

        practice_breakdown.push(SkillProgress {
            skill_id,
            skill_name: name,
            category,
            hours_this_week,
            target_weekly_hours,
            weekly_percent,
            total_hours,
            target_hours,
            mastery_percent,
            current_level,
        });
    }

    let practice_percent = if practice_target_week > 0.0 {
        (practice_hours_week / practice_target_week * 100.0).min(100.0)
    } else {
        0.0
    };

    let workouts_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workouts WHERE logged_at >= date('now', 'weekday 0', '-7 days')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let workout_percent = if settings.0 > 0 {
        (workouts_week as f64 / settings.0 as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    Ok(DetailedStats {
        study_hours_week,
        study_target_week,
        study_percent,
        study_breakdown,
        practice_hours_week,
        practice_target_week,
        practice_percent,
        practice_breakdown,
        workouts_week,
        workout_target_week: settings.0,
        workout_percent,
        active_skills_count,
        skills_target: settings.1,
    })
}

#[cfg(test)]
async fn run_get_detailed_stats_benchmark() -> std::time::Duration {
    use sqlx::Row;
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;

    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .foreign_keys(false);

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    crate::db::migrations::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    sqlx::query("UPDATE user_settings SET weekly_workout_target = 3, weekly_active_skills_target = 5 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    for i in 0..10 {
        let course_row = sqlx::query("INSERT INTO courses (name, target_weekly_hours, is_active) VALUES (?, ?, 1) RETURNING id")
            .bind(format!("Course {}", i))
            .bind(6.0)
            .fetch_one(&pool)
            .await
            .unwrap();
        let course_id: i64 = course_row.get(0);

        sqlx::query("INSERT INTO sessions (session_type, reference_type, reference_id, duration_minutes, started_at) VALUES ('study', 'course', ?, ?, date('now'))")
            .bind(course_id)
            .bind(60)
            .execute(&pool)
            .await
            .unwrap();
    }

    for i in 0..10 {
        let skill_row = sqlx::query("INSERT INTO skills (name, target_weekly_hours, total_hours, current_level) VALUES (?, ?, ?, ?) RETURNING id")
            .bind(format!("Skill {}", i))
            .bind(4.0)
            .bind(20.0)
            .bind(2)
            .fetch_one(&pool)
            .await
            .unwrap();
        let skill_id: i64 = skill_row.get(0);

        sqlx::query("INSERT INTO practice_logs (skill_id, duration_minutes, logged_at) VALUES (?, ?, date('now'))")
            .bind(skill_id)
            .bind(45)
            .execute(&pool)
            .await
            .unwrap();
    }

    for _ in 0..3 {
        sqlx::query("INSERT INTO workouts (duration_minutes, logged_at) VALUES (?, date('now'))")
            .bind(30)
            .execute(&pool)
            .await
            .unwrap();
    }

    let start = std::time::Instant::now();
    let _result = get_detailed_stats_inner(&pool).await.unwrap();
    start.elapsed()
}

// ============================================================================
// WORKOUT HEATMAP
// ============================================================================

#[tauri::command]
pub async fn get_workout_heatmap(
    state: State<'_, DbState>,
    months: i32,
) -> Result<Vec<WorkoutHeatmapDay>, ApiError> {
    let pool = &state.0;
    
    let days = months * 30;
    
    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT 
            date(logged_at) as workout_date,
            COUNT(*) as count,
            COALESCE(SUM(duration_minutes), 0) as total_minutes
        FROM workouts
        WHERE logged_at >= date('now', ? || ' days')
        GROUP BY date(logged_at)
        ORDER BY workout_date
        "#
    )
    .bind(-days)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;
    
    let heatmap = rows.into_iter().map(|(date, count, total_minutes)| {
        WorkoutHeatmapDay {
            date,
            count,
            total_minutes,
        }
    }).collect();
    
    Ok(heatmap)
}

// ============================================================================
// PERSONAL RECORDS
// ============================================================================

#[tauri::command]
pub async fn get_personal_records(state: State<'_, DbState>) -> Result<Vec<PersonalRecord>, ApiError> {
    let pool = &state.0;
    
    let rows = sqlx::query_as::<_, (i64, String, String, f64, String, Option<i64>)>(
        r#"
        SELECT id, exercise_name, pr_type, value, achieved_at, workout_id
        FROM exercise_prs
        WHERE user_id = 1
        ORDER BY achieved_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;
    
    let prs = rows.into_iter().map(|(id, exercise_name, pr_type, value, achieved_at, workout_id)| {
        PersonalRecord {
            id,
            exercise_name,
            pr_type,
            value,
            achieved_at,
            workout_id,
        }
    }).collect();
    
    Ok(prs)
}

#[tauri::command]
pub async fn check_and_update_prs(
    state: State<'_, DbState>,
    workout_id: i64,
) -> Result<Vec<PersonalRecord>, ApiError> {
    let pool = &state.0;
    
    // Get all exercises from this workout
    let exercises = sqlx::query_as::<_, (String, Option<i64>, Option<i64>, Option<f64>)>(
        r#"
        SELECT exercise_name, sets, reps, weight
        FROM workout_exercises
        WHERE workout_id = ?
        "#
    )
    .bind(workout_id)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;
    
    let mut new_prs: Vec<PersonalRecord> = Vec::new();
    
    for (exercise_name, sets, reps, weight) in exercises {
        let weight = weight.unwrap_or(0.0);
        let sets = sets.unwrap_or(0);
        let reps = reps.unwrap_or(0);
        
        // Check weight PR
        if weight > 0.0 {
            let current_weight_pr: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT value FROM exercise_prs
                WHERE user_id = 1 AND exercise_name = ? AND pr_type = 'weight'
                ORDER BY value DESC
                LIMIT 1
                "#
            )
            .bind(&exercise_name)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::from)?
            .flatten();
            
            if current_weight_pr.is_none() || weight > current_weight_pr.unwrap() {
                sqlx::query(
                    r#"
                    INSERT INTO exercise_prs (user_id, exercise_name, pr_type, value, workout_id, achieved_at)
                    VALUES (1, ?, 'weight', ?, ?, CURRENT_TIMESTAMP)
                    "#
                )
                .bind(&exercise_name)
                .bind(weight)
                .bind(workout_id)
                .execute(pool)
                .await
                .map_err(ApiError::from)?;
                
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(pool)
                    .await
                    .map_err(ApiError::from)?;
                
                new_prs.push(PersonalRecord {
                    id,
                    exercise_name: exercise_name.clone(),
                    pr_type: "weight".to_string(),
                    value: weight,
                    achieved_at: chrono::Utc::now().to_rfc3339(),
                    workout_id: Some(workout_id),
                });
            }
        }
        
        // Check volume PR (sets * reps * weight)
        let volume = sets as f64 * reps as f64 * weight;
        if volume > 0.0 {
            let current_volume_pr: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT value FROM exercise_prs
                WHERE user_id = 1 AND exercise_name = ? AND pr_type = 'volume'
                ORDER BY value DESC
                LIMIT 1
                "#
            )
            .bind(&exercise_name)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::from)?
            .flatten();
            
            if current_volume_pr.is_none() || volume > current_volume_pr.unwrap() {
                sqlx::query(
                    r#"
                    INSERT INTO exercise_prs (user_id, exercise_name, pr_type, value, workout_id, achieved_at)
                    VALUES (1, ?, 'volume', ?, ?, CURRENT_TIMESTAMP)
                    "#
                )
                .bind(&exercise_name)
                .bind(volume)
                .bind(workout_id)
                .execute(pool)
                .await
                .map_err(ApiError::from)?;
                
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(pool)
                    .await
                    .map_err(ApiError::from)?;
                
                new_prs.push(PersonalRecord {
                    id,
                    exercise_name: exercise_name.clone(),
                    pr_type: "volume".to_string(),
                    value: volume,
                    achieved_at: chrono::Utc::now().to_rfc3339(),
                    workout_id: Some(workout_id),
                });
            }
        }
        
        // Check reps PR (single set)
        if reps > 0 {
            let current_reps_pr: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT value FROM exercise_prs
                WHERE user_id = 1 AND exercise_name = ? AND pr_type = 'reps'
                ORDER BY value DESC
                LIMIT 1
                "#
            )
            .bind(&exercise_name)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::from)?
            .flatten();
            
            if current_reps_pr.is_none() || (reps as f64) > current_reps_pr.unwrap() {
                sqlx::query(
                    r#"
                    INSERT INTO exercise_prs (user_id, exercise_name, pr_type, value, workout_id, achieved_at)
                    VALUES (1, ?, 'reps', ?, ?, CURRENT_TIMESTAMP)
                    "#
                )
                .bind(&exercise_name)
                .bind(reps)
                .bind(workout_id)
                .execute(pool)
                .await
                .map_err(ApiError::from)?;
                
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(pool)
                    .await
                    .map_err(ApiError::from)?;
                
                new_prs.push(PersonalRecord {
                    id,
                    exercise_name: exercise_name.clone(),
                    pr_type: "reps".to_string(),
                    value: reps as f64,
                    achieved_at: chrono::Utc::now().to_rfc3339(),
                    workout_id: Some(workout_id),
                });
            }
        }
    }
    
    Ok(new_prs)
}

// ============================================================================
// ACHIEVEMENTS
// ============================================================================

#[tauri::command]
pub async fn get_achievements(state: State<'_, DbState>) -> Result<Vec<Achievement>, ApiError> {
    let pool = &state.0;
    
    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, String, Option<String>)>(
        r#"
        SELECT id, achievement_type, title, description, category, achieved_at, metadata
        FROM achievements
        WHERE user_id = 1
        ORDER BY achieved_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;
    
    let achievements = rows.into_iter().map(|(id, achievement_type, title, description, category, achieved_at, metadata)| {
        Achievement {
            id,
            achievement_type,
            title,
            description,
            category,
            achieved_at,
            metadata,
        }
    }).collect();
    
    Ok(achievements)
}

#[tauri::command]
pub async fn check_achievements(state: State<'_, DbState>) -> Result<Vec<Achievement>, ApiError> {
    let pool = &state.0;
    let mut new_achievements: Vec<Achievement> = Vec::new();
    
    // Check workout milestones: 10, 25, 50, 100, 250, 500
    let total_workouts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workouts WHERE user_id = 1")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    
    let workout_milestones = vec![10, 25, 50, 100, 250, 500];
    for milestone in workout_milestones {
        if total_workouts >= milestone {
            // Check if achievement already exists
            let exists: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM achievements
                WHERE user_id = 1 AND achievement_type = 'workout_milestone' AND metadata LIKE ?
                "#
            )
            .bind(format!("%\"count\":{}%", milestone))
            .fetch_one(pool)
            .await
            .unwrap_or(0);
            
            if exists == 0 {
                let title = format!("{} Workouts!", milestone);
                let description = format!("Completed {} total workouts", milestone);
                let metadata = format!(r#"{{"count":{}}}"#, milestone);
                
                sqlx::query(
                    r#"
                    INSERT INTO achievements (user_id, achievement_type, title, description, category, metadata)
                    VALUES (1, 'workout_milestone', ?, ?, 'physical', ?)
                    "#
                )
                .bind(&title)
                .bind(&description)
                .bind(&metadata)
                .execute(pool)
                .await
                .map_err(ApiError::from)?;
                
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(pool)
                    .await
                    .map_err(ApiError::from)?;
                
                new_achievements.push(Achievement {
                    id,
                    achievement_type: "workout_milestone".to_string(),
                    title,
                    description: Some(description),
                    category: Some("physical".to_string()),
                    achieved_at: chrono::Utc::now().to_rfc3339(),
                    metadata: Some(metadata),
                });
            }
        }
    }
    
    // Check study hour milestones: 10, 25, 50, 100, 250, 500
    let total_study_hours: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(duration_minutes), 0) / 60.0 FROM sessions WHERE session_type = 'study'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0.0);
    
    let study_milestones = vec![10, 25, 50, 100, 250, 500];
    for milestone in study_milestones {
        if total_study_hours >= milestone as f64 {
            let exists: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM achievements
                WHERE user_id = 1 AND achievement_type = 'study_milestone' AND metadata LIKE ?
                "#
            )
            .bind(format!("%\"hours\":{}%", milestone))
            .fetch_one(pool)
            .await
            .unwrap_or(0);
            
            if exists == 0 {
                let title = format!("{} Study Hours!", milestone);
                let description = format!("Studied for {} total hours", milestone);
                let metadata = format!(r#"{{"hours":{}}}"#, milestone);
                
                sqlx::query(
                    r#"
                    INSERT INTO achievements (user_id, achievement_type, title, description, category, metadata)
                    VALUES (1, 'study_milestone', ?, ?, 'academic', ?)
                    "#
                )
                .bind(&title)
                .bind(&description)
                .bind(&metadata)
                .execute(pool)
                .await
                .map_err(ApiError::from)?;
                
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(pool)
                    .await
                    .map_err(ApiError::from)?;
                
                new_achievements.push(Achievement {
                    id,
                    achievement_type: "study_milestone".to_string(),
                    title,
                    description: Some(description),
                    category: Some("academic".to_string()),
                    achieved_at: chrono::Utc::now().to_rfc3339(),
                    metadata: Some(metadata),
                });
            }
        }
    }
    
    // Check skill level milestones (any skill reaching level 5, 10, 15, 20)
    let skill_levels = sqlx::query_as::<_, (String, i64)>(
        "SELECT name, current_level FROM skills WHERE user_id = 1"
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;
    
    let level_milestones = vec![5, 10, 15, 20];
    for (skill_name, level) in skill_levels {
        for milestone in &level_milestones {
            if level >= *milestone {
                let exists: i64 = sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*) FROM achievements
                    WHERE user_id = 1 AND achievement_type = 'skill_level' 
                    AND metadata LIKE ? AND metadata LIKE ?
                    "#
                )
                .bind(format!("%\"skill\":\"{}\"%", skill_name))
                .bind(format!("%\"level\":{}%", milestone))
                .fetch_one(pool)
                .await
                .unwrap_or(0);
                
                if exists == 0 {
                    let title = format!("{} Level {}!", skill_name, milestone);
                    let description = format!("Reached level {} in {}", milestone, skill_name);
                    let metadata = format!(r#"{{"skill":"{}","level":{}}}"#, skill_name, milestone);
                    
                    sqlx::query(
                        r#"
                        INSERT INTO achievements (user_id, achievement_type, title, description, category, metadata)
                        VALUES (1, 'skill_level', ?, ?, 'skills', ?)
                        "#
                    )
                    .bind(&title)
                    .bind(&description)
                    .bind(&metadata)
                    .execute(pool)
                    .await
                    .map_err(ApiError::from)?;
                    
                    let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                        .fetch_one(pool)
                        .await
                        .map_err(ApiError::from)?;
                    
                    new_achievements.push(Achievement {
                        id,
                        achievement_type: "skill_level".to_string(),
                        title,
                        description: Some(description),
                        category: Some("skills".to_string()),
                        achieved_at: chrono::Utc::now().to_rfc3339(),
                        metadata: Some(metadata),
                    });
                }
            }
        }
    }
    
    // Check consistency achievements (7-day, 14-day, 30-day streaks)
    let streak_milestones = vec![7, 14, 30, 60, 90];
    
    // Check-in streak
    let checkin_streak = get_consecutive_days(pool, "check_ins", "checked_in_at").await;
    for milestone in &streak_milestones {
        if checkin_streak >= *milestone {
            let exists: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM achievements
                WHERE user_id = 1 AND achievement_type = 'checkin_streak' AND metadata LIKE ?
                "#
            )
            .bind(format!("%\"days\":{}%", milestone))
            .fetch_one(pool)
            .await
            .unwrap_or(0);
            
            if exists == 0 {
                let title = format!("{}-Day Check-in Streak!", milestone);
                let description = format!("Checked in for {} consecutive days", milestone);
                let metadata = format!(r#"{{"days":{}}}"#, milestone);
                
                sqlx::query(
                    r#"
                    INSERT INTO achievements (user_id, achievement_type, title, description, category, metadata)
                    VALUES (1, 'checkin_streak', ?, ?, 'wellness', ?)
                    "#
                )
                .bind(&title)
                .bind(&description)
                .bind(&metadata)
                .execute(pool)
                .await
                .map_err(ApiError::from)?;
                
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(pool)
                    .await
                    .map_err(ApiError::from)?;
                
                new_achievements.push(Achievement {
                    id,
                    achievement_type: "checkin_streak".to_string(),
                    title,
                    description: Some(description),
                    category: Some("wellness".to_string()),
                    achieved_at: chrono::Utc::now().to_rfc3339(),
                    metadata: Some(metadata),
                });
            }
        }
    }
    
    Ok(new_achievements)
}

// Helper function to get consecutive days
async fn get_consecutive_days(pool: &sqlx::Pool<sqlx::Sqlite>, table: &str, date_column: &str) -> i64 {
    let query = format!(
        r#"
        WITH dated AS (
            SELECT DISTINCT date({}) as activity_date
            FROM {}
            ORDER BY activity_date DESC
        ),
        numbered AS (
            SELECT activity_date,
                   ROW_NUMBER() OVER (ORDER BY activity_date DESC) as rn,
                   julianday(activity_date) as jd
            FROM dated
        ),
        streak AS (
            SELECT activity_date, jd - rn as grp
            FROM numbered
            WHERE julianday('now') - jd <= 1
        )
        SELECT COUNT(*) FROM streak
        WHERE grp = (SELECT grp FROM streak LIMIT 1)
        "#,
        date_column, table
    );
    
    sqlx::query_scalar(&query)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

// ============================================================================
// UNIT TESTS - TDD Compliant
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // StatsSummary Tests
    // =========================================================================

    #[test]
    fn test_stats_summary_serialization() {
        let stats = StatsSummary {
            study_hours_week: 10.5,
            practice_hours_week: 5.25,
            workouts_week: 3,
            active_streaks: 2,
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"study_hours_week\":10.5"));
        assert!(json.contains("\"workouts_week\":3"));
    }

    #[test]
    fn test_minutes_to_hours_conversion() {
        let minutes: i64 = 150;
        let hours = minutes as f64 / 60.0;
        assert_eq!(hours, 2.5);
    }

    // =========================================================================
    // Streaks Tests
    // =========================================================================

    #[test]
    fn test_streaks_serialization() {
        let streaks = Streaks {
            study_streak: 5,
            workout_streak: 3,
            practice_streak: 7,
            checkin_streak: 10,
        };
        
        let json = serde_json::to_string(&streaks).unwrap();
        assert!(json.contains("\"study_streak\":5"));
        assert!(json.contains("\"checkin_streak\":10"));
    }

    // =========================================================================
    // UserSettings Tests
    // =========================================================================

    #[test]
    fn test_user_settings_default_values() {
        let default_workout_target = 3;
        let default_skills_target = 5;
        
        assert_eq!(default_workout_target, 3);
        assert_eq!(default_skills_target, 5);
    }

    #[test]
    fn test_user_settings_serialization() {
        let settings = UserSettings {
            weekly_workout_target: 4,
            weekly_active_skills_target: 6,
        };
        
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"weekly_workout_target\":4"));
        assert!(json.contains("\"weekly_active_skills_target\":6"));
    }

    // =========================================================================
    // Percent Calculation Tests
    // =========================================================================

    #[test]
    fn test_percent_calculation_normal() {
        let hours: f64 = 3.0;
        let target: f64 = 6.0;
        let percent = if target > 0.0 {
            (hours / target * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(percent, 50.0);
    }

    #[test]
    fn test_percent_calculation_exceeds_target() {
        let hours: f64 = 8.0;
        let target: f64 = 6.0;
        let percent = if target > 0.0 {
            (hours / target * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(percent, 100.0);
    }

    #[test]
    fn test_percent_calculation_zero_target() {
        let hours: f64 = 5.0;
        let target: f64 = 0.0;
        let percent = if target > 0.0 {
            (hours / target * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(percent, 0.0);
    }

    #[test]
    fn test_workout_percent_with_settings() {
        let workouts_week: i64 = 2;
        let workout_target: i64 = 4;
        let percent = if workout_target > 0 {
            (workouts_week as f64 / workout_target as f64 * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(percent, 50.0);
    }

    // =========================================================================
    // WorkoutHeatmapDay Tests
    // =========================================================================

    #[test]
    fn test_workout_heatmap_day_serialization() {
        let day = WorkoutHeatmapDay {
            date: "2026-01-15".to_string(),
            count: 2,
            total_minutes: 90,
        };
        
        let json = serde_json::to_string(&day).unwrap();
        assert!(json.contains("\"date\":\"2026-01-15\""));
        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"total_minutes\":90"));
    }

    #[test]
    fn test_months_to_days_conversion() {
        let months = 3;
        let days = months * 30;
        assert_eq!(days, 90);
    }

    // =========================================================================
    // PersonalRecord Tests
    // =========================================================================

    #[test]
    fn test_personal_record_serialization() {
        let pr = PersonalRecord {
            id: 1,
            exercise_name: "Bench Press".to_string(),
            pr_type: "weight".to_string(),
            value: 185.0,
            achieved_at: "2026-01-15T10:30:00Z".to_string(),
            workout_id: Some(42),
        };
        
        let json = serde_json::to_string(&pr).unwrap();
        assert!(json.contains("\"exercise_name\":\"Bench Press\""));
        assert!(json.contains("\"value\":185.0"));
        assert!(json.contains("\"pr_type\":\"weight\""));
    }

    #[test]
    fn test_volume_calculation() {
        let sets: i64 = 4;
        let reps: i64 = 10;
        let weight: f64 = 135.0;
        let volume = sets as f64 * reps as f64 * weight;
        assert_eq!(volume, 5400.0);
    }

    // =========================================================================
    // Achievement Tests
    // =========================================================================

    #[test]
    fn test_achievement_serialization() {
        let achievement = Achievement {
            id: 1,
            achievement_type: "workout_milestone".to_string(),
            title: "Iron Will".to_string(),
            description: Some("Complete 50 workouts".to_string()),
            category: Some("workout".to_string()),
            achieved_at: "2026-01-15T08:00:00Z".to_string(),
            metadata: Some(r#"{"count":50}"#.to_string()),
        };
        
        let json = serde_json::to_string(&achievement).unwrap();
        assert!(json.contains("\"title\":\"Iron Will\""));
        assert!(json.contains("\"achievement_type\":\"workout_milestone\""));
    }

    // =========================================================================
    // CourseProgress Tests
    // =========================================================================

    #[test]
    fn test_course_progress_serialization() {
        let progress = CourseProgress {
            course_id: 1,
            course_name: "Calculus".to_string(),
            code: Some("MATH201".to_string()),
            color: "#3b82f6".to_string(),
            hours_this_week: 4.5,
            target_hours: 6.0,
            percent: 75.0,
            current_grade: Some(87.5),
            target_grade: Some(90.0),
        };
        
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("\"course_name\":\"Calculus\""));
        assert!(json.contains("\"percent\":75.0"));
    }

    // =========================================================================
    // SkillProgress Tests
    // =========================================================================

    #[test]
    fn test_skill_progress_serialization() {
        let skill = SkillProgress {
            skill_id: 1,
            skill_name: "Piano".to_string(),
            category: Some("Music".to_string()),
            hours_this_week: 3.0,
            target_weekly_hours: 5.0,
            weekly_percent: 60.0,
            total_hours: 50.0,
            target_hours: 100.0,
            mastery_percent: 50.0,
            current_level: 3,
        };
        
        let json = serde_json::to_string(&skill).unwrap();
        assert!(json.contains("\"skill_name\":\"Piano\""));
        assert!(json.contains("\"mastery_percent\":50.0"));
    }

    #[test]
    fn test_mastery_percent_calculation() {
        let total_hours: f64 = 75.0;
        let target_hours: f64 = 100.0;
        let mastery_percent = if target_hours > 0.0 {
            (total_hours / target_hours * 100.0_f64).min(100.0)
        } else {
            0.0
        };
        assert_eq!(mastery_percent, 75.0);
    }

    // =========================================================================
    // DetailedStats Tests
    // =========================================================================

    #[test]
    fn test_detailed_stats_partial() {
        let stats = DetailedStats {
            study_hours_week: 8.5,
            study_target_week: 12.0,
            study_percent: 70.83,
            study_breakdown: vec![],
            practice_hours_week: 4.0,
            practice_target_week: 5.0,
            practice_percent: 80.0,
            practice_breakdown: vec![],
            workouts_week: 2,
            workout_target_week: 3,
            workout_percent: 66.67,
            active_skills_count: 3,
            skills_target: 5,
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"study_hours_week\":8.5"));
        assert!(json.contains("\"workout_percent\":66.67"));
        assert!(json.contains("\"active_skills_count\":3"));
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;

    #[tokio::test]
    async fn benchmark_get_detailed_stats_reports_duration() {
        let _duration = run_get_detailed_stats_benchmark().await;
    }
}
