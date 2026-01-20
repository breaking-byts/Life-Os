#[cfg(test)]
mod tests {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::time::Instant;
    use crate::commands::courses::get_courses_with_progress_inner;
    use std::str::FromStr;

    async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(false);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .expect("Failed to create pool");

        // Run migrations
        // We can't easily use the existing run_migrations because it uses the macro with a relative path
        // that might not work from here or we might need to expose it.
        // But since we are inside the crate, crate::db::migrations::run_migrations should work if the macro path is correct relative to Cargo.toml.

        crate::db::migrations::run_migrations(&pool).await.expect("Failed to run migrations");

        // Ensure default user
        crate::db::connection::ensure_default_user(&pool).await.expect("Failed to create user");

        pool
    }

    #[tokio::test]
    async fn benchmark_get_courses_with_progress_perf() {
        let pool = setup_db().await;

        // 1. Populate DB
        println!("Populating database...");

        // Insert 50 courses
        for i in 0..50 {
            sqlx::query("INSERT INTO courses (user_id, name, created_at) VALUES (1, ?, datetime('now', ?))")
                .bind(format!("Course {}", i))
                .bind(format!("-{} days", i))
                .execute(&pool)
                .await
                .unwrap();
        }

        // Insert 50 sessions per course (2500 total)
        // Some this week, some older
        for i in 1..=50 { // course ids
            for j in 0..50 {
                let date_modifier = if j % 2 == 0 { "-3 days" } else { "-30 days" };
                sqlx::query(
                    "INSERT INTO sessions (user_id, session_type, reference_type, reference_id, started_at, duration_minutes)
                     VALUES (1, 'study', 'course', ?, datetime('now', ?), ?)"
                )
                .bind(i)
                .bind(date_modifier)
                .bind(60)
                .execute(&pool)
                .await
                .unwrap();
            }
        }

        // Insert 20 assignments per course (1000 total)
        // Some upcoming, some overdue
        for i in 1..=50 {
            for j in 0..20 {
                let date_modifier = if j % 2 == 0 { "+3 days" } else { "-3 days" };
                sqlx::query(
                    "INSERT INTO assignments (course_id, title, due_date, is_completed)
                     VALUES (?, ?, datetime('now', ?), 0)"
                )
                .bind(i)
                .bind(format!("Assignment {}", j))
                .bind(date_modifier)
                .execute(&pool)
                .await
                .unwrap();
            }
        }

        println!("Database populated. Running benchmark...");

        // Warmup
        let _ = get_courses_with_progress_inner(&pool).await;

        // Measure
        let start = Instant::now();
        let result = get_courses_with_progress_inner(&pool).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        let courses = result.unwrap();
        assert_eq!(courses.len(), 50);

        println!("BENCHMARK: get_courses_with_progress took {:?} for 50 courses", duration);
    }
}
