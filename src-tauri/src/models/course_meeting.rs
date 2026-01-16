use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CourseMeeting {
    pub id: i64,
    pub course_id: i64,
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    pub location: Option<String>,
    pub meeting_type: Option<String>,
    pub created_at: Option<String>,
}
