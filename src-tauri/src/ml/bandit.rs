//! Contextual Bandit - Thompson Sampling for action selection
//!
//! Uses Beta distributions to balance exploration vs exploitation,
//! learning which insights/nudges work best in different contexts.

#![allow(dead_code)] // Legacy bandit for backwards compatibility

use sqlx::{Pool, Sqlite};
use rand::prelude::*;
use rand_distr::Beta;

use super::models::{BanditArm, Context};

/// Contextual bandit using Thompson Sampling
pub struct ContextualBandit;

impl ContextualBandit {
    /// Get all enabled bandit arms
    pub async fn get_arms(pool: &Pool<Sqlite>) -> Result<Vec<BanditArm>, String> {
        sqlx::query_as::<_, BanditArm>(
            "SELECT * FROM agent_bandit_arms WHERE is_enabled = 1"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    /// Select the best arm using Thompson Sampling
    /// 
    /// Samples from each arm's Beta distribution and picks the highest sample.
    /// This naturally balances exploration (uncertain arms) with exploitation (known good arms).
    pub async fn select_arm(
        pool: &Pool<Sqlite>,
        _context: &Context,  // TODO: Use context for contextual bandit
        category_filter: Option<&str>,
    ) -> Result<Option<BanditArm>, String> {
        let arms = if let Some(category) = category_filter {
            sqlx::query_as::<_, BanditArm>(
                "SELECT * FROM agent_bandit_arms WHERE is_enabled = 1 AND category = ?"
            )
            .bind(category)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
        } else {
            Self::get_arms(pool).await?
        };

        if arms.is_empty() {
            return Ok(None);
        }

        let mut rng = thread_rng();
        let mut best_arm: Option<&BanditArm> = None;
        let mut best_sample = f64::NEG_INFINITY;

        for arm in &arms {
            // Sample from Beta distribution for this arm
            let dist = Beta::new(arm.alpha, arm.beta)
                .map_err(|e| format!("Invalid beta params: {}", e))?;
            let sample = dist.sample(&mut rng);

            if sample > best_sample {
                best_sample = sample;
                best_arm = Some(arm);
            }
        }

        Ok(best_arm.cloned())
    }

    /// Select multiple arms (for showing multiple insights)
    pub async fn select_top_arms(
        pool: &Pool<Sqlite>,
        context: &Context,
        n: usize,
    ) -> Result<Vec<BanditArm>, String> {
        let arms = Self::get_arms(pool).await?;
        if arms.is_empty() {
            return Ok(vec![]);
        }

        let mut rng = thread_rng();
        let mut scored_arms: Vec<(BanditArm, f64)> = vec![];

        for arm in arms {
            let dist = Beta::new(arm.alpha, arm.beta)
                .map_err(|e| format!("Invalid beta params: {}", e))?;
            let sample = dist.sample(&mut rng);
            
            // Apply context modifiers
            let adjusted_sample = Self::apply_context_modifier(sample, &arm, context);
            scored_arms.push((arm, adjusted_sample));
        }

        // Sort by score descending and take top n
        scored_arms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(scored_arms.into_iter().take(n).map(|(arm, _)| arm).collect())
    }

    /// Apply context-based modifiers to arm scores
    fn apply_context_modifier(base_score: f64, arm: &BanditArm, ctx: &Context) -> f64 {
        let mut modifier = 1.0;

        // Boost workout suggestions if no recent workout
        if arm.arm_name == "suggest_workout" && ctx.recent_workout_count == 0 {
            modifier *= 1.3;
        }

        // Boost study suggestions during typical study hours
        if arm.arm_name == "suggest_pomodoro" && (9..=21).contains(&ctx.hour_of_day) {
            modifier *= 1.2;
        }

        // Boost overdue warnings if there are overdue assignments
        if arm.arm_name == "warn_overdue" && ctx.overdue_assignments > 0 {
            modifier *= 1.5;
        }

        // Reduce celebration if user seems low energy
        if arm.arm_name == "celebrate_streak" {
            if let Some(energy) = ctx.energy {
                if energy < 4 {
                    modifier *= 0.5;  // Less celebration when tired
                }
            }
        }

        // Boost break reminders if in a long session
        if arm.arm_name == "recommend_break" && ctx.session_in_progress {
            modifier *= 1.4;
        }

        base_score * modifier
    }

    /// Update arm statistics after receiving feedback
    pub async fn update_arm(
        pool: &Pool<Sqlite>,
        arm_name: &str,
        reward: f64,  // 0.0 to 1.0
    ) -> Result<(), String> {
        // Update alpha (success) and beta (failure) based on reward
        // Binary approximation: treat reward >= 0.5 as success
        let (alpha_inc, beta_inc) = if reward >= 0.5 {
            (1.0, 0.0)
        } else {
            (0.0, 1.0)
        };

        sqlx::query(
            r#"
            UPDATE agent_bandit_arms 
            SET alpha = alpha + ?,
                beta = beta + ?,
                total_pulls = total_pulls + 1,
                total_reward = total_reward + ?,
                last_pulled = datetime('now')
            WHERE arm_name = ?
            "#
        )
        .bind(alpha_inc)
        .bind(beta_inc)
        .bind(reward)
        .bind(arm_name)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Record that an insight was shown (for tracking)
    pub async fn record_insight_shown(
        pool: &Pool<Sqlite>,
        insight_type: &str,
        arm_name: &str,
        context_json: &str,
    ) -> Result<i64, String> {
        let result = sqlx::query(
            r#"
            INSERT INTO agent_insights (insight_type, context_json, arm_index)
            SELECT ?, ?, id FROM agent_bandit_arms WHERE arm_name = ?
            "#
        )
        .bind(insight_type)
        .bind(context_json)
        .bind(arm_name)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }

    /// Record feedback on an insight
    pub async fn record_feedback(
        pool: &Pool<Sqlite>,
        insight_id: i64,
        acted_on: bool,
        feedback_score: Option<i32>,
    ) -> Result<(), String> {
        let reward = if acted_on { 1.0 } else if feedback_score.map(|s| s > 0).unwrap_or(false) { 0.7 } else { 0.0 };

        // Update insight record
        sqlx::query(
            r#"
            UPDATE agent_insights 
            SET acted_on = ?, feedback_score = ?, dismissed_at = datetime('now')
            WHERE id = ?
            "#
        )
        .bind(acted_on)
        .bind(feedback_score)
        .bind(insight_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        // Get the arm name for this insight and update it
        let arm_name: Option<String> = sqlx::query_scalar(
            r#"
            SELECT ba.arm_name FROM agent_insights ai
            JOIN agent_bandit_arms ba ON ai.arm_index = ba.id
            WHERE ai.id = ?
            "#
        )
        .bind(insight_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(name) = arm_name {
            Self::update_arm(pool, &name, reward).await?;
        }

        Ok(())
    }
}
