use tauri::State;
use crate::DbState;
use serde::Serialize;

/// A unified calendar item for frontend rendering
#[derive(Debug, Serialize, Clone)]
pub struct CalendarItem {
    pub id: String,           // Prefixed: "cm_1", "ce_2", "wpb_3", "asgn_4", "exam_5"
    pub source: String,       // course_meeting, calendar_event, plan_block, assignment, exam
    pub title: String,
    pub start_at: String,     // ISO datetime
    pub end_at: String,       // ISO datetime
    pub all_day: bool,
    pub color: Option<String>,
    pub course_id: Option<i64>,
    pub course_name: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,  // For plan blocks: suggested/accepted/locked
    pub locked: bool,
    pub editable: bool,
    pub metadata_json: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CalendarQuery {
    pub start_date: String,   // ISO date (YYYY-MM-DD)
    pub end_date: String,     // ISO date (YYYY-MM-DD)
    #[serde(default)]
    pub include_assignments: Option<bool>,
    #[serde(default)]
    pub include_exams: Option<bool>,
}

#[tauri::command]
pub async fn get_calendar_items(
    state: State<'_, DbState>,
    query: CalendarQuery,
) -> Result<Vec<CalendarItem>, String> {
    let pool = &state.0;
    let mut items: Vec<CalendarItem> = Vec::new();

    // Parse dates for day-of-week calculations
    let start_date = chrono::NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")
        .map_err(|_| "Invalid start_date format")?;
    let end_date = chrono::NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")
        .map_err(|_| "Invalid end_date format")?;

    // 1. Course meetings (expand weekly recurrence)
    let meetings = sqlx::query_as::<_, (i64, i64, i64, String, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
        r#"SELECT cm.id, cm.course_id, cm.day_of_week, cm.start_time, cm.end_time,
                  cm.location, cm.meeting_type, c.name as course_name, c.color
           FROM course_meetings cm
           JOIN courses c ON c.id = cm.course_id
           WHERE c.is_active = 1"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for (id, course_id, day_of_week, start_time, end_time, location, meeting_type, course_name, color) in meetings {
        // Expand to each occurrence in the date range
        let mut current = start_date;
        while current <= end_date {
            use chrono::Datelike;
            if current.weekday().num_days_from_sunday() as i64 == day_of_week {
                let start_at = format!("{}T{}:00", current, start_time);
                let end_at = format!("{}T{}:00", current, end_time);

                let title = format!("{} - {}",
                    course_name.as_deref().unwrap_or("Course"),
                    meeting_type.as_deref().unwrap_or("Class")
                );

                items.push(CalendarItem {
                    id: format!("cm_{}_{}", id, current),
                    source: "course_meeting".to_string(),
                    title,
                    start_at,
                    end_at,
                    all_day: false,
                    color: color.clone(),
                    course_id: Some(course_id),
                    course_name: course_name.clone(),
                    category: Some("class".to_string()),
                    status: None,
                    locked: true,
                    editable: false,
                    metadata_json: location.as_ref().map(|l| format!(r#"{{"location":"{}"}}"#, l)),
                });
            }
            current += chrono::Duration::days(1);
        }
    }

    // 2. Calendar events (one-off and recurring)
    let events = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, Option<i64>)>(
        r#"SELECT id, title, start_at, end_at, rrule, start_time, end_time, category, locked
           FROM calendar_events"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for (id, title, start_at, end_at, rrule, start_time, end_time, category, locked) in events {
        if let Some(ref rule) = rrule {
            // Recurring event: parse rrule like "WEEKLY:0,2,4"
            if rule.starts_with("WEEKLY:") {
                let days_str = rule.strip_prefix("WEEKLY:").unwrap();
                let days: Vec<i64> = days_str.split(',')
                    .filter_map(|d| d.parse().ok())
                    .collect();

                let st = start_time.as_deref().unwrap_or("09:00");
                let et = end_time.as_deref().unwrap_or("10:00");

                let mut current = start_date;
                while current <= end_date {
                    use chrono::Datelike;
                    let dow = current.weekday().num_days_from_sunday() as i64;
                    if days.contains(&dow) {
                        items.push(CalendarItem {
                            id: format!("ce_{}_{}", id, current),
                            source: "calendar_event".to_string(),
                            title: title.clone(),
                            start_at: format!("{}T{}:00", current, st),
                            end_at: format!("{}T{}:00", current, et),
                            all_day: false,
                            color: None,
                            course_id: None,
                            course_name: None,
                            category: Some(category.clone()),
                            status: None,
                            locked: locked.unwrap_or(0) == 1,
                            editable: locked.unwrap_or(0) != 1,
                            metadata_json: None,
                        });
                    }
                    current += chrono::Duration::days(1);
                }
            }
        } else if let (Some(sa), Some(ea)) = (start_at, end_at) {
            // One-off event: check if in range
            let event_date = sa.split('T').next().unwrap_or("");
            if event_date >= query.start_date.as_str() && event_date <= query.end_date.as_str() {
                items.push(CalendarItem {
                    id: format!("ce_{}", id),
                    source: "calendar_event".to_string(),
                    title: title.clone(),
                    start_at: sa,
                    end_at: ea,
                    all_day: false,
                    color: None,
                    course_id: None,
                    course_name: None,
                    category: Some(category.clone()),
                    status: None,
                    locked: locked.unwrap_or(0) == 1,
                    editable: locked.unwrap_or(0) != 1,
                    metadata_json: None,
                });
            }
        }
    }

    // 3. Week plan blocks
    let blocks = sqlx::query_as::<_, (i64, String, String, String, Option<i64>, Option<String>, Option<String>, Option<String>)>(
        r#"SELECT wpb.id, wpb.start_at, wpb.end_at, wpb.block_type, wpb.course_id,
                  wpb.title, wpb.status, c.color
           FROM week_plan_blocks wpb
           LEFT JOIN courses c ON c.id = wpb.course_id
           WHERE date(wpb.start_at) >= ? AND date(wpb.start_at) <= ?"#
    )
    .bind(&query.start_date)
    .bind(&query.end_date)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for (id, start_at, end_at, block_type, course_id, title, status, color) in blocks {
        let display_title = title.unwrap_or_else(|| block_type.clone());
        let is_locked = status.as_deref() == Some("locked");

        items.push(CalendarItem {
            id: format!("wpb_{}", id),
            source: "plan_block".to_string(),
            title: display_title,
            start_at,
            end_at,
            all_day: false,
            color,
            course_id,
            course_name: None,
            category: Some(block_type),
            status,
            locked: is_locked,
            editable: !is_locked,
            metadata_json: None,
        });
    }

    // 4. Assignments (as deadline markers)
    if query.include_assignments.unwrap_or(true) {
        let assignments = sqlx::query_as::<_, (i64, String, String, i64, Option<String>)>(
            r#"SELECT a.id, a.title, a.due_date, a.course_id, c.color
               FROM assignments a
               JOIN courses c ON c.id = a.course_id
               WHERE a.is_completed = 0
                 AND date(a.due_date) >= ? AND date(a.due_date) <= ?"#
        )
        .bind(&query.start_date)
        .bind(&query.end_date)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for (id, title, due_date, course_id, color) in assignments {
            items.push(CalendarItem {
                id: format!("asgn_{}", id),
                source: "assignment".to_string(),
                title: format!("Due: {}", title),
                start_at: due_date.clone(),
                end_at: due_date,
                all_day: true,
                color,
                course_id: Some(course_id),
                course_name: None,
                category: Some("deadline".to_string()),
                status: None,
                locked: true,
                editable: false,
                metadata_json: None,
            });
        }
    }

    // 5. Exams (as all-day or timed events)
    if query.include_exams.unwrap_or(true) {
        let exams = sqlx::query_as::<_, (i64, String, Option<String>, Option<i64>, i64, Option<String>)>(
            r#"SELECT e.id, e.title, e.exam_date, e.duration_minutes, e.course_id, c.color
               FROM exams e
               JOIN courses c ON c.id = e.course_id
               WHERE date(e.exam_date) >= ? AND date(e.exam_date) <= ?"#
        )
        .bind(&query.start_date)
        .bind(&query.end_date)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for (id, title, exam_date, duration, course_id, color) in exams {
            if let Some(ed) = exam_date {
                let all_day = duration.is_none();
                let end_at = if let Some(dur) = duration {
                    // Calculate end time
                    if ed.contains('T') {
                        let dt = chrono::NaiveDateTime::parse_from_str(&ed, "%Y-%m-%dT%H:%M:%S")
                            .or_else(|_| chrono::NaiveDateTime::parse_from_str(&ed, "%Y-%m-%dT%H:%M"))
                            .ok();
                        if let Some(start) = dt {
                            let end = start + chrono::Duration::minutes(dur);
                            end.format("%Y-%m-%dT%H:%M:%S").to_string()
                        } else {
                            ed.clone()
                        }
                    } else {
                        ed.clone()
                    }
                } else {
                    ed.clone()
                };

                items.push(CalendarItem {
                    id: format!("exam_{}", id),
                    source: "exam".to_string(),
                    title: format!("Exam: {}", title),
                    start_at: ed,
                    end_at,
                    all_day,
                    color,
                    course_id: Some(course_id),
                    course_name: None,
                    category: Some("exam".to_string()),
                    status: None,
                    locked: true,
                    editable: false,
                    metadata_json: None,
                });
            }
        }
    }

    // Sort by start_at
    items.sort_by(|a, b| a.start_at.cmp(&b.start_at));

    Ok(items)
}
