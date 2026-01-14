//! Data models for the ML module

#![allow(dead_code)] // Multi-scale reward methods for future use

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Context vector captured at decision time (legacy, for backwards compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub hour_of_day: i32,      // 0-23
    pub day_of_week: i32,      // 0-6 (Sunday = 0)
    pub mood: Option<i32>,     // 1-10
    pub energy: Option<i32>,   // 1-10
    pub recent_study_minutes: i32,
    pub recent_workout_count: i32,
    pub active_assignments: i32,
    pub overdue_assignments: i32,
    pub current_streak_days: i32,
    pub session_in_progress: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            hour_of_day: 12,
            day_of_week: 1,
            mood: None,
            energy: None,
            recent_study_minutes: 0,
            recent_workout_count: 0,
            active_assignments: 0,
            overdue_assignments: 0,
            current_streak_days: 0,
            session_in_progress: false,
        }
    }
}

/// A bandit arm representing an action the agent can take
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BanditArm {
    pub id: i64,
    pub arm_name: String,
    pub category: Option<String>,
    pub alpha: f64,  // Beta distribution parameter (successes + 1)
    pub beta: f64,   // Beta distribution parameter (failures + 1)
    pub total_pulls: i64,
    pub total_reward: f64,
    pub last_pulled: Option<String>,
    pub is_enabled: bool,
}

impl BanditArm {
    /// Calculate the expected value (mean) of this arm
    pub fn expected_value(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }
}

/// A discovered behavioral pattern
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Pattern {
    pub id: i64,
    pub pattern_type: String,
    pub pattern_name: Option<String>,
    pub pattern_json: String,
    pub support: f64,
    pub confidence: f64,
    pub last_validated: Option<String>,
    pub is_active: bool,
    pub created_at: Option<String>,
}

/// Structured pattern data that gets serialized to pattern_json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternData {
    /// Time-based pattern (e.g., "productive between 9-11am")
    Temporal {
        peak_hours: Vec<i32>,
        peak_days: Vec<i32>,
        metric: String,  // what's being measured
    },
    /// Sequence pattern (e.g., "workout → high energy check-in → productive study")
    Sequence {
        events: Vec<String>,
        typical_gap_minutes: Vec<i32>,
    },
    /// Correlation pattern (e.g., "mood > 7 correlates with longer sessions")
    Correlation {
        factor_a: String,
        factor_b: String,
        correlation: f64,
        direction: String,  // "positive" or "negative"
    },
}

/// User profile dimension
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProfileDimension {
    pub id: i64,
    pub dimension: String,
    pub value_json: String,
    pub confidence: f64,
    pub sample_count: i64,
    pub updated_at: Option<String>,
}

/// Agent insight with ML-enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveInsight {
    pub icon: String,
    pub message: String,
    pub category: String,
    pub arm_name: String,        // Which bandit arm generated this
    pub confidence: f64,         // How confident the agent is
    pub context_hash: String,    // For tracking
    pub insight_id: Option<i64>, // DB id once recorded
}

/// Feedback on an insight (explicit or implicit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightFeedback {
    pub insight_id: i64,
    pub feedback_type: FeedbackType,
    pub value: f64,  // 0-1 reward signal
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    /// User explicitly rated the insight
    Explicit { score: i32 },  // -1, 0, or 1
    /// User dismissed without action
    Dismissed,
    /// User took the suggested action
    ActedOn,
    /// Time-based implicit signal (e.g., completed task after seeing insight)
    Implicit { action: String },
}

// ============================================================================
// Multi-Scale Reward System
// ============================================================================

use chrono::{Duration, Local, NaiveDate};
use sqlx::{Pool, Sqlite};

/// Default reward weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardWeights {
    pub immediate: f32,
    pub daily: f32,
    pub weekly: f32,
    pub monthly: f32,
}

impl Default for RewardWeights {
    fn default() -> Self {
        Self {
            immediate: 0.2,
            daily: 0.3,
            weekly: 0.3,
            monthly: 0.2,
        }
    }
}

/// Multi-scale reward computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiScaleReward {
    pub immediate: f32,
    pub daily: Option<f32>,
    pub weekly: Option<f32>,
    pub monthly: Option<f32>,
    pub total: Option<f32>,
    pub weights: RewardWeights,
}

impl MultiScaleReward {
    /// Create a new reward with only immediate value
    pub fn new_immediate(immediate: f32) -> Self {
        Self {
            immediate,
            daily: None,
            weekly: None,
            monthly: None,
            total: None,
            weights: RewardWeights::default(),
        }
    }

    /// Compute total reward (when all scales are available)
    pub fn compute_total(&mut self) {
        let mut total = self.weights.immediate * self.immediate;
        let mut weight_sum = self.weights.immediate;

        if let Some(d) = self.daily {
            total += self.weights.daily * d;
            weight_sum += self.weights.daily;
        }

        if let Some(w) = self.weekly {
            total += self.weights.weekly * w;
            weight_sum += self.weights.weekly;
        }

        if let Some(m) = self.monthly {
            total += self.weights.monthly * m;
            weight_sum += self.weights.monthly;
        }

        self.total = Some(total / weight_sum);
    }
}

/// Multi-scale reward computation engine
pub struct RewardEngine;

impl RewardEngine {
    /// Get configured reward weights
    pub async fn get_weights(pool: &Pool<Sqlite>) -> Result<RewardWeights, String> {
        let json: Option<String> = sqlx::query_scalar(
            "SELECT value_json FROM agent_state WHERE key = 'reward_weights'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        match json {
            Some(j) => serde_json::from_str(&j).map_err(|e| e.to_string()),
            None => Ok(RewardWeights::default()),
        }
    }

    /// Compute immediate reward from user feedback
    pub fn compute_immediate_reward(
        acted_on: bool,
        feedback_score: Option<i32>,
        task_completed: bool,
        satisfaction_rating: Option<i32>,
    ) -> f32 {
        let mut reward = 0.0f32;
        let mut weight = 0.0f32;

        reward += if acted_on { 1.0 } else { 0.2 };
        weight += 1.0;

        if let Some(score) = feedback_score {
            reward += (score as f32 + 1.0) / 2.0;
            weight += 1.0;
        }

        if task_completed {
            reward += 1.0;
            weight += 1.0;
        }

        if let Some(rating) = satisfaction_rating {
            reward += (rating as f32 - 1.0) / 4.0;
            weight += 1.0;
        }

        if weight > 0.0 { reward / weight } else { 0.5 }
    }

    /// Compute daily reward
    pub async fn compute_daily_reward(
        pool: &Pool<Sqlite>,
        date: &NaiveDate,
    ) -> Result<f32, String> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let mut reward = 0.0f32;
        let mut weight = 0.0f32;

        // Big 3 completion
        let big3_stats: (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*), COALESCE(SUM(CASE WHEN is_completed = 1 THEN 1 ELSE 0 END), 0) FROM agent_big_three WHERE date = ?",
        )
        .bind(&date_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or((0, 0));

        if big3_stats.0 > 0 {
            reward += big3_stats.1 as f32 / big3_stats.0 as f32;
            weight += 2.0;
        }

        // Check-in completed
        let had_checkin: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM check_ins WHERE date(checked_in_at) = ?",
        )
        .bind(&date_str)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        reward += if had_checkin > 0 { 1.0 } else { 0.0 };
        weight += 1.0;

        // Study balance
        let study_mins: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions WHERE date(started_at) = ?",
        )
        .bind(&date_str)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let study_reward = if study_mins >= 120 && study_mins <= 360 {
            1.0
        } else if study_mins > 0 && study_mins < 120 {
            study_mins as f32 / 120.0
        } else if study_mins > 360 {
            (1.0 - ((study_mins - 360) as f32 / 240.0)).max(0.5)
        } else {
            0.0
        };
        reward += study_reward;
        weight += 1.0;

        Ok(if weight > 0.0 { reward / weight } else { 0.5 })
    }

    /// Compute weekly reward
    pub async fn compute_weekly_reward(
        pool: &Pool<Sqlite>,
        week_start: &NaiveDate,
    ) -> Result<f32, String> {
        let week_end = *week_start + Duration::days(6);
        let week_start_str = week_start.format("%Y-%m-%d").to_string();
        let week_end_str = week_end.format("%Y-%m-%d").to_string();

        let mut reward = 0.0f32;
        let mut weight = 0.0f32;

        // Study progress
        let study_mins: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions WHERE date(started_at) BETWEEN ? AND ?",
        )
        .bind(&week_start_str)
        .bind(&week_end_str)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let target_mins: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(target_weekly_hours), 20) * 60 FROM courses",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(1200);

        let study_progress = (study_mins as f32 / target_mins as f32).min(1.5);
        reward += if study_progress >= 0.8 && study_progress <= 1.2 {
            1.0
        } else if study_progress < 0.8 {
            study_progress / 0.8
        } else {
            0.8
        };
        weight += 2.0;

        // Practice consistency
        let practice_days: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT date(logged_at)) FROM practice_logs WHERE date(logged_at) BETWEEN ? AND ?",
        )
        .bind(&week_start_str)
        .bind(&week_end_str)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        reward += (practice_days as f32 / 5.0).min(1.0);
        weight += 1.0;

        Ok(if weight > 0.0 { reward / weight } else { 0.5 })
    }

    /// Update daily rewards
    pub async fn update_daily_rewards(pool: &Pool<Sqlite>) -> Result<usize, String> {
        let yesterday = Local::now().date_naive() - Duration::days(1);
        let yesterday_str = yesterday.format("%Y-%m-%d").to_string();

        let daily_reward = Self::compute_daily_reward(pool, &yesterday).await?;

        let result = sqlx::query(
            "UPDATE agent_reward_log SET reward_daily = ? WHERE date(timestamp) = ? AND reward_daily IS NULL",
        )
        .bind(daily_reward)
        .bind(&yesterday_str)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() as usize)
    }

    /// Finalize rewards
    pub async fn finalize_rewards(pool: &Pool<Sqlite>) -> Result<usize, String> {
        let weights = Self::get_weights(pool).await?;

        let result = sqlx::query(
            r#"
            UPDATE agent_reward_log 
            SET reward_total = (
                ? * reward_immediate +
                ? * COALESCE(reward_daily, 0) +
                ? * COALESCE(reward_weekly, 0) +
                ? * COALESCE(reward_monthly, 0)
            ) / (
                ? +
                CASE WHEN reward_daily IS NOT NULL THEN ? ELSE 0 END +
                CASE WHEN reward_weekly IS NOT NULL THEN ? ELSE 0 END +
                CASE WHEN reward_monthly IS NOT NULL THEN ? ELSE 0 END
            )
            WHERE reward_total IS NULL
            AND (reward_daily IS NOT NULL OR reward_weekly IS NOT NULL)
            "#,
        )
        .bind(weights.immediate)
        .bind(weights.daily)
        .bind(weights.weekly)
        .bind(weights.monthly)
        .bind(weights.immediate)
        .bind(weights.daily)
        .bind(weights.weekly)
        .bind(weights.monthly)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() as usize)
    }
}
