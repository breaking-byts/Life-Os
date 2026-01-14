//! User Profile - Models user preferences and behavioral tendencies
//!
//! Maintains an evolving understanding of the user's patterns,
//! preferences, and productivity rhythms.

#![allow(dead_code)] // Profile dimension getters for future use

use sqlx::{Pool, Sqlite};
use serde::{Deserialize, Serialize};
use serde_json;

use super::models::ProfileDimension;

/// User profile management
pub struct UserProfile;

/// Profile values for different dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileValue {
    /// Preferred hours of day (0-23)
    PeakHours(Vec<i32>),
    /// Preferred days of week (0-6)
    PeakDays(Vec<i32>),
    /// Average values (e.g., avg session length, avg mood)
    Average(f64),
    /// Threshold values (e.g., minimum comfortable streak)
    Threshold(f64),
    /// Preference score (0-1 scale)
    Preference(f64),
}

impl UserProfile {
    /// Update a profile dimension with a new observation
    pub async fn update_dimension(
        pool: &Pool<Sqlite>,
        dimension: &str,
        value: &ProfileValue,
    ) -> Result<(), String> {
        let value_json = serde_json::to_string(value).map_err(|e| e.to_string())?;

        // Get existing value for averaging
        let existing: Option<ProfileDimension> = sqlx::query_as::<_, ProfileDimension>(
            "SELECT * FROM agent_profile WHERE dimension = ?"
        )
        .bind(dimension)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(dim) = existing {
            // Update with exponential moving average for smooth adaptation
            let new_confidence = (dim.confidence * 0.9 + 0.1).min(0.95);  // Increase confidence, cap at 0.95
            let new_count = dim.sample_count + 1;

            sqlx::query(
                r#"
                UPDATE agent_profile 
                SET value_json = ?, confidence = ?, sample_count = ?, updated_at = datetime('now')
                WHERE dimension = ?
                "#
            )
            .bind(&value_json)
            .bind(new_confidence)
            .bind(new_count)
            .bind(dimension)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        } else {
            // Insert new dimension
            sqlx::query(
                r#"
                INSERT INTO agent_profile (dimension, value_json, confidence, sample_count)
                VALUES (?, ?, 0.5, 1)
                "#
            )
            .bind(dimension)
            .bind(&value_json)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Get a profile dimension value
    pub async fn get_dimension(
        pool: &Pool<Sqlite>,
        dimension: &str,
    ) -> Result<Option<(ProfileValue, f64)>, String> {
        let dim: Option<ProfileDimension> = sqlx::query_as(
            "SELECT * FROM agent_profile WHERE dimension = ?"
        )
        .bind(dimension)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(d) = dim {
            let value: ProfileValue = serde_json::from_str(&d.value_json)
                .map_err(|e| e.to_string())?;
            Ok(Some((value, d.confidence)))
        } else {
            Ok(None)
        }
    }

    /// Learn preferred study hours from session data
    pub async fn learn_study_preferences(pool: &Pool<Sqlite>) -> Result<(), String> {
        // Find hours with most study sessions
        let preferred_hours: Vec<(i32,)> = sqlx::query_as(
            r#"
            SELECT CAST(strftime('%H', started_at) AS INTEGER) as hour
            FROM sessions
            WHERE session_type = 'study' AND duration_minutes >= 20
            GROUP BY hour
            ORDER BY COUNT(*) DESC, AVG(duration_minutes) DESC
            LIMIT 4
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        if !preferred_hours.is_empty() {
            let hours: Vec<i32> = preferred_hours.into_iter().map(|(h,)| h).collect();
            Self::update_dimension(pool, "preferred_study_hours", &ProfileValue::PeakHours(hours)).await?;
        }

        // Learn average session length
        let avg_length: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT AVG(duration_minutes) FROM sessions
            WHERE session_type = 'study' AND duration_minutes IS NOT NULL
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some((avg,)) = avg_length {
            Self::update_dimension(pool, "avg_study_session", &ProfileValue::Average(avg)).await?;
        }

        Ok(())
    }

    /// Learn workout preferences
    pub async fn learn_workout_preferences(pool: &Pool<Sqlite>) -> Result<(), String> {
        // Find preferred workout days
        let preferred_days: Vec<(i32,)> = sqlx::query_as(
            r#"
            SELECT CAST(strftime('%w', logged_at) AS INTEGER) as day
            FROM workouts
            GROUP BY day
            ORDER BY COUNT(*) DESC
            LIMIT 3
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        if !preferred_days.is_empty() {
            let days: Vec<i32> = preferred_days.into_iter().map(|(d,)| d).collect();
            Self::update_dimension(pool, "preferred_workout_days", &ProfileValue::PeakDays(days)).await?;
        }

        // Learn workout frequency preference
        let weekly_avg: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT AVG(cnt) FROM (
                SELECT COUNT(*) as cnt
                FROM workouts
                GROUP BY strftime('%Y-%W', logged_at)
            )
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some((avg,)) = weekly_avg {
            Self::update_dimension(pool, "target_weekly_workouts", &ProfileValue::Average(avg)).await?;
        }

        Ok(())
    }

    /// Learn mood/energy baselines
    pub async fn learn_wellbeing_baselines(pool: &Pool<Sqlite>) -> Result<(), String> {
        let baseline: Option<(f64, f64)> = sqlx::query_as(
            r#"
            SELECT AVG(mood), AVG(energy)
            FROM check_ins
            WHERE mood IS NOT NULL AND energy IS NOT NULL
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some((mood, energy)) = baseline {
            Self::update_dimension(pool, "baseline_mood", &ProfileValue::Average(mood)).await?;
            Self::update_dimension(pool, "baseline_energy", &ProfileValue::Average(energy)).await?;
        }

        Ok(())
    }

    /// Run all profile learning tasks
    pub async fn learn_all(pool: &Pool<Sqlite>) -> Result<(), String> {
        Self::learn_study_preferences(pool).await?;
        Self::learn_workout_preferences(pool).await?;
        Self::learn_wellbeing_baselines(pool).await?;
        Ok(())
    }

    /// Get all profile dimensions for display
    pub async fn get_all_dimensions(pool: &Pool<Sqlite>) -> Result<Vec<ProfileDimension>, String> {
        sqlx::query_as::<_, ProfileDimension>(
            "SELECT * FROM agent_profile ORDER BY updated_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }
}
