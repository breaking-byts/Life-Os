use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct GoogleEventLink {
    pub id: i64,
    pub local_type: String,
    pub local_id: i64,
    pub google_calendar_id: String,
    pub google_event_id: String,
    pub etag: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
}
