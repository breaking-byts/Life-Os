use tauri::State;
use crate::DbState;

#[derive(Debug, serde::Serialize)]
pub struct Insight {
    pub icon: String,
    pub message: String,
    pub category: String,
}

#[tauri::command]
pub async fn get_insights(state: State<'_, DbState>) -> Result<Vec<Insight>, String> {
    let pool = &state.0;
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
        });
    }

    // Check for no study sessions this week
    let study_sessions_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE session_type = 'study' AND started_at >= date('now', '-6 days')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if study_sessions_week == 0 {
        insights.push(Insight {
            icon: "📚".to_string(),
            message: "No study sessions logged this week. Start a Pomodoro to build momentum!".to_string(),
            category: "academic".to_string(),
        });
    }

    // Check for consistent workouts (positive insight)
    let workout_streak: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT date(logged_at)) FROM workouts WHERE logged_at >= date('now', '-6 days')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if workout_streak >= 3 {
        insights.push(Insight {
            icon: "🔥".to_string(),
            message: format!("Great job! {} workout days this week. Keep the momentum!", workout_streak),
            category: "fitness".to_string(),
        });
    }

    // Check for skills needing practice
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
        insights.push(Insight {
            icon: "🎯".to_string(),
            message: format!("{} skill{} haven't been practiced this week. Time for some deliberate practice!", 
                skills_need_practice, if skills_need_practice == 1 { "" } else { "s" }),
            category: "skills".to_string(),
        });
    }

    // If no insights, add a default positive message
    if insights.is_empty() {
        insights.push(Insight {
            icon: "✨".to_string(),
            message: "You're on track! Keep up the great work across all areas.".to_string(),
            category: "general".to_string(),
        });
    }

    Ok(insights)
}
