use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WeekPlanBlock {
    pub id: i64,
    pub user_id: i64,
    pub week_start_date: String,
    pub start_at: String,
    pub end_at: String,
    pub block_type: String,
    pub course_id: Option<i64>,
    pub weekly_task_id: Option<i64>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub rationale_json: Option<String>,
    pub created_at: Option<String>,
}
