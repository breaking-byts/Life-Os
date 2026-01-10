use sqlx::{Pool, Sqlite, migrate::MigrateError};

pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), MigrateError> {
    sqlx::migrate!("src/db/migrations").run(pool).await
}

