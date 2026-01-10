use tauri::State;

use crate::DbState;

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

#[tauri::command]
pub async fn get_stats(state: State<'_, DbState>) -> Result<StatsSummary, String> {
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
pub async fn get_streaks(state: State<'_, DbState>) -> Result<Streaks, String> {
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
