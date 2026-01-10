use tauri::State;

use crate::{DbState, models::exercise::ExerciseCache, services::wger};

fn normalize_exercise_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}

/// Create a custom (local-only) exercise in the cache.
#[tauri::command]
pub async fn create_custom_exercise(
    state: State<'_, DbState>,
    name: String,
) -> Result<ExerciseCache, String> {
    let pool = &state.0;
    let normalized = normalize_exercise_name(&name);
    if normalized.is_empty() {
        return Err("Exercise name cannot be empty".to_string());
    }

    // Insert or return existing custom exercise with same (case-insensitive) name.
    let existing = sqlx::query_as::<_, ExerciseCache>(
        "SELECT * FROM exercises_cache WHERE source = 'custom' AND lower(name) = lower(?) LIMIT 1",
    )
    .bind(&normalized)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = existing {
        return Ok(row);
    }

    let rec = sqlx::query_as::<_, ExerciseCache>(
        r#"
        INSERT INTO exercises_cache (wger_id, name, category, muscles, equipment, description, cached_at, source, created_at)
        VALUES (NULL, ?, NULL, NULL, NULL, NULL, CURRENT_TIMESTAMP, 'custom', CURRENT_TIMESTAMP)
        RETURNING id, wger_id, name, category, muscles, equipment, description, cached_at, source, created_at
        "#,
    )
    .bind(&normalized)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rec)
}

/// Fetch exercises from wger API and cache them locally
#[tauri::command]
pub async fn fetch_and_cache_exercises(state: State<'_, DbState>) -> Result<usize, String> {
    let pool = &state.0;
    fetch_and_cache_internal(pool).await
}

/// Search exercises in local cache (name-only), auto-fetches if cache is empty.
///
/// Cache-first, async refresh behavior:
/// - Always returns local matches.
/// - If cache is empty and query is non-empty, it triggers a background fetch.
#[tauri::command]
pub async fn search_exercises(
    state: State<'_, DbState>,
    query: String,
) -> Result<Vec<ExerciseCache>, String> {
    let pool = &state.0;

    let normalized_query = query.trim().to_lowercase();
    if normalized_query.len() < 2 {
        return Ok(vec![]);
    }

    // If cache is empty, trigger fetch in background (but don't block search)
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exercises_cache")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    if count == 0 {
        let pool_clone = pool.clone();
        let q = normalized_query.clone();
        tauri::async_runtime::spawn(async move {
            log::info!("Exercise cache empty, fetching from wger...");
            if let Err(e) = fetch_and_cache_internal(&pool_clone).await {
                log::warn!("Failed to fetch exercises: {}", e);
            }
            // After fetching, opportunistically invalidate this query key by name prefix match.
            let prefix = format!("{}%", q);
            let contains = format!("%{}%", q);
            let _ = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM exercises_cache WHERE lower(name) LIKE ? OR lower(name) LIKE ?",
            )
            .bind(&prefix)
            .bind(&contains)
            .fetch_one(&pool_clone)
            .await;
        });
    }

    let prefix = format!("{}%", normalized_query);
    let contains = format!("%{}%", normalized_query);

    // Prefix-first query for responsiveness; fall back to contains for coverage.
    let rows = sqlx::query_as::<_, ExerciseCache>(
        r#"
        SELECT * FROM exercises_cache
        WHERE lower(name) LIKE ?
        ORDER BY name
        LIMIT 20
        "#,
    )
    .bind(&prefix)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if rows.len() >= 20 {
        return Ok(rows);
    }

    let remaining = 50_i64.saturating_sub(rows.len() as i64);
    let extra = sqlx::query_as::<_, ExerciseCache>(
        r#"
        SELECT * FROM exercises_cache
        WHERE lower(name) LIKE ?
          AND lower(name) NOT LIKE ?
        ORDER BY name
        LIMIT ?
        "#,
    )
    .bind(&contains)
    .bind(&prefix)
    .bind(remaining)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut combined = rows;
    combined.extend(extra);
    Ok(combined)
}

async fn fetch_and_cache_internal(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<usize, String> {
    log::info!("Fetching exercises from wger...");
    let exercises = wger::fetch_exercises().await?;
    let count = exercises.len();
    log::info!("Fetched {} exercises. Caching...", count);
    
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    
    for ex in exercises {
        sqlx::query(
            r#"INSERT OR REPLACE INTO exercises_cache 
               (wger_id, name, category, muscles, equipment, description, cached_at, source)
               VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, 'wger')"#
        )
        .bind(ex.wger_id)
        .bind(&ex.name)
        .bind(&ex.category)
        .bind(&ex.muscles)
        .bind(&ex.equipment)
        .bind(&ex.description)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    
    log::info!("Successfully cached {} exercises.", count);
    Ok(count)
}


