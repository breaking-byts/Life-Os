//! Rich Context Features (50+ dimensions)
//!
//! Captures comprehensive user context for ML decision-making.
//! This is the feature vector used by the contextual bandit.

#![allow(dead_code)] // Serialization methods for future use

use chrono::{Datelike, Local, NaiveDate, Timelike};
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

/// Number of features in the rich context vector
pub const FEATURE_DIM: usize = 50;

/// Rich context with 50+ dimensional feature vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichContext {
    // Temporal features (6)
    pub hour_of_day: f32,           // 0-23, normalized to 0-1
    pub day_of_week: f32,           // 0-6, normalized to 0-1
    pub week_of_year: f32,          // 1-52, normalized to 0-1
    pub is_weekend: f32,            // 0 or 1
    pub time_since_wake: f32,       // Hours since first activity, normalized
    pub time_until_sleep: f32,      // Estimated hours until typical sleep

    // Physiological features (6)
    pub energy_level: f32,          // 1-10, normalized to 0-1
    pub energy_trajectory: f32,     // Change from previous (-1 to 1)
    pub mood_level: f32,            // 1-10, normalized to 0-1
    pub mood_trajectory: f32,       // Change from previous (-1 to 1)
    pub fatigue_score: f32,         // Inferred from patterns (0-1)
    pub recovery_need: f32,         // Based on recent intensity (0-1)

    // Learning/Skill features (6)
    pub skill_momentum: f32,        // Recent consistency (0-1)
    pub practice_diversity: f32,    // How varied practice has been (0-1)
    pub learning_rate: f32,         // Skill improvement velocity (0-1)
    pub focus_trend: f32,           // Session quality trend (-1 to 1)
    pub pomodoros_today: f32,       // Count, normalized
    pub study_minutes_today: f32,   // Minutes, normalized to hours

    // Goal features (6)
    pub big_3_completion: f32,      // % of Big 3 done (0-1)
    pub weekly_goal_progress: f32,  // % toward weekly goals (0-1)
    pub assignment_urgency: f32,    // Max urgency score (0-1)
    pub overdue_count: f32,         // Count, normalized
    pub streak_days: f32,           // Days, normalized
    pub goal_alignment: f32,        // Actions aligned with goals (0-1)

    // Circadian features (4)
    pub circadian_phase: f32,       // Phase in wake cycle (0-1)
    pub peak_focus_prob: f32,       // Probability this is peak hour (0-1)
    pub optimal_creative: f32,      // Good for creative work (0-1)
    pub optimal_analytical: f32,    // Good for analytical work (0-1)

    // Historical/Memory features (6)
    pub similar_context_outcome: f32,  // Avg outcome from similar contexts (0-1)
    pub same_hour_productivity: f32,   // Historical productivity at this hour (0-1)
    pub same_day_energy: f32,          // Historical energy on this day (0-1)
    pub hours_since_break: f32,        // Hours since last break, normalized
    pub hours_since_workout: f32,      // Hours since workout, normalized
    pub hours_since_checkin: f32,      // Hours since check-in, normalized

    // Workload features (6)
    pub active_assignments: f32,       // Count, normalized
    pub due_today: f32,                // Count, normalized
    pub due_this_week: f32,            // Count, normalized
    pub study_hours_week: f32,         // Hours this week, normalized
    pub target_hours_week: f32,        // Target hours, normalized
    pub workload_balance: f32,         // Actual/target ratio, clamped to 0-2

    // Interaction features (10)
    pub energy_x_hour: f32,            // Energy × circadian alignment
    pub mood_x_workload: f32,          // Mood × inverse workload
    pub streak_x_momentum: f32,        // Streak × consistency
    pub fatigue_x_time: f32,           // Fatigue × time of day factor
    pub focus_x_complexity: f32,       // Focus trend × task complexity
    pub recovery_x_intensity: f32,     // Recovery need × planned intensity
    pub energy_traj_x_goals: f32,      // Energy trajectory × goal urgency
    pub mood_traj_x_social: f32,       // Mood trajectory × social context
    pub circadian_x_task: f32,         // Circadian fit × task type
    pub history_x_current: f32,        // Historical success × current similarity
}

impl Default for RichContext {
    fn default() -> Self {
        Self {
            // Temporal
            hour_of_day: 0.5,
            day_of_week: 0.5,
            week_of_year: 0.5,
            is_weekend: 0.0,
            time_since_wake: 0.5,
            time_until_sleep: 0.5,

            // Physiological
            energy_level: 0.5,
            energy_trajectory: 0.0,
            mood_level: 0.5,
            mood_trajectory: 0.0,
            fatigue_score: 0.3,
            recovery_need: 0.3,

            // Learning
            skill_momentum: 0.5,
            practice_diversity: 0.5,
            learning_rate: 0.5,
            focus_trend: 0.0,
            pomodoros_today: 0.0,
            study_minutes_today: 0.0,

            // Goals
            big_3_completion: 0.0,
            weekly_goal_progress: 0.0,
            assignment_urgency: 0.0,
            overdue_count: 0.0,
            streak_days: 0.0,
            goal_alignment: 0.5,

            // Circadian
            circadian_phase: 0.5,
            peak_focus_prob: 0.5,
            optimal_creative: 0.5,
            optimal_analytical: 0.5,

            // Historical
            similar_context_outcome: 0.5,
            same_hour_productivity: 0.5,
            same_day_energy: 0.5,
            hours_since_break: 0.5,
            hours_since_workout: 0.5,
            hours_since_checkin: 0.5,

            // Workload
            active_assignments: 0.0,
            due_today: 0.0,
            due_this_week: 0.0,
            study_hours_week: 0.0,
            target_hours_week: 0.0,
            workload_balance: 1.0,

            // Interactions
            energy_x_hour: 0.25,
            mood_x_workload: 0.25,
            streak_x_momentum: 0.0,
            fatigue_x_time: 0.15,
            focus_x_complexity: 0.0,
            recovery_x_intensity: 0.09,
            energy_traj_x_goals: 0.0,
            mood_traj_x_social: 0.0,
            circadian_x_task: 0.25,
            history_x_current: 0.25,
        }
    }
}

impl RichContext {
    /// Convert to feature vector for ML
    pub fn to_feature_vector(&self) -> Array1<f32> {
        Array1::from(vec![
            // Temporal (6)
            self.hour_of_day,
            self.day_of_week,
            self.week_of_year,
            self.is_weekend,
            self.time_since_wake,
            self.time_until_sleep,
            // Physiological (6)
            self.energy_level,
            self.energy_trajectory,
            self.mood_level,
            self.mood_trajectory,
            self.fatigue_score,
            self.recovery_need,
            // Learning (6)
            self.skill_momentum,
            self.practice_diversity,
            self.learning_rate,
            self.focus_trend,
            self.pomodoros_today,
            self.study_minutes_today,
            // Goals (6)
            self.big_3_completion,
            self.weekly_goal_progress,
            self.assignment_urgency,
            self.overdue_count,
            self.streak_days,
            self.goal_alignment,
            // Circadian (4)
            self.circadian_phase,
            self.peak_focus_prob,
            self.optimal_creative,
            self.optimal_analytical,
            // Historical (6)
            self.similar_context_outcome,
            self.same_hour_productivity,
            self.same_day_energy,
            self.hours_since_break,
            self.hours_since_workout,
            self.hours_since_checkin,
            // Workload (6)
            self.active_assignments,
            self.due_today,
            self.due_this_week,
            self.study_hours_week,
            self.target_hours_week,
            self.workload_balance,
            // Interactions (10)
            self.energy_x_hour,
            self.mood_x_workload,
            self.streak_x_momentum,
            self.fatigue_x_time,
            self.focus_x_complexity,
            self.recovery_x_intensity,
            self.energy_traj_x_goals,
            self.mood_traj_x_social,
            self.circadian_x_task,
            self.history_x_current,
        ])
    }

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        let vec = self.to_feature_vector();
        vec.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != FEATURE_DIM * 4 {
            return None;
        }

        let floats: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        if floats.len() != FEATURE_DIM {
            return None;
        }

        Some(Self {
            hour_of_day: floats[0],
            day_of_week: floats[1],
            week_of_year: floats[2],
            is_weekend: floats[3],
            time_since_wake: floats[4],
            time_until_sleep: floats[5],
            energy_level: floats[6],
            energy_trajectory: floats[7],
            mood_level: floats[8],
            mood_trajectory: floats[9],
            fatigue_score: floats[10],
            recovery_need: floats[11],
            skill_momentum: floats[12],
            practice_diversity: floats[13],
            learning_rate: floats[14],
            focus_trend: floats[15],
            pomodoros_today: floats[16],
            study_minutes_today: floats[17],
            big_3_completion: floats[18],
            weekly_goal_progress: floats[19],
            assignment_urgency: floats[20],
            overdue_count: floats[21],
            streak_days: floats[22],
            goal_alignment: floats[23],
            circadian_phase: floats[24],
            peak_focus_prob: floats[25],
            optimal_creative: floats[26],
            optimal_analytical: floats[27],
            similar_context_outcome: floats[28],
            same_hour_productivity: floats[29],
            same_day_energy: floats[30],
            hours_since_break: floats[31],
            hours_since_workout: floats[32],
            hours_since_checkin: floats[33],
            active_assignments: floats[34],
            due_today: floats[35],
            due_this_week: floats[36],
            study_hours_week: floats[37],
            target_hours_week: floats[38],
            workload_balance: floats[39],
            energy_x_hour: floats[40],
            mood_x_workload: floats[41],
            streak_x_momentum: floats[42],
            fatigue_x_time: floats[43],
            focus_x_complexity: floats[44],
            recovery_x_intensity: floats[45],
            energy_traj_x_goals: floats[46],
            mood_traj_x_social: floats[47],
            circadian_x_task: floats[48],
            history_x_current: floats[49],
        })
    }

    /// Get feature names for explainability
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "hour_of_day",
            "day_of_week",
            "week_of_year",
            "is_weekend",
            "time_since_wake",
            "time_until_sleep",
            "energy_level",
            "energy_trajectory",
            "mood_level",
            "mood_trajectory",
            "fatigue_score",
            "recovery_need",
            "skill_momentum",
            "practice_diversity",
            "learning_rate",
            "focus_trend",
            "pomodoros_today",
            "study_minutes_today",
            "big_3_completion",
            "weekly_goal_progress",
            "assignment_urgency",
            "overdue_count",
            "streak_days",
            "goal_alignment",
            "circadian_phase",
            "peak_focus_prob",
            "optimal_creative",
            "optimal_analytical",
            "similar_context_outcome",
            "same_hour_productivity",
            "same_day_energy",
            "hours_since_break",
            "hours_since_workout",
            "hours_since_checkin",
            "active_assignments",
            "due_today",
            "due_this_week",
            "study_hours_week",
            "target_hours_week",
            "workload_balance",
            "energy_x_hour",
            "mood_x_workload",
            "streak_x_momentum",
            "fatigue_x_time",
            "focus_x_complexity",
            "recovery_x_intensity",
            "energy_traj_x_goals",
            "mood_traj_x_social",
            "circadian_x_task",
            "history_x_current",
        ]
    }

    /// Create human-readable context description for embedding
    pub fn to_description(&self) -> String {
        let energy_desc = if self.energy_level > 0.7 {
            "high energy"
        } else if self.energy_level > 0.4 {
            "moderate energy"
        } else {
            "low energy"
        };

        let mood_desc = if self.mood_level > 0.7 {
            "good mood"
        } else if self.mood_level > 0.4 {
            "neutral mood"
        } else {
            "low mood"
        };

        let time_desc = if self.hour_of_day < 0.5 {
            "morning"
        } else if self.hour_of_day < 0.75 {
            "afternoon"
        } else {
            "evening"
        };

        let workload_desc = if self.active_assignments > 0.3 {
            "high workload"
        } else if self.active_assignments > 0.1 {
            "moderate workload"
        } else {
            "light workload"
        };

        format!(
            "{}, {}, {} {}, {}",
            energy_desc,
            mood_desc,
            if self.is_weekend > 0.5 {
                "weekend"
            } else {
                "weekday"
            },
            time_desc,
            workload_desc
        )
    }
}

/// Rich feature store - captures comprehensive context
pub struct RichFeatureStore;

impl RichFeatureStore {
    /// Capture current rich context from the database
    pub async fn capture_context(pool: &Pool<Sqlite>) -> Result<RichContext, String> {
        let now = Local::now();
        let hour = now.hour() as f32;
        let day = now.weekday().num_days_from_sunday() as f32;
        let week = now.iso_week().week() as f32;

        let mut ctx = RichContext::default();

        // === Temporal features ===
        ctx.hour_of_day = hour / 23.0;
        ctx.day_of_week = day / 6.0;
        ctx.week_of_year = week / 52.0;
        ctx.is_weekend = if day == 0.0 || day == 6.0 { 1.0 } else { 0.0 };

        // Estimate time since wake (assume 7am wake time by default)
        let wake_hour = 7.0f32;
        ctx.time_since_wake = ((hour - wake_hour).max(0.0) / 16.0).min(1.0);

        // Estimate time until sleep (assume 11pm sleep time)
        let sleep_hour = 23.0f32;
        ctx.time_until_sleep = ((sleep_hour - hour).max(0.0) / 16.0).min(1.0);

        // === Physiological features ===
        let checkin_data: Option<(Option<i32>, Option<i32>, Option<i32>, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT c1.mood, c1.energy, c2.mood, c2.energy
            FROM check_ins c1
            LEFT JOIN check_ins c2 ON date(c2.checked_in_at) = date('now', '-1 day')
            WHERE date(c1.checked_in_at) = date('now')
            ORDER BY c1.checked_in_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some((mood, energy, prev_mood, prev_energy)) = checkin_data {
            if let Some(e) = energy {
                ctx.energy_level = (e as f32 - 1.0) / 9.0;
                if let Some(pe) = prev_energy {
                    ctx.energy_trajectory = ((e - pe) as f32 / 9.0).clamp(-1.0, 1.0);
                }
            }
            if let Some(m) = mood {
                ctx.mood_level = (m as f32 - 1.0) / 9.0;
                if let Some(pm) = prev_mood {
                    ctx.mood_trajectory = ((m - pm) as f32 / 9.0).clamp(-1.0, 1.0);
                }
            }
        }

        // Hours since last check-in
        let hours_since_checkin: Option<f64> = sqlx::query_scalar(
            "SELECT (julianday('now') - julianday(checked_in_at)) * 24 FROM check_ins ORDER BY checked_in_at DESC LIMIT 1"
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        ctx.hours_since_checkin = hours_since_checkin.map(|h| (h as f32 / 24.0).min(1.0)).unwrap_or(1.0);

        // === Learning/Skill features ===
        // Pomodoros today
        let pomodoros: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sessions WHERE session_type = 'study' AND date(started_at) = date('now')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.pomodoros_today = (pomodoros as f32 / 12.0).min(1.0); // Normalize to ~12 max

        // Study minutes today
        let study_mins: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions WHERE session_type = 'study' AND date(started_at) = date('now')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.study_minutes_today = (study_mins as f32 / 480.0).min(1.0); // Normalize to 8 hours

        // Practice diversity (unique skills practiced this week)
        let skills_practiced: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT skill_id) FROM practice_logs WHERE logged_at >= date('now', '-7 days')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        let total_skills: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM skills")
            .fetch_one(pool)
            .await
            .unwrap_or(1);
        ctx.practice_diversity = if total_skills > 0 {
            (skills_practiced as f32 / total_skills as f32).min(1.0)
        } else {
            0.0
        };

        // === Goal features ===
        // Big 3 completion
        let today = now.format("%Y-%m-%d").to_string();
        let big3_stats: (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*), SUM(CASE WHEN is_completed = 1 THEN 1 ELSE 0 END) FROM agent_big_three WHERE date = ?"
        )
        .bind(&today)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or((0, 0));
        ctx.big_3_completion = if big3_stats.0 > 0 {
            big3_stats.1 as f32 / big3_stats.0 as f32
        } else {
            0.0
        };

        // Assignment urgency (max urgency of incomplete assignments)
        let urgency_data: Vec<(Option<String>, i32)> = sqlx::query_as(
            r#"
            SELECT due_date, priority FROM assignments 
            WHERE is_completed = 0 AND due_date IS NOT NULL
            ORDER BY due_date ASC
            LIMIT 5
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut max_urgency = 0.0f32;
        for (due_date, priority) in urgency_data {
            if let Some(due) = due_date {
                if let Ok(due_naive) = NaiveDate::parse_from_str(&due, "%Y-%m-%d") {
                    let days_until = (due_naive - now.date_naive()).num_days();
                    let time_urgency = if days_until < 0 {
                        1.0 // Overdue
                    } else if days_until == 0 {
                        0.95
                    } else {
                        (1.0 - (days_until as f32 / 14.0)).max(0.0) // 2 weeks = 0 urgency
                    };
                    let priority_factor = priority as f32 / 3.0;
                    let urgency = time_urgency * 0.7 + priority_factor * 0.3;
                    max_urgency = max_urgency.max(urgency);
                }
            }
        }
        ctx.assignment_urgency = max_urgency;

        // Overdue count
        let overdue: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM assignments WHERE is_completed = 0 AND due_date < date('now')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.overdue_count = (overdue as f32 / 5.0).min(1.0);

        // Active assignments
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM assignments WHERE is_completed = 0"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.active_assignments = (active as f32 / 20.0).min(1.0);

        // Due today/this week
        let due_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM assignments WHERE is_completed = 0 AND due_date = date('now')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.due_today = (due_today as f32 / 5.0).min(1.0);

        let due_week: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM assignments WHERE is_completed = 0 AND due_date BETWEEN date('now') AND date('now', '+7 days')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.due_this_week = (due_week as f32 / 10.0).min(1.0);

        // Streak days (check-in streak)
        let streak: i64 = sqlx::query_scalar(
            r#"
            WITH RECURSIVE streak AS (
                SELECT date('now') as d, 1 as count
                UNION ALL
                SELECT date(d, '-1 day'), count + 1 FROM streak
                WHERE EXISTS (SELECT 1 FROM check_ins WHERE date(checked_in_at) = date(d, '-1 day'))
                AND count < 100
            )
            SELECT MAX(count) FROM streak
            WHERE EXISTS (SELECT 1 FROM check_ins WHERE date(checked_in_at) = d)
            "#
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.streak_days = (streak as f32 / 30.0).min(1.0);

        // === Workout features ===
        let hours_since_workout: Option<f64> = sqlx::query_scalar(
            "SELECT (julianday('now') - julianday(logged_at)) * 24 FROM workouts ORDER BY logged_at DESC LIMIT 1"
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        ctx.hours_since_workout = hours_since_workout.map(|h| (h as f32 / 48.0).min(1.0)).unwrap_or(1.0);

        // === Circadian features ===
        // Peak focus probability based on common patterns
        let peak_hours = vec![9.0, 10.0, 11.0, 14.0, 15.0, 16.0];
        ctx.peak_focus_prob = if peak_hours.iter().any(|&h| (hour - h).abs() < 1.0) {
            0.8
        } else if hour >= 6.0 && hour <= 20.0 {
            0.5
        } else {
            0.2
        };

        // Circadian phase (0 = wake, 1 = sleep)
        ctx.circadian_phase = ctx.time_since_wake;

        // Creative vs analytical (morning = creative, afternoon = analytical)
        ctx.optimal_creative = if hour >= 6.0 && hour <= 12.0 { 0.8 } else { 0.4 };
        ctx.optimal_analytical = if hour >= 14.0 && hour <= 18.0 { 0.8 } else { 0.4 };

        // === Fatigue and recovery ===
        // Estimate fatigue from recent session density
        let recent_session_mins: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions WHERE started_at >= datetime('now', '-4 hours')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.fatigue_score = (recent_session_mins as f32 / 180.0).min(1.0); // 3 hours = max fatigue

        // Recovery need based on fatigue and time since break
        let hours_since_break: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT MIN((julianday('now') - julianday(ended_at)) * 24) 
            FROM sessions 
            WHERE ended_at IS NOT NULL
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        ctx.hours_since_break = hours_since_break.map(|h| (h as f32 / 2.0).min(1.0)).unwrap_or(0.0);
        ctx.recovery_need = (ctx.fatigue_score * 0.6 + ctx.hours_since_break * 0.4).min(1.0);

        // === Weekly study hours ===
        let week_study: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM sessions WHERE session_type = 'study' AND started_at >= date('now', 'weekday 0', '-7 days')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.study_hours_week = (week_study as f32 / 60.0) / 40.0; // Normalize to 40 hours

        // Target hours from courses
        let target: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(target_weekly_hours), 0) FROM courses"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        ctx.target_hours_week = (target as f32 / 40.0).min(1.0);
        ctx.workload_balance = if target > 0 {
            ((week_study as f32 / 60.0) / target as f32).clamp(0.0, 2.0)
        } else {
            1.0
        };

        // === Compute interaction features ===
        ctx.energy_x_hour = ctx.energy_level * ctx.peak_focus_prob;
        ctx.mood_x_workload = ctx.mood_level * (1.0 - ctx.active_assignments);
        ctx.streak_x_momentum = ctx.streak_days * ctx.skill_momentum;
        ctx.fatigue_x_time = ctx.fatigue_score * (1.0 - ctx.time_until_sleep);
        ctx.focus_x_complexity = ctx.focus_trend * ctx.assignment_urgency;
        ctx.recovery_x_intensity = ctx.recovery_need * ctx.fatigue_score;
        ctx.energy_traj_x_goals = ((ctx.energy_trajectory + 1.0) / 2.0) * ctx.assignment_urgency;
        ctx.mood_traj_x_social = (ctx.mood_trajectory + 1.0) / 2.0; // Placeholder
        ctx.circadian_x_task = ctx.peak_focus_prob * ctx.optimal_analytical;
        ctx.history_x_current = ctx.similar_context_outcome * ctx.energy_level;

        Ok(ctx)
    }

    /// Save a rich context snapshot to the database
    pub async fn save_snapshot(pool: &Pool<Sqlite>, ctx: &RichContext) -> Result<i64, String> {
        let result = sqlx::query(
            r#"
            INSERT INTO agent_rich_context (
                hour_of_day, day_of_week, week_of_year, is_weekend, time_since_wake, time_until_sleep,
                energy_level, energy_trajectory, mood_level, mood_trajectory, fatigue_score, recovery_need,
                skill_momentum, practice_diversity, learning_rate_estimate, focus_trend, pomodoros_today, study_minutes_today,
                big_3_completion, weekly_goal_progress, assignment_urgency_max, overdue_count, streak_days, goal_alignment_score,
                circadian_phase, peak_focus_probability, optimal_for_creative, optimal_for_analytical,
                similar_context_outcomes, same_hour_avg_productivity, same_day_avg_energy, 
                last_break_hours_ago, last_workout_hours_ago, last_checkin_hours_ago,
                active_assignments, assignments_due_today, assignments_due_week,
                study_hours_this_week, target_study_hours_week, workload_balance,
                energy_x_hour, mood_x_workload, streak_x_momentum, fatigue_x_time, focus_x_complexity,
                recovery_x_intensity, energy_trajectory_x_goals, mood_trajectory_x_social, 
                circadian_x_task_type, historical_x_current
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind((ctx.hour_of_day * 23.0) as i32)
        .bind((ctx.day_of_week * 6.0) as i32)
        .bind((ctx.week_of_year * 52.0) as i32)
        .bind(ctx.is_weekend as i32)
        .bind(ctx.time_since_wake)
        .bind(ctx.time_until_sleep)
        .bind(ctx.energy_level)
        .bind(ctx.energy_trajectory)
        .bind(ctx.mood_level)
        .bind(ctx.mood_trajectory)
        .bind(ctx.fatigue_score)
        .bind(ctx.recovery_need)
        .bind(ctx.skill_momentum)
        .bind(ctx.practice_diversity)
        .bind(ctx.learning_rate)
        .bind(ctx.focus_trend)
        .bind(ctx.pomodoros_today as i32)
        .bind(ctx.study_minutes_today)
        .bind(ctx.big_3_completion)
        .bind(ctx.weekly_goal_progress)
        .bind(ctx.assignment_urgency)
        .bind(ctx.overdue_count as i32)
        .bind(ctx.streak_days as i32)
        .bind(ctx.goal_alignment)
        .bind(ctx.circadian_phase)
        .bind(ctx.peak_focus_prob)
        .bind(ctx.optimal_creative)
        .bind(ctx.optimal_analytical)
        .bind(ctx.similar_context_outcome)
        .bind(ctx.same_hour_productivity)
        .bind(ctx.same_day_energy)
        .bind(ctx.hours_since_break)
        .bind(ctx.hours_since_workout)
        .bind(ctx.hours_since_checkin)
        .bind(ctx.active_assignments as i32)
        .bind(ctx.due_today as i32)
        .bind(ctx.due_this_week as i32)
        .bind(ctx.study_hours_week)
        .bind(ctx.target_hours_week)
        .bind(ctx.workload_balance)
        .bind(ctx.energy_x_hour)
        .bind(ctx.mood_x_workload)
        .bind(ctx.streak_x_momentum)
        .bind(ctx.fatigue_x_time)
        .bind(ctx.focus_x_complexity)
        .bind(ctx.recovery_x_intensity)
        .bind(ctx.energy_traj_x_goals)
        .bind(ctx.mood_traj_x_social)
        .bind(ctx.circadian_x_task)
        .bind(ctx.history_x_current)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }
}
