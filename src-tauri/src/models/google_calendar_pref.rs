use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct GoogleCalendarPref {
    pub id: i64,
    pub user_id: i64,
    pub import_all: Option<i64>,
    pub export_calendar_id: Option<String>,
    pub updated_at: Option<String>,
}
