use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Assignment {
    pub id: i64,
    pub course_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<String>,
    pub is_completed: Option<i64>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}
