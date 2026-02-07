use sqlx::SqlitePool;
use crate::agent::IntelligenceAgent;

#[tokio::test]
async fn test_redundant_context_saving() {
    // 1. Setup DB
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // 2. Run Migrations
    let m1 = include_str!("../db/migrations/001_initial.sql");
    let m4 = include_str!("../db/migrations/004_intelligence_agent.sql");

    // Split by ; and execute to ensure all statements run
    for query in m1.split(';') {
        if !query.trim().is_empty() {
            sqlx::query(query).execute(&pool).await.expect("Failed to run migration 001");
        }
    }
    for query in m4.split(';') {
         if !query.trim().is_empty() {
            sqlx::query(query).execute(&pool).await.expect("Failed to run migration 004");
        }
    }

    // 3. Seed Data
    // We need a user for foreign keys (though SQLite might not enforce them by default, it's safer)
    sqlx::query("INSERT INTO users (name) VALUES ('Test User')")
        .execute(&pool).await.unwrap();

    // 4. Call get_recommendations
    // We ask for 3 recommendations
    let recs = IntelligenceAgent::get_recommendations(&pool, 3).await;

    assert!(recs.is_ok(), "get_recommendations failed: {:?}", recs.err());
    let recs = recs.unwrap();
    assert_eq!(recs.len(), 3, "Expected 3 recommendations");

    // 5. Count agent_rich_context rows
    // Optimized logic:
    // 1 save before loop
    // 0 saves inside loop (reusing ID)
    // Total = 1
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_rich_context")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count, 1, "Expected 1 context snapshot (optimized)");
}
