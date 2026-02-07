# Phase 6 Domain Model Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce domain enums for session type and exercise source with serde/sqlx string mapping plus TDD coverage.

**Architecture:** Add `SessionType` and `ExerciseSource` enums in their model modules with serde renames and sqlx Type/Encode/Decode so SQLite stores strings. Update model structs and command inputs to use enums, leaving SQL text unchanged. Add focused tests for serde and SQL round-trips.

**Tech Stack:** Rust, serde, sqlx (sqlite), tokio.

### Task 1: SessionType serde + sqlx tests (RED)

**Files:**
- Modify: `src-tauri/src/models/session.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn session_type_serde_roundtrip() {
    let json = serde_json::to_string(&SessionType::Study).unwrap();
    assert_eq!(json, "\"study\"");
    let parsed: SessionType = serde_json::from_str("\"practice\"").unwrap();
    assert_eq!(parsed, SessionType::Practice);
    assert!(serde_json::from_str::<SessionType>("\"invalid\"").is_err());
}

#[tokio::test]
async fn session_type_sqlx_roundtrip() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let value: SessionType = sqlx::query_scalar("SELECT ?")
        .bind(SessionType::Study)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(value, SessionType::Study);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test session_type_serde_roundtrip -- --nocapture`

Expected: FAIL with missing `SessionType` or missing sqlx/serde mapping.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Study,
    Practice,
}

impl sqlx::Type<sqlx::Sqlite> for SessionType {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for SessionType {
    fn encode_by_ref(&self, buf: &mut sqlx::sqlite::SqliteArgumentValue<'q>) -> sqlx::encode::IsNull {
        let value = match self {
            SessionType::Study => "study",
            SessionType::Practice => "practice",
        };
        <&str as sqlx::Encode<sqlx::Sqlite>>::encode_by_ref(&value, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for SessionType {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let raw = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match raw.as_str() {
            "study" => Ok(SessionType::Study),
            "practice" => Ok(SessionType::Practice),
            other => Err(format!("invalid session_type: {}", other).into()),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test session_type_serde_roundtrip -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/models/session.rs
git commit -m "test: add session type serde/sqlx coverage"
```

### Task 2: ExerciseSource serde + sqlx tests (RED)

**Files:**
- Modify: `src-tauri/src/models/exercise.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn exercise_source_serde_roundtrip() {
    let json = serde_json::to_string(&ExerciseSource::Wger).unwrap();
    assert_eq!(json, "\"wger\"");
    let parsed: ExerciseSource = serde_json::from_str("\"custom\"").unwrap();
    assert_eq!(parsed, ExerciseSource::Custom);
    assert!(serde_json::from_str::<ExerciseSource>("\"invalid\"").is_err());
}

#[tokio::test]
async fn exercise_source_sqlx_roundtrip() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let value: ExerciseSource = sqlx::query_scalar("SELECT ?")
        .bind(ExerciseSource::Custom)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(value, ExerciseSource::Custom);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test exercise_source_serde_roundtrip -- --nocapture`

Expected: FAIL with missing `ExerciseSource` or missing sqlx/serde mapping.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseSource {
    Wger,
    Custom,
}

impl sqlx::Type<sqlx::Sqlite> for ExerciseSource {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ExerciseSource {
    fn encode_by_ref(&self, buf: &mut sqlx::sqlite::SqliteArgumentValue<'q>) -> sqlx::encode::IsNull {
        let value = match self {
            ExerciseSource::Wger => "wger",
            ExerciseSource::Custom => "custom",
        };
        <&str as sqlx::Encode<sqlx::Sqlite>>::encode_by_ref(&value, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ExerciseSource {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let raw = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match raw.as_str() {
            "wger" => Ok(ExerciseSource::Wger),
            "custom" => Ok(ExerciseSource::Custom),
            other => Err(format!("invalid exercise source: {}", other).into()),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test exercise_source_serde_roundtrip -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/models/exercise.rs
git commit -m "test: add exercise source serde/sqlx coverage"
```

### Task 3: Replace model fields and command inputs with enums

**Files:**
- Modify: `src-tauri/src/models/session.rs`
- Modify: `src-tauri/src/models/exercise.rs`
- Modify: `src-tauri/src/commands/sessions.rs`
- Modify: `src-tauri/src/commands/exercises.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn exercise_source_decodes_from_text_column() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query("CREATE TABLE exercises_cache (source TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO exercises_cache (source) VALUES ('wger')")
        .execute(&pool)
        .await
        .unwrap();

    let value: ExerciseSource = sqlx::query_scalar("SELECT source FROM exercises_cache")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(value, ExerciseSource::Wger);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test exercise_source_decodes_from_text_column -- --nocapture`

Expected: FAIL until enum is used in model and sqlx mapping works for column extraction.

**Step 3: Write minimal implementation**

```rust
pub struct Session {
    pub session_type: SessionType,
    // ...
}

pub struct ExerciseCache {
    pub source: ExerciseSource,
    // ...
}

pub struct SessionInput {
    pub session_type: SessionType,
    // ...
}
```

Ensure binds use enum types; no SQL text changes.

**Step 4: Run test to verify it passes**

Run: `cargo test exercise_source_decodes_from_text_column -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/models/session.rs src-tauri/src/models/exercise.rs src-tauri/src/commands/sessions.rs src-tauri/src/commands/exercises.rs
git commit -m "feat: align session/exercise domain enums"
```

### Task 4: Targeted test run

**Files:**
- None

**Step 1: Run targeted tests**

Run: `cargo test session_type_serde_roundtrip session_type_sqlx_roundtrip exercise_source_serde_roundtrip exercise_source_sqlx_roundtrip exercise_source_decodes_from_text_column -- --nocapture`

Expected: PASS

**Step 2: Commit (if needed)**

Only if any additional fixes are required.
