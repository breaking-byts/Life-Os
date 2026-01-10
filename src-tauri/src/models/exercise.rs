use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ExerciseCache {
    pub id: i64,
    pub wger_id: Option<i64>,
    pub name: String,
    pub category: Option<String>,
    pub muscles: Option<String>,
    pub equipment: Option<String>,
    pub description: Option<String>,
    pub cached_at: Option<String>,
    pub source: String,
    pub created_at: Option<String>,
}
