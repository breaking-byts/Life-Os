//! Pattern Miner - Discovers behavioral patterns from user data
//!
//! Identifies temporal patterns, sequences, and correlations to build
//! deep understanding of user behavior.

use sqlx::{Pool, Sqlite};
use serde_json;


use super::models::{Pattern, PatternData, Context};

/// Pattern miner for discovering behavioral insights
pub struct PatternMiner;

impl PatternMiner {
    /// Analyze check-in data to find mood/energy patterns
    pub async fn analyze_mood_patterns(pool: &Pool<Sqlite>) -> Result<Vec<Pattern>, String> {
        let mut patterns = vec![];

        // Find peak mood hours
        let peak_hours: Vec<(i32, f64)> = sqlx::query_as(
            r#"
            SELECT 
                CAST(strftime('%H', checked_in_at) AS INTEGER) as hour,
                AVG(mood) as avg_mood
            FROM check_ins
            WHERE mood IS NOT NULL
            GROUP BY hour
            HAVING COUNT(*) >= 3
            ORDER BY avg_mood DESC
            LIMIT 3
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        if !peak_hours.is_empty() {
            let hours: Vec<i32> = peak_hours.iter().map(|(h, _)| *h).collect();
            let avg_mood: f64 = peak_hours.iter().map(|(_, m)| m).sum::<f64>() / peak_hours.len() as f64;
            
            let pattern_data = PatternData::Temporal {
                peak_hours: hours.clone(),
                peak_days: vec![],
                metric: "mood".to_string(),
            };

            patterns.push(Pattern {
                id: 0,
                pattern_type: "temporal".to_string(),
                pattern_name: Some("peak_mood_hours".to_string()),
                pattern_json: serde_json::to_string(&pattern_data).unwrap_or_default(),
                support: peak_hours.len() as f64 / 24.0,
                confidence: avg_mood / 10.0,
                last_validated: None,
                is_active: true,
                created_at: None,
            });
        }

        // Find peak energy days
        let peak_days: Vec<(i32, f64)> = sqlx::query_as(
            r#"
            SELECT 
                CAST(strftime('%w', checked_in_at) AS INTEGER) as day,
                AVG(energy) as avg_energy
            FROM check_ins
            WHERE energy IS NOT NULL
            GROUP BY day
            HAVING COUNT(*) >= 2
            ORDER BY avg_energy DESC
            LIMIT 2
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        if !peak_days.is_empty() {
            let days: Vec<i32> = peak_days.iter().map(|(d, _)| *d).collect();
            let avg_energy: f64 = peak_days.iter().map(|(_, e)| e).sum::<f64>() / peak_days.len() as f64;
            
            let pattern_data = PatternData::Temporal {
                peak_hours: vec![],
                peak_days: days.clone(),
                metric: "energy".to_string(),
            };

            patterns.push(Pattern {
                id: 0,
                pattern_type: "temporal".to_string(),
                pattern_name: Some("peak_energy_days".to_string()),
                pattern_json: serde_json::to_string(&pattern_data).unwrap_or_default(),
                support: peak_days.len() as f64 / 7.0,
                confidence: avg_energy / 10.0,
                last_validated: None,
                is_active: true,
                created_at: None,
            });
        }

        Ok(patterns)
    }

    /// Analyze study session patterns
    pub async fn analyze_study_patterns(pool: &Pool<Sqlite>) -> Result<Vec<Pattern>, String> {
        let mut patterns = vec![];

        // Find productive study hours
        let productive_hours: Vec<(i32, f64)> = sqlx::query_as(
            r#"
            SELECT 
                CAST(strftime('%H', started_at) AS INTEGER) as hour,
                AVG(duration_minutes) as avg_duration
            FROM sessions
            WHERE session_type = 'study' AND duration_minutes IS NOT NULL
            GROUP BY hour
            HAVING COUNT(*) >= 3 AND AVG(duration_minutes) >= 25
            ORDER BY avg_duration DESC
            LIMIT 3
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        if !productive_hours.is_empty() {
            let hours: Vec<i32> = productive_hours.iter().map(|(h, _)| *h).collect();
            let avg_duration: f64 = productive_hours.iter().map(|(_, d)| d).sum::<f64>() / productive_hours.len() as f64;
            
            let pattern_data = PatternData::Temporal {
                peak_hours: hours.clone(),
                peak_days: vec![],
                metric: "study_duration".to_string(),
            };

            patterns.push(Pattern {
                id: 0,
                pattern_type: "temporal".to_string(),
                pattern_name: Some("productive_study_hours".to_string()),
                pattern_json: serde_json::to_string(&pattern_data).unwrap_or_default(),
                support: productive_hours.len() as f64 / 24.0,
                confidence: (avg_duration / 60.0).min(1.0),  // Normalize to 0-1
                last_validated: None,
                is_active: true,
                created_at: None,
            });
        }

        Ok(patterns)
    }

    /// Analyze workout → productivity correlations
    pub async fn analyze_workout_correlations(pool: &Pool<Sqlite>) -> Result<Vec<Pattern>, String> {
        let mut patterns = vec![];

        // Check if study sessions are longer/more frequent after workouts
        let correlation: Option<(Option<f64>, Option<f64>)> = sqlx::query_as(
            r#"
            WITH study_with_recent_workout AS (
                SELECT s.duration_minutes,
                    CASE WHEN EXISTS (
                        SELECT 1 FROM workouts w
                        WHERE date(w.logged_at) = date(s.started_at)
                        AND w.logged_at < s.started_at
                    ) THEN 1 ELSE 0 END as had_workout
                FROM sessions s
                WHERE s.session_type = 'study' AND s.duration_minutes IS NOT NULL
            )
            SELECT 
                AVG(CASE WHEN had_workout = 1 THEN duration_minutes END) as with_workout,
                AVG(CASE WHEN had_workout = 0 THEN duration_minutes END) as without_workout
            FROM study_with_recent_workout
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some((Some(with), Some(without))) = correlation {
            if with > without * 1.1 {  // At least 10% improvement
                let improvement = (with - without) / without;
                
                let pattern_data = PatternData::Correlation {
                    factor_a: "morning_workout".to_string(),
                    factor_b: "study_duration".to_string(),
                    correlation: improvement,
                    direction: "positive".to_string(),
                };

                patterns.push(Pattern {
                    id: 0,
                    pattern_type: "correlation".to_string(),
                    pattern_name: Some("workout_boosts_study".to_string()),
                    pattern_json: serde_json::to_string(&pattern_data).unwrap_or_default(),
                    support: 0.5,  // Placeholder
                    confidence: improvement.min(1.0),
                    last_validated: None,
                    is_active: true,
                    created_at: None,
                });
            }
        }

        Ok(patterns)
    }

    /// Run all pattern analyses and save discovered patterns
    pub async fn discover_and_save_patterns(pool: &Pool<Sqlite>) -> Result<usize, String> {
        let mut all_patterns = vec![];
        
        all_patterns.extend(Self::analyze_mood_patterns(pool).await?);
        all_patterns.extend(Self::analyze_study_patterns(pool).await?);
        all_patterns.extend(Self::analyze_workout_correlations(pool).await?);

        let count = all_patterns.len();

        for pattern in all_patterns {
            // Upsert pattern by name
            sqlx::query(
                r#"
                INSERT INTO agent_patterns (pattern_type, pattern_name, pattern_json, support, confidence, last_validated, is_active)
                VALUES (?, ?, ?, ?, ?, datetime('now'), 1)
                ON CONFLICT(id) DO UPDATE SET
                    pattern_json = excluded.pattern_json,
                    support = excluded.support,
                    confidence = excluded.confidence,
                    last_validated = datetime('now')
                "#
            )
            .bind(&pattern.pattern_type)
            .bind(&pattern.pattern_name)
            .bind(&pattern.pattern_json)
            .bind(pattern.support)
            .bind(pattern.confidence)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(count)
    }

    /// Get active patterns for insight generation
    pub async fn get_active_patterns(pool: &Pool<Sqlite>) -> Result<Vec<Pattern>, String> {
        sqlx::query_as::<_, Pattern>(
            "SELECT * FROM agent_patterns WHERE is_active = 1 ORDER BY confidence DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    /// Generate insight text from a pattern
    pub fn pattern_to_insight(pattern: &Pattern, ctx: &Context) -> Option<String> {
        let data: PatternData = serde_json::from_str(&pattern.pattern_json).ok()?;
        
        match data {
            PatternData::Temporal { peak_hours, peak_days, metric } => {
                if metric == "mood" && !peak_hours.is_empty() && peak_hours.contains(&ctx.hour_of_day) {
                    Some(format!("You're typically in a great mood around {}:00! Good time for challenging work.", ctx.hour_of_day))
                } else if metric == "study_duration" && !peak_hours.is_empty() && peak_hours.contains(&ctx.hour_of_day) {
                    Some(format!("Your most productive study hours include now ({}:00). Perfect time for deep work!", ctx.hour_of_day))
                } else if metric == "energy" && !peak_days.is_empty() {
                    let day_names = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
                    if peak_days.contains(&ctx.day_of_week) {
                        Some(format!("{} is typically a high-energy day for you!", day_names.get(ctx.day_of_week as usize).unwrap_or(&"Today")))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            PatternData::Correlation { factor_a, factor_b, correlation, direction } => {
                if factor_a.contains("workout") && direction == "positive" && ctx.recent_workout_count > 0 {
                    Some(format!("Nice workout! You typically see {:.0}% better {} afterward.", correlation * 100.0, factor_b.replace("_", " ")))
                } else if factor_a.contains("workout") && ctx.recent_workout_count == 0 {
                    Some(format!("A workout could boost your {} by {:.0}%!", factor_b.replace("_", " "), correlation * 100.0))
                } else {
                    None
                }
            },
            PatternData::Sequence { .. } => {
                // TODO: Implement sequence-based insights
                None
            }
        }
    }
}
