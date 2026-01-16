use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WeeklyTask {
    pub id: i64,
    pub user_id: i64,
    pub week_start_date: String,
    pub title: String,
    pub course_id: Option<i64>,
    pub duration_minutes: Option<i64>,
    pub priority: Option<String>,
    pub notes: Option<String>,
    pub completed: Option<i64>,
    pub created_at: Option<String>,
}
