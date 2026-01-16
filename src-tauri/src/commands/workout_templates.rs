use tauri::State;

use crate::{DbState, models::workout::{WorkoutTemplate, WorkoutTemplateExercise}};

#[derive(Debug, serde::Deserialize)]
pub struct TemplateExerciseInput {
    pub exercise_id: Option<i64>,
    pub exercise_name: String,
    pub default_sets: Option<i64>,
    pub default_reps: Option<i64>,
    pub default_weight: Option<f64>,
    pub order_index: Option<i64>,
}

#[tauri::command]
pub async fn get_workout_templates(state: State<'_, DbState>) -> Result<Vec<WorkoutTemplate>, String> {
    let pool = &state.0;
    let rows = sqlx::query_as::<_, WorkoutTemplate>(
        "SELECT id, user_id, name, created_at, updated_at FROM workout_templates ORDER BY updated_at DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_template_exercises(
    state: State<'_, DbState>,
    template_id: i64,
) -> Result<Vec<WorkoutTemplateExercise>, String> {
    let pool = &state.0;
    let exercises = sqlx::query_as::<_, WorkoutTemplateExercise>(
        "SELECT id, template_id, exercise_id, exercise_name, default_sets, default_reps, default_weight, order_index 
         FROM workout_template_exercises WHERE template_id = ? ORDER BY order_index, id"
    )
    .bind(template_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(exercises)
}

#[tauri::command]
pub async fn create_workout_template(
    state: State<'_, DbState>,
    name: String,
    exercises: Vec<TemplateExerciseInput>,
) -> Result<WorkoutTemplate, String> {
    let pool = &state.0;
    
    // Create the template
    let template = sqlx::query_as::<_, WorkoutTemplate>(
        "INSERT INTO workout_templates (user_id, name) VALUES (1, ?) 
         RETURNING id, user_id, name, created_at, updated_at"
    )
    .bind(&name)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    // Add exercises to the template
    for (idx, ex) in exercises.iter().enumerate() {
        sqlx::query(
            "INSERT INTO workout_template_exercises (template_id, exercise_id, exercise_name, default_sets, default_reps, default_weight, order_index) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(template.id)
        .bind(ex.exercise_id)
        .bind(&ex.exercise_name)
        .bind(ex.default_sets)
        .bind(ex.default_reps)
        .bind(ex.default_weight)
        .bind(ex.order_index.unwrap_or(idx as i64))
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }
    
    Ok(template)
}

#[tauri::command]
pub async fn update_workout_template(
    state: State<'_, DbState>,
    id: i64,
    name: String,
    exercises: Vec<TemplateExerciseInput>,
) -> Result<WorkoutTemplate, String> {
    let pool = &state.0;
    
    // Update the template name and updated_at
    let template = sqlx::query_as::<_, WorkoutTemplate>(
        "UPDATE workout_templates SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? 
         RETURNING id, user_id, name, created_at, updated_at"
    )
    .bind(&name)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    // Delete existing exercises
    sqlx::query("DELETE FROM workout_template_exercises WHERE template_id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    // Add new exercises
    for (idx, ex) in exercises.iter().enumerate() {
        sqlx::query(
            "INSERT INTO workout_template_exercises (template_id, exercise_id, exercise_name, default_sets, default_reps, default_weight, order_index) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(template.id)
        .bind(ex.exercise_id)
        .bind(&ex.exercise_name)
        .bind(ex.default_sets)
        .bind(ex.default_reps)
        .bind(ex.default_weight)
        .bind(ex.order_index.unwrap_or(idx as i64))
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }
    
    Ok(template)
}

#[tauri::command]
pub async fn delete_workout_template(state: State<'_, DbState>, id: i64) -> Result<bool, String> {
    let pool = &state.0;
    let result = sqlx::query("DELETE FROM workout_templates WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("Workout template not found".to_string());
    }

    Ok(true)
}
