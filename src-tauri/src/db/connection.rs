use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

pub async fn establish_pool(db_path: PathBuf) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite:{}?mode=rwc", db_path.display()))
        .await
}

pub async fn ensure_default_user(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO users (id, name, email) VALUES (1, 'Default User', NULL)")
        .execute(pool)
        .await?
        ;
    Ok(())
}

