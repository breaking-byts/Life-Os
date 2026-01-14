use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Workout {
    pub id: i64,
    pub user_id: i64,
    pub name: Option<String>,
    pub duration_minutes: Option<i64>,
    pub notes: Option<String>,
    pub logged_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WorkoutExercise {
    pub id: i64,
    pub workout_id: i64,
    pub exercise_id: Option<i64>,
    pub exercise_name: String,
    pub sets: Option<i64>,
    pub reps: Option<i64>,
    pub weight: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WorkoutTemplate {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WorkoutTemplateExercise {
    pub id: i64,
    pub template_id: i64,
    pub exercise_id: Option<i64>,
    pub exercise_name: String,
    pub default_sets: Option<i64>,
    pub default_reps: Option<i64>,
    pub default_weight: Option<f64>,
    pub order_index: Option<i64>,
}
