//! Maximum Intelligence Productivity Agent
//!
//! Orchestrates all ML components to provide intelligent, adaptive recommendations
//! that optimize both immediate satisfaction and long-term goal achievement.
//!
//! Components:
//! - Semantic Memory (LanceDB): Finds similar past experiences
//! - Rich Context (50+ features): Comprehensive user state
//! - Hybrid Bandit (Linear/Neural): Action selection with UCB
//! - Multi-Scale Rewards: Balanced optimization across timescales
//! - Explainability: Feature importance and reasoning

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::ml::bandit_v2::{ActionSelection, BanditAction, HybridBandit};
use crate::ml::models::RewardEngine;
use crate::ml::rich_features::{RichContext, RichFeatureStore};
use crate::ml::semantic_memory::SemanticMemory;

/// Recommendation from the intelligence agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecommendation {
    /// Primary recommended action
    pub action: BanditAction,
    /// Expected reward (0-1)
    pub expected_reward: f32,
    /// Uncertainty in prediction
    pub uncertainty: f32,
    /// UCB score used for selection
    pub ucb_score: f32,
    /// Human-readable explanation
    pub explanation: String,
    /// Top contributing features
    pub top_features: Vec<FeatureContribution>,
    /// Similar past experiences
    pub similar_experiences: Vec<PastExperience>,
    /// Alternative recommendations
    pub alternatives: Vec<AlternativeAction>,
    /// Confidence level (low, medium, high)
    pub confidence_level: String,
    /// Recommendation ID (for feedback tracking)
    pub recommendation_id: Option<i64>,
}

/// Feature contribution for explainability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContribution {
    pub name: String,
    pub value: f32,
    pub contribution: f32,
    pub direction: String, // "positive" or "negative"
}

/// Similar past experience from semantic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastExperience {
    pub description: String,
    pub outcome: f32,
    pub similarity: f32,
    pub timestamp: String,
}

/// Alternative action recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeAction {
    pub action: BanditAction,
    pub expected_reward: f32,
    pub reason: String,
}

/// Agent statistics and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub mode: String,                    // "linear" or "neural"
    pub total_samples: i64,
    pub ready_for_neural: bool,
    pub memory_events: usize,
    pub avg_accuracy: f32,
    pub exploration_rate: f32,
    pub last_training: Option<String>,
}

/// The Maximum Intelligence Productivity Agent
pub struct IntelligenceAgent;

impl IntelligenceAgent {
    /// Get the top recommendation for the current context
    pub async fn get_recommendation(pool: &Pool<Sqlite>) -> Result<AgentRecommendation, String> {
        let recommendations = Self::get_recommendations(pool, 3).await?;
        recommendations
            .into_iter()
            .next()
            .ok_or_else(|| "No recommendations available".to_string())
    }

    /// Get top N recommendations
    pub async fn get_recommendations(
        pool: &Pool<Sqlite>,
        n: usize,
    ) -> Result<Vec<AgentRecommendation>, String> {
        // Capture current rich context
        let context = RichFeatureStore::capture_context(pool).await?;

        // Save context snapshot
        let context_id = RichFeatureStore::save_snapshot(pool, &context).await.ok();

        // Get similar past experiences from semantic memory
        let similar_experiences = Self::get_similar_experiences(&context).await.unwrap_or_default();

        // Update context with memory-based features
        let mut enriched_context = context.clone();
        if !similar_experiences.is_empty() {
            let avg_outcome: f32 = similar_experiences
                .iter()
                .map(|e| e.outcome * e.similarity)
                .sum::<f32>()
                / similar_experiences.iter().map(|e| e.similarity).sum::<f32>();
            enriched_context.similar_context_outcome = avg_outcome;
        }

        // Get action selections from bandit
        let selections = HybridBandit::select_top_actions(pool, &enriched_context, n + 2, None).await?;

        if selections.is_empty() {
            return Err("No actions available".to_string());
        }

        // Build recommendations
        let mut recommendations = Vec::new();

        for (i, selection) in selections.into_iter().enumerate() {
            if i >= n {
                break;
            }

            let explanation = Self::generate_explanation(&selection, &context, &similar_experiences);
            let confidence_level = Self::compute_confidence_level(&selection);

            let top_features: Vec<FeatureContribution> = selection
                .feature_contributions
                .iter()
                .map(|(name, contrib)| FeatureContribution {
                    name: name.clone(),
                    value: context.to_feature_vector()[RichContext::feature_names()
                        .iter()
                        .position(|&n| n == name)
                        .unwrap_or(0)],
                    contribution: *contrib,
                    direction: if *contrib >= 0.0 {
                        "positive".to_string()
                    } else {
                        "negative".to_string()
                    },
                })
                .collect();

            // Record recommendation
            let rec_id = Self::record_recommendation(
                pool,
                &selection.action.name,
                selection.expected_reward,
                selection.uncertainty,
                selection.ucb_score,
                context_id,
                &explanation,
            )
            .await
            .ok();

            recommendations.push(AgentRecommendation {
                action: selection.action,
                expected_reward: selection.expected_reward,
                uncertainty: selection.uncertainty,
                ucb_score: selection.ucb_score,
                explanation,
                top_features,
                similar_experiences: if i == 0 {
                    similar_experiences.clone()
                } else {
                    vec![]
                },
                alternatives: vec![], // Filled below
                confidence_level,
                recommendation_id: rec_id,
            });
        }

        // Add alternatives to first recommendation
        if recommendations.len() > 1 {
            let alternatives: Vec<AlternativeAction> = recommendations[1..]
                .iter()
                .map(|r| AlternativeAction {
                    action: r.action.clone(),
                    expected_reward: r.expected_reward,
                    reason: format!(
                        "Also good for {}: {}",
                        r.action.category, r.action.description
                    ),
                })
                .collect();
            recommendations[0].alternatives = alternatives;
        }

        Ok(recommendations)
    }

    /// Get similar past experiences from semantic memory
    async fn get_similar_experiences(context: &RichContext) -> Result<Vec<PastExperience>, String> {
        let memory = SemanticMemory::global().await?;

        let context_desc = context.to_description();
        let results = memory.search_similar(&context_desc, 5, None).await?;

        Ok(results
            .into_iter()
            .map(|r| PastExperience {
                description: r.event.content,
                outcome: r.event.outcome_score.unwrap_or(0.5),
                similarity: r.similarity,
                timestamp: r.event.timestamp,
            })
            .collect())
    }

    /// Generate human-readable explanation
    fn generate_explanation(
        selection: &ActionSelection,
        context: &RichContext,
        similar: &[PastExperience],
    ) -> String {
        let mut parts = Vec::new();

        // Primary reason based on top features
        if let Some((feature, _contribution)) = selection.feature_contributions.first() {
            let feature_reason = match feature.as_str() {
                "energy_level" => {
                    if context.energy_level > 0.7 {
                        "Your energy is high right now"
                    } else if context.energy_level < 0.4 {
                        "Your energy is low, so let's pick something manageable"
                    } else {
                        "Your energy level is moderate"
                    }
                }
                "hour_of_day" => {
                    if context.hour_of_day < 0.5 {
                        "Morning is a great time for focused work"
                    } else if context.hour_of_day < 0.75 {
                        "Afternoon is ideal for this activity"
                    } else {
                        "Evening is good for winding down"
                    }
                }
                "assignment_urgency" => {
                    if context.assignment_urgency > 0.7 {
                        "You have urgent deadlines approaching"
                    } else {
                        "Your workload is manageable"
                    }
                }
                "peak_focus_prob" => {
                    if context.peak_focus_prob > 0.6 {
                        "This is typically your peak focus time"
                    } else {
                        "This time is good for lighter tasks"
                    }
                }
                "recovery_need" => {
                    if context.recovery_need > 0.6 {
                        "You could use some recovery time"
                    } else {
                        "You're well-rested"
                    }
                }
                "streak_days" => {
                    if context.streak_days > 0.3 {
                        "You're on a great streak, keep it going!"
                    } else {
                        "Let's build some momentum"
                    }
                }
                _ => "Based on your current context",
            };
            parts.push(feature_reason.to_string());
        }

        // Add action-specific context
        let action_context = match selection.action.category.as_str() {
            "productivity" => {
                if context.pomodoros_today < 0.3 {
                    "You haven't done much focused work yet today."
                } else {
                    ""
                }
            }
            "physical" => {
                if context.hours_since_workout > 0.5 {
                    "It's been a while since your last workout."
                } else {
                    ""
                }
            }
            "wellness" => {
                if context.hours_since_checkin > 0.5 {
                    "A quick check-in would help track your progress."
                } else {
                    ""
                }
            }
            _ => "",
        };
        if !action_context.is_empty() {
            parts.push(action_context.to_string());
        }

        // Add memory-based reasoning
        if !similar.is_empty() {
            let avg_outcome: f32 = similar.iter().map(|s| s.outcome).sum::<f32>() / similar.len() as f32;
            if avg_outcome > 0.7 {
                parts.push("Similar situations in the past led to good outcomes.".to_string());
            }
        }

        if parts.is_empty() {
            format!(
                "Recommended: {}. {}",
                selection.action.name.replace("_", " "),
                selection.action.description
            )
        } else {
            format!("{}. {}", parts.join(" "), selection.action.description)
        }
    }

    /// Compute confidence level based on uncertainty
    fn compute_confidence_level(selection: &ActionSelection) -> String {
        if selection.uncertainty < 0.2 {
            "high".to_string()
        } else if selection.uncertainty < 0.5 {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    /// Record a recommendation for tracking
    async fn record_recommendation(
        pool: &Pool<Sqlite>,
        action: &str,
        expected_reward: f32,
        uncertainty: f32,
        ucb_score: f32,
        context_id: Option<i64>,
        explanation: &str,
    ) -> Result<i64, String> {
        let result = sqlx::query(
            r#"
            INSERT INTO agent_recommendations 
            (action_recommended, confidence, uncertainty, ucb_score, context_id, explanation_json)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(action)
        .bind(expected_reward)
        .bind(uncertainty)
        .bind(ucb_score)
        .bind(context_id)
        .bind(serde_json::json!({"text": explanation}).to_string())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }

    /// Record feedback on a recommendation
    pub async fn record_feedback(
        pool: &Pool<Sqlite>,
        recommendation_id: i64,
        accepted: bool,
        alternative_chosen: Option<&str>,
        feedback_score: Option<i32>,
        outcome_score: Option<f32>,
    ) -> Result<(), String> {
        // Get the recommendation details
        let rec: Option<(String, Option<Vec<u8>>)> = sqlx::query_as(
            r#"
            SELECT r.action_recommended, c.context_features 
            FROM agent_recommendations r
            LEFT JOIN (
                SELECT id, NULL as context_features FROM agent_rich_context
            ) c ON r.context_id = c.id
            WHERE r.id = ?
            "#,
        )
        .bind(recommendation_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        // Update recommendation record
        sqlx::query(
            r#"
            UPDATE agent_recommendations 
            SET was_accepted = ?, alternative_chosen = ?, feedback_score = ?, outcome_score = ?
            WHERE id = ?
            "#,
        )
        .bind(accepted)
        .bind(alternative_chosen)
        .bind(feedback_score)
        .bind(outcome_score)
        .bind(recommendation_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        // If we have outcome, update the bandit
        if let (Some((action_name, _context_bytes)), Some(outcome)) = (rec, outcome_score) {
            // Get current context (or reconstruct from bytes)
            let context = RichFeatureStore::capture_context(pool).await.unwrap_or_default();

            // Update bandit with observed reward
            HybridBandit::update(pool, &action_name, &context, outcome as f64).await?;

            // Log for multi-scale reward computation
            let immediate_reward = RewardEngine::compute_immediate_reward(
                accepted,
                feedback_score,
                outcome > 0.7,
                None,
            );
            HybridBandit::log_reward(pool, &action_name, &context, immediate_reward, "explicit")
                .await?;
        }

        Ok(())
    }

    /// Record that user completed an action (for memory and learning)
    pub async fn record_action_completed(
        pool: &Pool<Sqlite>,
        action_type: &str,
        description: &str,
        outcome_score: f32,
        metadata: Option<serde_json::Value>,
    ) -> Result<i64, String> {
        // Add to semantic memory
        let memory = SemanticMemory::global().await?;
        let context = RichFeatureStore::capture_context(pool).await?;

        let content = format!("{}: {}", action_type, description);
        let metadata_json = metadata.map(|m| m.to_string());

        let event_id = sqlx::query(
            r#"
            INSERT INTO agent_memory_events 
            (event_type, content, metadata_json, outcome_immediate)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(action_type)
        .bind(&content)
        .bind(&metadata_json)
        .bind(outcome_score)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();

        // Add to vector store
        let _ = memory
            .add_event(
                event_id,
                &chrono::Local::now().to_rfc3339(),
                action_type,
                &content,
                metadata_json.as_deref(),
                Some(outcome_score),
            )
            .await;

        // Update bandit with the action and outcome
        if let Some(action_name) = Self::map_event_to_action(action_type) {
            HybridBandit::update(pool, action_name, &context, outcome_score as f64).await?;
        }

        Ok(event_id)
    }

    /// Map event type to bandit action
    fn map_event_to_action(event_type: &str) -> Option<&'static str> {
        match event_type {
            "study_session" | "pomodoro" => Some("start_pomodoro"),
            "workout" => Some("do_workout"),
            "checkin" => Some("do_checkin"),
            "skill_practice" => Some("practice_skill"),
            "assignment_completed" => Some("tackle_assignment"),
            "break" => Some("take_break"),
            "weekly_review" => Some("weekly_review"),
            _ => None,
        }
    }

    /// Get agent status and statistics
    pub async fn get_status(pool: &Pool<Sqlite>) -> Result<AgentStatus, String> {
        let mode = HybridBandit::get_mode(pool).await?;
        let total_samples = HybridBandit::total_samples(pool).await?;
        let ready = HybridBandit::ready_for_neural(pool).await?;

        let memory_events = match SemanticMemory::global().await {
            Ok(mem) => mem.count_events().await.unwrap_or(0),
            Err(_) => 0,
        };

        // Compute average accuracy from recent recommendations
        let accuracy: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT AVG(CASE WHEN was_accepted = 1 THEN 1.0 ELSE 0.0 END)
            FROM agent_recommendations
            WHERE timestamp >= datetime('now', '-7 days')
            "#,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let exploration_rate: f64 = sqlx::query_scalar(
            "SELECT CAST(value_json AS REAL) FROM agent_state WHERE key = 'exploration_rate'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or(0.2);

        let last_training: Option<String> = sqlx::query_scalar(
            "SELECT value_json FROM agent_state WHERE key = 'last_neural_training'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .and_then(|s: String| serde_json::from_str(&s).ok());

        Ok(AgentStatus {
            mode,
            total_samples,
            ready_for_neural: ready,
            memory_events,
            avg_accuracy: accuracy.unwrap_or(0.5) as f32,
            exploration_rate: exploration_rate as f32,
            last_training,
        })
    }

    /// Perform daily maintenance (update rewards, train if needed)
    pub async fn daily_maintenance(pool: &Pool<Sqlite>) -> Result<(), String> {
        // Update daily rewards from yesterday
        RewardEngine::update_daily_rewards(pool).await?;

        // Finalize any complete reward records
        RewardEngine::finalize_rewards(pool).await?;

        // Check if we should switch to neural mode
        if HybridBandit::ready_for_neural(pool).await? {
            let current_mode = HybridBandit::get_mode(pool).await?;
            if current_mode == "linear" {
                // Log that we're ready for neural upgrade
                // (actual training would be done offline)
                log::info!("Agent ready for neural upgrade with {} samples", 
                    HybridBandit::total_samples(pool).await?);
            }
        }

        Ok(())
    }

    /// Get Big 3 goals for today
    pub async fn get_big_three(pool: &Pool<Sqlite>) -> Result<Vec<BigThreeGoal>, String> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let goals: Vec<(i64, i32, String, Option<String>, Option<String>, bool)> = sqlx::query_as(
            r#"
            SELECT id, priority, title, description, category, is_completed
            FROM agent_big_three
            WHERE date = ?
            ORDER BY priority
            "#,
        )
        .bind(&today)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(goals
            .into_iter()
            .map(|(id, priority, title, description, category, is_completed)| BigThreeGoal {
                id,
                priority,
                title,
                description,
                category,
                is_completed,
            })
            .collect())
    }

    /// Set Big 3 goals for today
    pub async fn set_big_three(
        pool: &Pool<Sqlite>,
        goals: Vec<(String, Option<String>, Option<String>)>,
    ) -> Result<(), String> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        // Clear existing goals for today
        sqlx::query("DELETE FROM agent_big_three WHERE date = ?")
            .bind(&today)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Insert new goals
        for (i, (title, description, category)) in goals.into_iter().take(3).enumerate() {
            sqlx::query(
                r#"
                INSERT INTO agent_big_three (date, priority, title, description, category)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(&today)
            .bind((i + 1) as i32)
            .bind(&title)
            .bind(&description)
            .bind(&category)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Complete a Big 3 goal
    pub async fn complete_big_three(
        pool: &Pool<Sqlite>,
        goal_id: i64,
        satisfaction: Option<i32>,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE agent_big_three 
            SET is_completed = 1, 
                completed_at = datetime('now'),
                satisfaction_rating = ?
            WHERE id = ?
            "#,
        )
        .bind(satisfaction)
        .bind(goal_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

/// Big 3 goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigThreeGoal {
    pub id: i64,
    pub priority: i32,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_completed: bool,
}
