use tauri::State;
use crate::{
    DbState,
    error::ApiError,
    models::course_meeting::CourseMeeting,
    utils::is_valid_time,
};

#[derive(Debug, serde::Deserialize)]
pub struct CourseMeetingInput {
    pub course_id: i64,
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub meeting_type: Option<String>,
}

#[tauri::command]
pub async fn create_course_meeting(
    state: State<'_, DbState>,
    data: CourseMeetingInput,
) -> Result<CourseMeeting, ApiError> {
    let pool = &state.0;

    // Validate day_of_week
    if data.day_of_week < 0 || data.day_of_week > 6 {
        return Err(ApiError::validation(
            "day_of_week must be 0-6 (Sunday-Saturday)",
        ));
    }

    // Validate time format (basic HH:MM check)
    if !is_valid_time(&data.start_time) || !is_valid_time(&data.end_time) {
        return Err(ApiError::validation(
            "Invalid time format. Use HH:MM (24-hour)",
        ));
    }

    if data.start_time >= data.end_time {
        return Err(ApiError::validation("start_time must be before end_time"));
    }

    let meeting_type = data.meeting_type.unwrap_or_else(|| "lecture".to_string());

    let rec = sqlx::query_as::<_, CourseMeeting>(
        r#"INSERT INTO course_meetings (course_id, day_of_week, start_time, end_time, location, meeting_type)
           VALUES (?, ?, ?, ?, ?, ?)
           RETURNING *"#
    )
    .bind(data.course_id)
    .bind(data.day_of_week)
    .bind(&data.start_time)
    .bind(&data.end_time)
    .bind(&data.location)
    .bind(&meeting_type)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to create course meeting: {}", e);
        ApiError::from_sqlx(e, "Failed to create course meeting")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn get_course_meetings(
    state: State<'_, DbState>,
    course_id: Option<i64>,
) -> Result<Vec<CourseMeeting>, ApiError> {
    let pool = &state.0;

    let meetings = if let Some(cid) = course_id {
        sqlx::query_as::<_, CourseMeeting>(
            "SELECT * FROM course_meetings WHERE course_id = ? ORDER BY day_of_week, start_time"
        )
        .bind(cid)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, CourseMeeting>(
            "SELECT * FROM course_meetings ORDER BY day_of_week, start_time"
        )
        .fetch_all(pool)
        .await
    };

    meetings.map_err(|e| {
        log::error!("Failed to fetch course meetings: {}", e);
        ApiError::from_sqlx(e, "Failed to fetch course meetings")
    })
}

#[tauri::command]
pub async fn update_course_meeting(
    state: State<'_, DbState>,
    id: i64,
    data: CourseMeetingInput,
) -> Result<CourseMeeting, ApiError> {
    let pool = &state.0;

    if data.day_of_week < 0 || data.day_of_week > 6 {
        return Err(ApiError::validation(
            "day_of_week must be 0-6 (Sunday-Saturday)",
        ));
    }

    if !is_valid_time(&data.start_time) || !is_valid_time(&data.end_time) {
        return Err(ApiError::validation(
            "Invalid time format. Use HH:MM (24-hour)",
        ));
    }

    if data.start_time >= data.end_time {
        return Err(ApiError::validation("start_time must be before end_time"));
    }

    let rec = sqlx::query_as::<_, CourseMeeting>(
        r#"UPDATE course_meetings
           SET course_id = ?, day_of_week = ?, start_time = ?, end_time = ?,
               location = ?, meeting_type = ?
           WHERE id = ?
           RETURNING *"#
    )
    .bind(data.course_id)
    .bind(data.day_of_week)
    .bind(&data.start_time)
    .bind(&data.end_time)
    .bind(&data.location)
    .bind(&data.meeting_type)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to update course meeting {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to update course meeting")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn delete_course_meeting(
    state: State<'_, DbState>,
    id: i64,
) -> Result<bool, ApiError> {
    let pool = &state.0;

    let result = sqlx::query("DELETE FROM course_meetings WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete course meeting {}: {}", id, e);
            ApiError::from_sqlx(e, "Failed to delete course meeting")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Course meeting not found"));
    }

    Ok(true)
}
