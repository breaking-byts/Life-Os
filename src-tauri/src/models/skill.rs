use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Skill {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub target_weekly_hours: Option<f64>,
    pub current_level: Option<i64>,
    pub total_hours: Option<f64>,
    pub created_at: Option<String>,
}
