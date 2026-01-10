use tauri::{Manager, State};

use crate::{db::migrations::run_migrations, DbState};

#[tauri::command]
pub async fn get_db_path(app: tauri::AppHandle) -> Result<String, String> {
    let mut path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    path.push("life-os.sqlite");
    Ok(path.to_string_lossy().to_string())
}

/// Dangerous: deletes the local sqlite database file and recreates it.
/// Intended only for local development / recovery.
/// This command is only available in debug builds.
#[cfg(debug_assertions)]
#[tauri::command]
pub async fn reset_local_db(app: tauri::AppHandle) -> Result<bool, String> {
    log::warn!("reset_local_db called - this is a destructive operation");
    let mut path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    path.push("life-os.sqlite");

    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }

    // Reconnect + rerun migrations by opening a fresh pool via sqlx directly.
    let url = format!("sqlite:{}", path.to_string_lossy());
    let pool = sqlx::SqlitePool::connect(&url)
        .await
        .map_err(|e| e.to_string())?;
    run_migrations(&pool).await.map_err(|e| e.to_string())?;

    Ok(true)
}

/// Stub for release builds - returns error if called
#[cfg(not(debug_assertions))]
#[tauri::command]
pub async fn reset_local_db(_app: tauri::AppHandle) -> Result<bool, String> {
    Err("This operation is only available in development builds".to_string())
}

/// Clears just the exercises cache (keeps everything else).
#[tauri::command]
pub async fn clear_exercises_cache(state: State<'_, DbState>) -> Result<i64, String> {
    let pool = &state.0;
    let res = sqlx::query("DELETE FROM exercises_cache")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.rows_affected() as i64)
}

/// Debug: get exercise cache stats
#[tauri::command]
pub async fn get_exercise_cache_stats(state: State<'_, DbState>) -> Result<serde_json::Value, String> {
    let pool = &state.0;
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exercises_cache")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    let sample: Vec<(i64, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, source FROM exercises_cache ORDER BY id LIMIT 5"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "count": count,
        "sample": sample.iter().map(|(id, name, source)| {
            serde_json::json!({"id": id, "name": name, "source": source})
        }).collect::<Vec<_>>()
    }))
}
