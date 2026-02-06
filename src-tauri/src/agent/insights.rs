//! Agent Insights - Adaptive, ML-powered insight generation
//!
//! Combines rule-based fallbacks with learned patterns and
//! contextual bandit selection for personalized recommendations.

use tauri::State;
use serde_json;

use crate::{DbState, error::ApiError};
use crate::ml::{FeatureStore, ContextualBandit, PatternMiner, UserProfile};
use crate::ml::models::AdaptiveInsight;

#[derive(Debug, serde::Serialize)]
pub struct Insight {
    pub icon: String,
    pub message: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insight_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arm_name: Option<String>,
}

/// Get adaptive insights using ML-powered selection
#[tauri::command]
pub async fn get_insights(state: State<'_, DbState>) -> Result<Vec<Insight>, ApiError> {
    get_insights_for_pool(&state.0).await
}

async fn get_insights_for_pool(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<Insight>, ApiError> {
    let mut insights = Vec::new();

    // Capture current context
    let context = FeatureStore::capture_context(pool)
        .await
        .map_err(ApiError::internal)?;

    // Save context snapshot for pattern mining (don't fail if this fails)
    let _ = FeatureStore::save_snapshot(pool, &context).await;

    // Get top bandit arms to show
    let selected_arms = ContextualBandit::select_top_arms(pool, &context, 3)
        .await
        .map_err(ApiError::internal)?;

    // Get active patterns for context-aware insights
    let patterns = PatternMiner::get_active_patterns(pool)
        .await
        .map_err(ApiError::internal)?;

    // Generate insights from selected arms
    for arm in &selected_arms {
        if let Some(insight) = generate_insight_for_arm(pool, &arm.arm_name, &context).await {
            // Record that this insight was shown
            let context_json = serde_json::to_string(&context).unwrap_or_default();
            let insight_id = ContextualBandit::record_insight_shown(
                pool,
                &insight.category,
                &arm.arm_name,
                &context_json,
            )
            .await
            .ok();

            insights.push(Insight {
                icon: insight.icon,
                message: insight.message,
                category: insight.category,
                confidence: Some(arm.expected_value()),
                insight_id,
                arm_name: Some(arm.arm_name.clone()),
            });
        }
    }

    // Add pattern-based insights
    for pattern in patterns.iter().take(2) {
        if let Some(msg) = PatternMiner::pattern_to_insight(pattern, &context) {
            // Avoid duplicate insights
            if !insights.iter().any(|i| i.message.contains(&msg[..20.min(msg.len())])) {
                insights.push(Insight {
                    icon: "🔍".to_string(),
                    message: msg,
                    category: pattern.pattern_type.clone(),
                    confidence: Some(pattern.confidence),
                    insight_id: None,
                    arm_name: None,
                });
            }
        }
    }

    // Fallback to rule-based if no ML insights
    if insights.is_empty() {
        insights = get_fallback_insights(pool).await?;
    }

    // If still no insights, default message
    if insights.is_empty() {
        insights.push(Insight {
            icon: "✨".to_string(),
            message: "You're on track! Keep up the great work across all areas.".to_string(),
            category: "general".to_string(),
            confidence: None,
            insight_id: None,
            arm_name: None,
        });
    }

    Ok(insights)
}

/// Generate an insight for a specific bandit arm
async fn generate_insight_for_arm(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    arm_name: &str,
    context: &crate::ml::models::Context,
) -> Option<AdaptiveInsight> {
    match arm_name {
        "remind_checkin" => {
            let has_checkin_today: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM check_ins WHERE date(checked_in_at) = date('now')"
            )
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if has_checkin_today == 0 {
                Some(AdaptiveInsight {
                    icon: "📝".to_string(),
                    message: "Start your day with a quick check-in to track mood and energy.".to_string(),
                    category: "wellness".to_string(),
                    arm_name: arm_name.to_string(),
                    confidence: 0.8,
                    context_hash: String::new(),
                    insight_id: None,
                })
            } else {
                None
            }
        },
        
        "suggest_pomodoro" => {
            if !context.session_in_progress {
                let optimal = (9..=11).contains(&context.hour_of_day) || (14..=17).contains(&context.hour_of_day);
                if optimal {
                    Some(AdaptiveInsight {
                        icon: "🍅".to_string(),
                        message: "Great time for focused work! Start a Pomodoro session.".to_string(),
                        category: "productivity".to_string(),
                        arm_name: arm_name.to_string(),
                        confidence: 0.85,
                        context_hash: String::new(),
                        insight_id: None,
                    })
                } else {
                    Some(AdaptiveInsight {
                        icon: "🍅".to_string(),
                        message: "Ready to focus? Start a Pomodoro to build momentum.".to_string(),
                        category: "productivity".to_string(),
                        arm_name: arm_name.to_string(),
                        confidence: 0.6,
                        context_hash: String::new(),
                        insight_id: None,
                    })
                }
            } else {
                None
            }
        },
        
        "celebrate_streak" => {
            if context.current_streak_days >= 3 {
                Some(AdaptiveInsight {
                    icon: "🔥".to_string(),
                    message: format!("{} day streak! You're building great habits.", context.current_streak_days),
                    category: "motivation".to_string(),
                    arm_name: arm_name.to_string(),
                    confidence: 0.9,
                    context_hash: String::new(),
                    insight_id: None,
                })
            } else {
                None
            }
        },
        
        "warn_overdue" => {
            if context.overdue_assignments > 0 {
                Some(AdaptiveInsight {
                    icon: "⚠️".to_string(),
                    message: format!("You have {} overdue assignment{}. Consider prioritizing these today.", 
                        context.overdue_assignments, if context.overdue_assignments == 1 { "" } else { "s" }),
                    category: "academic".to_string(),
                    arm_name: arm_name.to_string(),
                    confidence: 0.95,
                    context_hash: String::new(),
                    insight_id: None,
                })
            } else {
                None
            }
        },
        
        "recommend_break" => {
            if context.session_in_progress && context.recent_study_minutes > 90 {
                Some(AdaptiveInsight {
                    icon: "☕".to_string(),
                    message: "You've been working for a while. A short break could boost your focus!".to_string(),
                    category: "wellness".to_string(),
                    arm_name: arm_name.to_string(),
                    confidence: 0.8,
                    context_hash: String::new(),
                    insight_id: None,
                })
            } else {
                None
            }
        },
        
        "suggest_workout" => {
            if context.recent_workout_count == 0 {
                let morning = context.hour_of_day < 12;
                Some(AdaptiveInsight {
                    icon: "💪".to_string(),
                    message: if morning {
                        "A morning workout could boost your energy and focus for the day ahead!".to_string()
                    } else {
                        "No workout today yet. Even a short session can improve your mood!".to_string()
                    },
                    category: "physical".to_string(),
                    arm_name: arm_name.to_string(),
                    confidence: 0.75,
                    context_hash: String::new(),
                    insight_id: None,
                })
            } else {
                None
            }
        },
        
        "practice_reminder" => {
            let skills_need_practice: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM skills s
                WHERE NOT EXISTS (
                    SELECT 1 FROM practice_logs p 
                    WHERE p.skill_id = s.id 
                    AND p.logged_at >= date('now', '-6 days')
                )
                "#
            )
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if skills_need_practice > 0 {
                Some(AdaptiveInsight {
                    icon: "🎯".to_string(),
                    message: format!("{} skill{} haven't been practiced this week. Time for some deliberate practice!", 
                        skills_need_practice, if skills_need_practice == 1 { "" } else { "s" }),
                    category: "skills".to_string(),
                    arm_name: arm_name.to_string(),
                    confidence: 0.7,
                    context_hash: String::new(),
                    insight_id: None,
                })
            } else {
                None
            }
        },
        
        "productivity_tip" => {
            // Context-aware productivity tips
            if let Some(energy) = context.energy {
                if energy < 5 {
                    Some(AdaptiveInsight {
                        icon: "💡".to_string(),
                        message: "Low energy detected. Try a quick walk or some light stretching to recharge.".to_string(),
                        category: "general".to_string(),
                        arm_name: arm_name.to_string(),
                        confidence: 0.65,
                        context_hash: String::new(),
                        insight_id: None,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        },
        
        "weekly_reflection" => {
            // Suggest weekly review on Sundays or if none this week
            let day_is_sunday = context.day_of_week == 0;
            let has_review_this_week: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM weekly_reviews WHERE week_start >= date('now', 'weekday 0', '-7 days')"
            )
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if (day_is_sunday || has_review_this_week == 0) && context.hour_of_day >= 17 {
                Some(AdaptiveInsight {
                    icon: "📊".to_string(),
                    message: "Time for your weekly reflection. Review wins and areas for improvement.".to_string(),
                    category: "reflection".to_string(),
                    arm_name: arm_name.to_string(),
                    confidence: 0.7,
                    context_hash: String::new(),
                    insight_id: None,
                })
            } else {
                None
            }
        },
        
        _ => None,
    }
}

/// Fallback to simple rule-based insights when ML hasn't learned enough
async fn get_fallback_insights(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<Insight>, ApiError> {
    let mut insights = Vec::new();

    // Check for missing check-in today
    let has_checkin_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM check_ins WHERE date(checked_in_at) = date('now')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if has_checkin_today == 0 {
        insights.push(Insight {
            icon: "📝".to_string(),
            message: "Start your day with a quick check-in to track mood and energy.".to_string(),
            category: "wellness".to_string(),
            confidence: None,
            insight_id: None,
            arm_name: None,
        });
    }

    // Check for overdue assignments
    let overdue_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM assignments WHERE is_completed = 0 AND due_date < date('now')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if overdue_count > 0 {
        insights.push(Insight {
            icon: "⚠️".to_string(),
            message: format!("You have {} overdue assignment{}. Consider prioritizing these today.", 
                overdue_count, if overdue_count == 1 { "" } else { "s" }),
            category: "academic".to_string(),
            confidence: None,
            insight_id: None,
            arm_name: None,
        });
    }

    Ok(insights)
}

/// Record feedback on an insight (called from frontend)
#[tauri::command]
pub async fn record_insight_feedback(
    state: State<'_, DbState>,
    insight_id: i64,
    acted_on: bool,
    feedback_score: Option<i32>,
) -> Result<(), ApiError> {
    let pool = &state.0;
    ContextualBandit::record_feedback(pool, insight_id, acted_on, feedback_score)
        .await
        .map_err(ApiError::internal)
}

/// Trigger pattern mining (can be called periodically or on-demand)
#[tauri::command]
pub async fn run_pattern_analysis(state: State<'_, DbState>) -> Result<usize, ApiError> {
    let pool = &state.0;
    
    // Run pattern mining
    let patterns_found = PatternMiner::discover_and_save_patterns(pool)
        .await
        .map_err(ApiError::internal)?;
    
    // Update user profile
    UserProfile::learn_all(pool)
        .await
        .map_err(ApiError::internal)?;
    
    Ok(patterns_found)
}

/// Get user profile for display
#[tauri::command]
pub async fn get_user_profile(state: State<'_, DbState>) -> Result<Vec<crate::ml::models::ProfileDimension>, ApiError> {
    let pool = &state.0;
    UserProfile::get_all_dimensions(pool)
        .await
        .map_err(ApiError::internal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn setup_pool_with_migrations() -> sqlx::Pool<sqlx::Sqlite> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(false);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to connect to in-memory DB");

        crate::db::migrations::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }


    #[tokio::test]
    async fn get_insights_returns_error_when_capture_context_fails() {
        let pool = setup_pool_with_migrations().await;
        sqlx::query("DROP TABLE check_ins")
            .execute(&pool)
            .await
            .expect("Failed to drop check_ins table");
        let result = get_insights_for_pool(&pool).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::Internal);
        assert!(
            err.message.contains("check_ins") || err.message.contains("no such table"),
            "error message: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn get_insights_returns_error_when_bandit_selection_fails() {
        let pool = setup_pool_with_migrations().await;
        sqlx::query("DROP TABLE agent_bandit_arms")
            .execute(&pool)
            .await
            .expect("Failed to drop bandit arms table");
        let result = get_insights_for_pool(&pool).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::Internal);
        assert!(
            err.message.contains("agent_bandit_arms") || err.message.contains("no such table"),
            "error message: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn get_insights_returns_error_when_pattern_fetch_fails() {
        let pool = setup_pool_with_migrations().await;
        sqlx::query("DROP TABLE agent_patterns")
            .execute(&pool)
            .await
            .expect("Failed to drop patterns table");
        let result = get_insights_for_pool(&pool).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::Internal);
        assert!(
            err.message.contains("agent_patterns") || err.message.contains("no such table"),
            "error message: {}",
            err.message
        );
    }
}
