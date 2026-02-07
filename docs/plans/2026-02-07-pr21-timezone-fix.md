# PR21 Timezone Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Resolve the calendar command rebase conflict and ensure datetime normalization preserves local time without UTC shifts.

**Architecture:** Keep the Tauri command signature returning `ApiError`, retain a test helper for direct pool usage, and update `parse_datetime_to_rfc3339` to interpret naive datetimes in local time. Add focused unit tests that compare normalized output against `Local` conversion to prove no time shift.

**Tech Stack:** Rust, chrono, sqlx (sqlite), tokio.

### Task 1: Resolve calendar rebase conflict cleanly

**Files:**
- Modify: `src-tauri/src/commands/calendar.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn calendar_items_emit_rfc3339_times() {
    let pool = setup_db().await;

    sqlx::query(
        r#"INSERT INTO calendar_events (user_id, title, start_at, end_at, category)
           VALUES (1, 'Test Event', '2026-02-07T09:30:00', '2026-02-07T10:30:00', 'busy')"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let items = get_calendar_items_for_test(
        &pool,
        CalendarQuery {
            start_date: "2026-02-07".to_string(),
            end_date: "2026-02-07".to_string(),
            include_assignments: Some(false),
            include_exams: Some(false),
        },
    )
    .await
    .unwrap();

    let event = items
        .into_iter()
        .find(|item| item.source == "calendar_event")
        .expect("calendar event not found");

    assert!(event.start_at.contains('Z') || event.start_at.contains('+'));
    assert!(event.end_at.contains('Z') || event.end_at.contains('+'));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test calendar_items_emit_rfc3339_times`
Expected: FAIL because `calendar.rs` has merge conflict markers or missing helper.

**Step 3: Write minimal implementation**

```rust
#[tauri::command]
pub async fn get_calendar_items(
    state: State<'_, DbState>,
    query: CalendarQuery,
) -> Result<Vec<CalendarItem>, ApiError> {
    get_calendar_items_for_test(&state.0, query).await
}

async fn get_calendar_items_for_test(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    query: CalendarQuery,
) -> Result<Vec<CalendarItem>, ApiError> {
    // existing implementation body
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test calendar_items_emit_rfc3339_times`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/commands/calendar.rs
git commit -m "fix: resolve calendar command conflict"
```

### Task 2: Add failing test for local-time preservation

**Files:**
- Modify: `src-tauri/src/utils.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn parses_naive_datetime_as_local_time() {
    let input = "2026-02-07T09:30:00";
    let naive = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S").unwrap();
    let expected = chrono::Local
        .from_local_datetime(&naive)
        .single()
        .expect("expected local time")
        .to_rfc3339();

    let parsed = parse_datetime_to_rfc3339(input).unwrap();
    assert_eq!(parsed, expected);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test parses_naive_datetime_as_local_time`
Expected: FAIL (current implementation treats naive as UTC).

**Step 3: Write minimal implementation**

```rust
if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
    if let Some(local) = chrono::Local.from_local_datetime(&naive).single() {
        return Some(local.to_rfc3339());
    }
}

if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
    if let Some(local) = chrono::Local.from_local_datetime(&naive).single() {
        return Some(local.to_rfc3339());
    }
}

if let Ok(date) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
    let naive = date.and_hms_opt(0, 0, 0)?;
    if let Some(local) = chrono::Local.from_local_datetime(&naive).single() {
        return Some(local.to_rfc3339());
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test parses_naive_datetime_as_local_time`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/utils.rs
git commit -m "fix: preserve local time in datetime normalization"
```

### Task 3: Verify targeted suite and report

**Files:**
- Modify: `src-tauri/src/commands/calendar.rs`
- Modify: `src-tauri/src/utils.rs`

**Step 1: Run targeted tests**

Run: `cargo test calendar_items_emit_rfc3339_times parses_naive_datetime_as_local_time`
Expected: PASS

**Step 2: Commit (if required by workflow)**

```bash
git status -sb
```

**Step 3: Report results**

Summarize conflict resolution, local-time normalization behavior, and test output.
