use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct GoogleAccount {
    pub id: i64,
    pub user_id: i64,
    pub google_user_id: String,
    pub email: Option<String>,
    pub connected_at: Option<String>,
}
