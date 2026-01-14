//! Intelligence Agent Commands
//!
//! Tauri commands for the Maximum Intelligence Productivity Agent.
//! Exposes recommendation, feedback, and status APIs to the frontend.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::agent::{
    AgentRecommendation, AgentStatus, BigThreeGoal, IntelligenceAgent,
};
use crate::ml::{RichContext, RichFeatureStore};
use crate::DbState;

/// Get top recommendations from the intelligence agent
#[tauri::command]
pub async fn get_agent_recommendations(
    state: State<'_, DbState>,
    count: Option<usize>,
) -> Result<Vec<AgentRecommendation>, String> {
    let pool = &state.0;
    let n = count.unwrap_or(3);
    IntelligenceAgent::get_recommendations(pool, n).await
}

/// Get the top recommendation
#[tauri::command]
pub async fn get_agent_recommendation(
    state: State<'_, DbState>,
) -> Result<AgentRecommendation, String> {
    let pool = &state.0;
    IntelligenceAgent::get_recommendation(pool).await
}

/// Record feedback on a recommendation
#[tauri::command]
pub async fn record_recommendation_feedback(
    state: State<'_, DbState>,
    recommendation_id: i64,
    accepted: bool,
    alternative_chosen: Option<String>,
    feedback_score: Option<i32>,
    outcome_score: Option<f32>,
) -> Result<(), String> {
    let pool = &state.0;
    IntelligenceAgent::record_feedback(
        pool,
        recommendation_id,
        accepted,
        alternative_chosen.as_deref(),
        feedback_score,
        outcome_score,
    )
    .await
}

/// Record a completed action for learning
#[tauri::command]
pub async fn record_action_completed(
    state: State<'_, DbState>,
    action_type: String,
    description: String,
    outcome_score: f32,
    metadata: Option<serde_json::Value>,
) -> Result<i64, String> {
    let pool = &state.0;
    IntelligenceAgent::record_action_completed(pool, &action_type, &description, outcome_score, metadata)
        .await
}

/// Get agent status and statistics
#[tauri::command]
pub async fn get_agent_status(state: State<'_, DbState>) -> Result<AgentStatus, String> {
    let pool = &state.0;
    IntelligenceAgent::get_status(pool).await
}

/// Get current rich context (50+ features)
#[tauri::command]
pub async fn get_rich_context(state: State<'_, DbState>) -> Result<RichContext, String> {
    let pool = &state.0;
    RichFeatureStore::capture_context(pool).await
}

/// Get Big 3 goals for today
#[tauri::command]
pub async fn get_big_three(state: State<'_, DbState>) -> Result<Vec<BigThreeGoal>, String> {
    let pool = &state.0;
    IntelligenceAgent::get_big_three(pool).await
}

/// Set Big 3 goals for today
#[derive(Debug, Deserialize)]
pub struct BigThreeInput {
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[tauri::command]
pub async fn set_big_three(
    state: State<'_, DbState>,
    goals: Vec<BigThreeInput>,
) -> Result<(), String> {
    let pool = &state.0;
    let goals_tuple: Vec<_> = goals
        .into_iter()
        .map(|g| (g.title, g.description, g.category))
        .collect();
    IntelligenceAgent::set_big_three(pool, goals_tuple).await
}

/// Complete a Big 3 goal
#[tauri::command]
pub async fn complete_big_three(
    state: State<'_, DbState>,
    goal_id: i64,
    satisfaction: Option<i32>,
) -> Result<(), String> {
    let pool = &state.0;
    IntelligenceAgent::complete_big_three(pool, goal_id, satisfaction).await
}

/// Run daily maintenance (reward updates, cleanup)
#[tauri::command]
pub async fn run_agent_maintenance(state: State<'_, DbState>) -> Result<(), String> {
    let pool = &state.0;
    IntelligenceAgent::daily_maintenance(pool).await
}

/// Get feature names for UI display
#[tauri::command]
pub fn get_feature_names() -> Vec<String> {
    RichContext::feature_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// Search semantic memory for similar experiences
#[tauri::command]
pub async fn search_similar_experiences(
    _state: State<'_, DbState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SimilarExperience>, String> {
    use crate::ml::SemanticMemory;

    let memory = SemanticMemory::global().await?;
    let results = memory.search_similar(&query, limit.unwrap_or(5), None).await?;

    Ok(results
        .into_iter()
        .map(|r| SimilarExperience {
            content: r.event.content,
            event_type: r.event.event_type,
            timestamp: r.event.timestamp,
            outcome: r.event.outcome_score,
            similarity: r.similarity,
        })
        .collect())
}

#[derive(Debug, Serialize)]
pub struct SimilarExperience {
    pub content: String,
    pub event_type: String,
    pub timestamp: String,
    pub outcome: Option<f32>,
    pub similarity: f32,
}

/// Update reward weights (for tuning)
#[tauri::command]
pub async fn set_reward_weights(
    state: State<'_, DbState>,
    immediate: f32,
    daily: f32,
    weekly: f32,
    monthly: f32,
) -> Result<(), String> {
    let pool = &state.0;
    
    // Validate weights sum to ~1.0
    let total = immediate + daily + weekly + monthly;
    if (total - 1.0).abs() > 0.01 {
        return Err("Weights must sum to 1.0".to_string());
    }

    let weights = serde_json::json!({
        "immediate": immediate,
        "daily": daily,
        "weekly": weekly,
        "monthly": monthly
    });

    sqlx::query(
        "UPDATE agent_state SET value_json = ?, updated_at = datetime('now') WHERE key = 'reward_weights'",
    )
    .bind(weights.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Set exploration rate (beta for UCB)
#[tauri::command]
pub async fn set_exploration_rate(
    state: State<'_, DbState>,
    rate: f32,
) -> Result<(), String> {
    let pool = &state.0;
    
    if rate < 0.0 || rate > 5.0 {
        return Err("Exploration rate must be between 0 and 5".to_string());
    }

    sqlx::query(
        "UPDATE agent_state SET value_json = ?, updated_at = datetime('now') WHERE key = 'exploration_rate'",
    )
    .bind(rate.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
