use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CheckIn {
    pub id: i64,
    pub user_id: i64,
    pub mood: Option<i64>,
    pub energy: Option<i64>,
    pub notes: Option<String>,
    pub checked_in_at: Option<String>,
}
