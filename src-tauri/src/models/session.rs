use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub session_type: String,
    pub reference_id: Option<i64>,
    pub reference_type: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_minutes: Option<i64>,
    pub notes: Option<String>,
}
