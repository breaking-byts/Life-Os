use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Course {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub code: Option<String>,
    pub color: Option<String>,
    pub credit_hours: Option<i64>,
    pub target_weekly_hours: Option<f64>,
    pub is_active: Option<i64>,
    pub created_at: Option<String>,
}
