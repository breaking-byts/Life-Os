use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CalendarEvent {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub rrule: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub category: String,
    pub domain: Option<String>,
    pub linked_id: Option<i64>,
    pub locked: Option<i64>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}
