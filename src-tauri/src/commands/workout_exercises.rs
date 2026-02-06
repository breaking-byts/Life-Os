use tauri::State;

use crate::{DbState, error::ApiError, models::workout::WorkoutExercise};

#[derive(Debug, serde::Deserialize)]
pub struct WorkoutExerciseInput {
    pub workout_id: i64,
    pub exercise_id: Option<i64>,
    pub exercise_name: String,
    pub sets: Option<i64>,
    pub reps: Option<i64>,
    pub weight: Option<f64>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn add_exercise_to_workout(state: State<'_, DbState>, data: WorkoutExerciseInput) -> Result<WorkoutExercise, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, WorkoutExercise>(
        "INSERT INTO workout_exercises (workout_id, exercise_id, exercise_name, sets, reps, weight, notes) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id, workout_id, exercise_id, exercise_name, sets, reps, weight, notes"
    )
    .bind(data.workout_id)
    .bind(data.exercise_id)
    .bind(&data.exercise_name)
    .bind(data.sets)
    .bind(data.reps)
    .bind(data.weight)
    .bind(&data.notes)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn update_workout_exercise(state: State<'_, DbState>, id: i64, data: WorkoutExerciseInput) -> Result<WorkoutExercise, ApiError> {
    let pool = &state.0;
    let rec = sqlx::query_as::<_, WorkoutExercise>(
        "UPDATE workout_exercises SET workout_id = COALESCE(?, workout_id), exercise_id = COALESCE(?, exercise_id), exercise_name = COALESCE(?, exercise_name), sets = COALESCE(?, sets), reps = COALESCE(?, reps), weight = COALESCE(?, weight), notes = COALESCE(?, notes) WHERE id = ? RETURNING id, workout_id, exercise_id, exercise_name, sets, reps, weight, notes"
    )
    .bind(Some(data.workout_id))
    .bind(data.exercise_id)
    .bind(Some(&data.exercise_name))
    .bind(data.sets)
    .bind(data.reps)
    .bind(data.weight)
    .bind(&data.notes)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(rec)
}

#[tauri::command]
pub async fn remove_exercise(state: State<'_, DbState>, id: i64) -> Result<bool, ApiError> {
    let pool = &state.0;
    let result = sqlx::query("DELETE FROM workout_exercises WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ApiError::from)?;
    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Workout exercise not found"));
    }
    Ok(true)
}

#[tauri::command]
pub async fn get_workout_exercises(
    state: State<'_, DbState>,
    workout_id: i64,
) -> Result<Vec<WorkoutExercise>, ApiError> {
    let pool = &state.0;
    let exercises = sqlx::query_as::<_, WorkoutExercise>(
        "SELECT id, workout_id, exercise_id, exercise_name, sets, reps, weight, notes 
         FROM workout_exercises WHERE workout_id = ? ORDER BY id"
    )
    .bind(workout_id)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(exercises)
}
