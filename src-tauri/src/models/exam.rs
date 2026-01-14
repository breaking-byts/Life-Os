use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Exam {
    pub id: i64,
    pub course_id: i64,
    pub title: String,
    pub exam_date: Option<String>,
    pub location: Option<String>,
    pub duration_minutes: Option<i64>,
    pub notes: Option<String>,
    pub grade: Option<f64>,
    pub weight: Option<f64>,
    pub created_at: Option<String>,
}
