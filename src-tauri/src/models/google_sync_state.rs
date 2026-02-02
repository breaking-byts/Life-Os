use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct GoogleSyncState {
    pub id: i64,
    pub google_calendar_id: String,
    pub sync_token: Option<String>,
    pub updated_at: Option<String>,
}
