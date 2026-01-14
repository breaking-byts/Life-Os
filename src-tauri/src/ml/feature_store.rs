//! Feature Store - Extracts and manages behavioral features from user data
//!
//! Captures context snapshots for ML training and real-time decision making.

#![allow(dead_code)] // Legacy feature store for backwards compatibility

use sqlx::{Pool, Sqlite};
use chrono::{Local, Datelike, Timelike};

use super::models::Context;

/// Feature store for capturing and retrieving user context
pub struct FeatureStore;

impl FeatureStore {
    /// Capture current context from the database
    pub async fn capture_context(pool: &Pool<Sqlite>) -> Result<Context, String> {
        let now = Local::now();
        let hour_of_day = now.hour() as i32;
        let day_of_week = now.weekday().num_days_from_sunday() as i32;

        // Get latest check-in for mood/energy
        let (mood, energy): (Option<i32>, Option<i32>) = sqlx::query_as(
            r#"
            SELECT mood, energy FROM check_ins
            WHERE date(checked_in_at) = date('now')
            ORDER BY checked_in_at DESC LIMIT 1
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or((None, None));

        // Recent study minutes (last 24h)
        let recent_study_minutes: i64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions
            WHERE session_type = 'study'
            AND started_at >= datetime('now', '-1 day')
            "#
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // Recent workout count (last 24h)
        let recent_workout_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM workouts
            WHERE logged_at >= datetime('now', '-1 day')
            "#
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // Active (incomplete) assignments
        let active_assignments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM assignments WHERE is_completed = 0"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // Overdue assignments
        let overdue_assignments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM assignments WHERE is_completed = 0 AND due_date < date('now')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // Best current streak (simplified - just check-in streak for now)
        let current_streak_days: i64 = sqlx::query_scalar(
            r#"
            WITH dated_checkins AS (
                SELECT DISTINCT date(checked_in_at) as checkin_date
                FROM check_ins
                ORDER BY checkin_date DESC
            )
            SELECT COUNT(*) FROM dated_checkins
            WHERE julianday('now', 'start of day') - julianday(checkin_date) <= 7
            "#
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // Check if session is in progress
        let session_in_progress: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sessions WHERE ended_at IS NULL"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(Context {
            hour_of_day,
            day_of_week,
            mood: mood.map(|m| m as i32),
            energy: energy.map(|e| e as i32),
            recent_study_minutes: recent_study_minutes as i32,
            recent_workout_count: recent_workout_count as i32,
            active_assignments: active_assignments as i32,
            overdue_assignments: overdue_assignments as i32,
            current_streak_days: current_streak_days as i32,
            session_in_progress: session_in_progress > 0,
        })
    }

    /// Save a context snapshot for later pattern mining
    pub async fn save_snapshot(pool: &Pool<Sqlite>, ctx: &Context) -> Result<i64, String> {
        let result = sqlx::query(
            r#"
            INSERT INTO agent_feature_snapshots (
                hour_of_day, day_of_week, mood, energy,
                recent_study_minutes, recent_workout_count,
                active_assignments, overdue_assignments,
                current_streak_days, session_in_progress
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(ctx.hour_of_day)
        .bind(ctx.day_of_week)
        .bind(ctx.mood)
        .bind(ctx.energy)
        .bind(ctx.recent_study_minutes)
        .bind(ctx.recent_workout_count)
        .bind(ctx.active_assignments)
        .bind(ctx.overdue_assignments)
        .bind(ctx.current_streak_days)
        .bind(ctx.session_in_progress)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }

    /// Get historical context snapshots for pattern mining
    pub async fn get_snapshots(
        pool: &Pool<Sqlite>,
        days_back: i32
    ) -> Result<Vec<Context>, String> {
        let rows: Vec<(i32, i32, Option<i32>, Option<i32>, i32, i32, i32, i32, i32, bool)> = 
            sqlx::query_as(
            r#"
            SELECT hour_of_day, day_of_week, mood, energy,
                   recent_study_minutes, recent_workout_count,
                   active_assignments, overdue_assignments,
                   current_streak_days, session_in_progress
            FROM agent_feature_snapshots
            WHERE captured_at >= datetime('now', ? || ' days')
            ORDER BY captured_at DESC
            "#
        )
        .bind(-days_back)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| Context {
            hour_of_day: r.0,
            day_of_week: r.1,
            mood: r.2,
            energy: r.3,
            recent_study_minutes: r.4,
            recent_workout_count: r.5,
            active_assignments: r.6,
            overdue_assignments: r.7,
            current_streak_days: r.8,
            session_in_progress: r.9,
        }).collect())
    }
}
