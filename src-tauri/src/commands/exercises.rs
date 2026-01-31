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
    
    bulk_insert_exercises(pool, exercises).await
}

async fn bulk_insert_exercises(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    exercises: Vec<wger::ExerciseData>,
) -> Result<usize, String> {
    if exercises.is_empty() {
        return Ok(0);
    }
    
    let count = exercises.len();
    let mut query_builder: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new(
        "INSERT INTO exercises_cache (wger_id, name, category, muscles, equipment, description, cached_at, source)"
    );

    query_builder.push_values(exercises, |mut b, ex| {
        b.push_bind(ex.wger_id);
        b.push_bind(ex.name);
        b.push_bind(ex.category);
        b.push_bind(ex.muscles);
        b.push_bind(ex.equipment);
        b.push_bind(ex.description);
        b.push("CURRENT_TIMESTAMP");
        b.push_bind("wger");
    });

    query_builder.push(
        " ON CONFLICT(wger_id) DO UPDATE SET
            name = excluded.name,
            category = excluded.category,
            muscles = excluded.muscles,
            equipment = excluded.equipment,
            description = excluded.description,
            cached_at = CURRENT_TIMESTAMP,
            source = 'wger'"
    );

    let query = query_builder.build();
    query.execute(pool).await.map_err(|e| e.to_string())?;
    
    log::info!("Successfully cached {} exercises.", count);
    Ok(count)
}



#[cfg(test)]
mod performance_tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use crate::services::wger::ExerciseData;
    use std::time::Instant;

    // Helper to setup DB
    async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;

        // Disable FKs to avoid migration issues with missing default user
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(false);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to connect to in-memory DB");

        crate::db::migrations::run_migrations(&pool).await.expect("Failed to run migrations");

        pool
    }

    fn generate_dummy_exercises(count: usize) -> Vec<ExerciseData> {
        let mut exercises = Vec::with_capacity(count);
        for i in 0..count {
            exercises.push(ExerciseData {
                wger_id: i as i64,
                name: format!("Exercise {}", i),
                category: Some("General".to_string()),
                muscles: Some("Full Body".to_string()),
                equipment: Some("Bodyweight".to_string()),
                description: Some("A dummy exercise".to_string()),
            });
        }
        exercises
    }

    // Legacy insert loop (exact copy of original logic)
    async fn legacy_insert_loop(pool: &sqlx::Pool<sqlx::Sqlite>, exercises: &[ExerciseData]) -> Result<(), String> {
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
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_legacy_insert() {
        let pool = setup_db().await;
        let count = 1000;
        let exercises = generate_dummy_exercises(count);

        let start = Instant::now();
        legacy_insert_loop(&pool, &exercises).await.expect("Legacy insert failed");
        let duration = start.elapsed();

        println!("Legacy insert of {} items took: {:?}", count, duration);
    }

    #[tokio::test]
    async fn benchmark_bulk_insert() {
        let pool = setup_db().await;
        let count = 1000;
        let exercises = generate_dummy_exercises(count);

        let start = Instant::now();
        bulk_insert_exercises(&pool, exercises).await.expect("Bulk insert failed");
        let duration = start.elapsed();

        println!("Bulk insert of {} items took: {:?}", count, duration);
    }

    #[tokio::test]
    async fn verify_insert_correctness() {
        let pool = setup_db().await;
        let exercises = vec![
            ExerciseData {
                wger_id: 1,
                name: "Test Ex 1".to_string(),
                category: Some("Cat 1".to_string()),
                muscles: Some("Muscles 1".to_string()),
                equipment: Some("Eq 1".to_string()),
                description: Some("Desc 1".to_string()),
            },
            ExerciseData {
                wger_id: 2,
                name: "Test Ex 2".to_string(),
                category: Some("Cat 2".to_string()),
                muscles: None,
                equipment: None,
                description: None,
            },
        ];

        bulk_insert_exercises(&pool, exercises).await.expect("Insert failed");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exercises_cache")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 2);

        let ex1: ExerciseCache = sqlx::query_as("SELECT * FROM exercises_cache WHERE wger_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(ex1.name, "Test Ex 1");
        assert_eq!(ex1.source, "wger");

        // Verify Update behavior
        let updates = vec![
            ExerciseData {
                wger_id: 1,
                name: "Updated Name".to_string(),
                category: Some("Cat 1".to_string()),
                muscles: Some("Muscles 1".to_string()),
                equipment: Some("Eq 1".to_string()),
                description: Some("Desc 1".to_string()),
            }
        ];

        bulk_insert_exercises(&pool, updates).await.expect("Update failed");

        let ex1_updated: ExerciseCache = sqlx::query_as("SELECT * FROM exercises_cache WHERE wger_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(ex1_updated.name, "Updated Name");
    }
}
