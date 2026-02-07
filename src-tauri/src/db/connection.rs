use sqlx::{Pool, Sqlite, sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode}};
use std::path::PathBuf;
use std::str::FromStr;

pub async fn establish_pool(db_path: PathBuf) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}?mode=rwc", db_path.display()))?
        .busy_timeout(std::time::Duration::from_secs(3))
        .journal_mode(SqliteJournalMode::Wal);

    SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await
}

pub async fn ensure_default_user(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO users (id, name, email) VALUES (1, 'Default User', NULL)")
        .execute(pool)
        .await?
        ;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn establish_pool_sets_busy_timeout_and_wal() {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        path.push(format!("life-os-test-{}.sqlite", nanos));

        let pool = establish_pool(path.clone()).await.unwrap();

        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");

        let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(busy_timeout >= 2000 && busy_timeout <= 5000);

        drop(pool);
        let _ = std::fs::remove_file(path);
    }
}
