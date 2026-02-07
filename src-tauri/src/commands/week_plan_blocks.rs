use tauri::State;
use crate::{DbState, error::ApiError, models::week_plan_block::WeekPlanBlock};

#[derive(Debug, serde::Deserialize)]
pub struct WeekPlanBlockInput {
    #[serde(default)]
    pub user_id: Option<i64>,
    pub week_start_date: String,
    pub start_at: String,
    pub end_at: String,
    pub block_type: String,
    #[serde(default)]
    pub course_id: Option<i64>,
    #[serde(default)]
    pub weekly_task_id: Option<i64>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub rationale_json: Option<String>,
}

const VALID_BLOCK_TYPES: &[&str] = &["study", "assignment", "exam_prep", "break", "weekly_task"];
const VALID_STATUSES: &[&str] = &["suggested", "accepted", "locked"];

fn validate_block_type(block_type: &str) -> Result<(), ApiError> {
    if !VALID_BLOCK_TYPES.contains(&block_type) {
        return Err(ApiError::validation(format!(
            "Invalid block_type '{}'. Must be one of: {:?}",
            block_type, VALID_BLOCK_TYPES
        )));
    }
    Ok(())
}

fn validate_status(status: &str) -> Result<(), ApiError> {
    if !VALID_STATUSES.contains(&status) {
        return Err(ApiError::validation(format!(
            "Invalid status '{}'. Must be one of: {:?}",
            status, VALID_STATUSES
        )));
    }
    Ok(())
}

#[tauri::command]
pub async fn create_week_plan_block(
    state: State<'_, DbState>,
    data: WeekPlanBlockInput,
) -> Result<WeekPlanBlock, ApiError> {
    let pool = &state.0;

    // Validate block_type
    validate_block_type(&data.block_type)?;

    // Validate status if provided
    let status = data.status.unwrap_or_else(|| "suggested".to_string());
    validate_status(&status)?;

    let user_id = data.user_id.unwrap_or(1);

    let rec = sqlx::query_as::<_, WeekPlanBlock>(
        r#"INSERT INTO week_plan_blocks (user_id, week_start_date, start_at, end_at, block_type, course_id, weekly_task_id, title, status, rationale_json)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING *"#
    )
    .bind(user_id)
    .bind(&data.week_start_date)
    .bind(&data.start_at)
    .bind(&data.end_at)
    .bind(&data.block_type)
    .bind(data.course_id)
    .bind(data.weekly_task_id)
    .bind(&data.title)
    .bind(&status)
    .bind(&data.rationale_json)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to create week plan block: {}", e);
        ApiError::from_sqlx(e, "Failed to create week plan block")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn get_week_plan_blocks(
    state: State<'_, DbState>,
    week_start_date: String,
) -> Result<Vec<WeekPlanBlock>, ApiError> {
    let pool = &state.0;

    let blocks = sqlx::query_as::<_, WeekPlanBlock>(
        "SELECT * FROM week_plan_blocks WHERE week_start_date = ? ORDER BY start_at"
    )
    .bind(&week_start_date)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch week plan blocks: {}", e);
        ApiError::from_sqlx(e, "Failed to fetch week plan blocks")
    })?;

    Ok(blocks)
}

#[tauri::command]
pub async fn update_week_plan_block(
    state: State<'_, DbState>,
    id: i64,
    data: WeekPlanBlockInput,
) -> Result<WeekPlanBlock, ApiError> {
    let pool = &state.0;

    // Validate block_type
    validate_block_type(&data.block_type)?;

    // Validate status if provided
    if let Some(ref status) = data.status {
        validate_status(status)?;
    }

    let rec = sqlx::query_as::<_, WeekPlanBlock>(
        r#"UPDATE week_plan_blocks
           SET user_id = COALESCE(?, user_id),
               week_start_date = COALESCE(?, week_start_date),
               start_at = COALESCE(?, start_at),
               end_at = COALESCE(?, end_at),
               block_type = COALESCE(?, block_type),
               course_id = COALESCE(?, course_id),
               weekly_task_id = COALESCE(?, weekly_task_id),
               title = COALESCE(?, title),
               status = COALESCE(?, status),
               rationale_json = COALESCE(?, rationale_json)
           WHERE id = ?
           RETURNING *"#
    )
    .bind(data.user_id)
    .bind(&data.week_start_date)
    .bind(&data.start_at)
    .bind(&data.end_at)
    .bind(&data.block_type)
    .bind(data.course_id)
    .bind(data.weekly_task_id)
    .bind(&data.title)
    .bind(&data.status)
    .bind(&data.rationale_json)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to update week plan block {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to update week plan block")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn accept_week_plan_block(
    state: State<'_, DbState>,
    id: i64,
) -> Result<WeekPlanBlock, ApiError> {
    let pool = &state.0;

    let rec = sqlx::query_as::<_, WeekPlanBlock>(
        r#"UPDATE week_plan_blocks
           SET status = 'accepted'
           WHERE id = ?
           RETURNING *"#
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to accept week plan block {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to accept week plan block")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn lock_week_plan_block(
    state: State<'_, DbState>,
    id: i64,
) -> Result<WeekPlanBlock, ApiError> {
    let pool = &state.0;

    let rec = sqlx::query_as::<_, WeekPlanBlock>(
        r#"UPDATE week_plan_blocks
           SET status = 'locked'
           WHERE id = ?
           RETURNING *"#
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to lock week plan block {}: {}", id, e);
        ApiError::from_sqlx(e, "Failed to lock week plan block")
    })?;

    Ok(rec)
}

#[tauri::command]
pub async fn delete_week_plan_block(
    state: State<'_, DbState>,
    id: i64,
) -> Result<bool, ApiError> {
    let pool = &state.0;

    let result = sqlx::query("DELETE FROM week_plan_blocks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete week plan block {}: {}", id, e);
            ApiError::from_sqlx(e, "Failed to delete week plan block")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Week plan block not found"));
    }

    Ok(true)
}

#[tauri::command]
pub async fn clear_suggested_blocks(
    state: State<'_, DbState>,
    week_start_date: String,
) -> Result<i64, ApiError> {
    let pool = &state.0;

    let result = sqlx::query("DELETE FROM week_plan_blocks WHERE week_start_date = ? AND status = 'suggested'")
        .bind(&week_start_date)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to clear suggested blocks for week {}: {}", week_start_date, e);
            ApiError::from_sqlx(e, "Failed to clear suggested blocks")
        })?;

    Ok(result.rows_affected() as i64)
}

#[tauri::command]
pub async fn bulk_create_plan_blocks(
    state: State<'_, DbState>,
    blocks: Vec<WeekPlanBlockInput>,
) -> Result<Vec<WeekPlanBlock>, ApiError> {
    let pool = &state.0;

    // Validate all blocks first
    for (i, block) in blocks.iter().enumerate() {
        validate_block_type(&block.block_type)
            .map_err(|e| ApiError::validation(format!("Block {}: {}", i, e.message)))?;
        if let Some(ref status) = block.status {
            validate_status(status)
                .map_err(|e| ApiError::validation(format!("Block {}: {}", i, e.message)))?;
        }
    }

    if blocks.is_empty() {
        return Ok(Vec::new());
    }

    let mut transaction = pool.begin().await.map_err(|e| {
        log::error!("Failed to start transaction for bulk create week plan blocks: {}", e);
        ApiError::from_sqlx(e, "Failed to bulk create week plan blocks")
    })?;

    let mut qb = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "INSERT INTO week_plan_blocks (user_id, week_start_date, start_at, end_at, block_type, course_id, weekly_task_id, title, status, rationale_json) ",
    );

    qb.push_values(blocks.iter(), |mut b, data| {
        let user_id = data.user_id.unwrap_or(1);
        let status = data.status.clone().unwrap_or_else(|| "suggested".to_string());

        b.push_bind(user_id)
            .push_bind(&data.week_start_date)
            .push_bind(&data.start_at)
            .push_bind(&data.end_at)
            .push_bind(&data.block_type)
            .push_bind(data.course_id)
            .push_bind(data.weekly_task_id)
            .push_bind(&data.title)
            .push_bind(status)
            .push_bind(&data.rationale_json);
    });

    qb.push(" RETURNING *");

    let created_blocks = qb
        .build_query_as::<WeekPlanBlock>()
        .fetch_all(&mut *transaction)
        .await
        .map_err(|e| {
            log::error!("Failed to bulk create week plan blocks: {}", e);
            ApiError::from_sqlx(e, "Failed to bulk create week plan blocks")
        })?;

    transaction.commit().await.map_err(|e| {
        log::error!("Failed to commit bulk create week plan blocks: {}", e);
        ApiError::from_sqlx(e, "Failed to bulk create week plan blocks")
    })?;

    Ok(created_blocks)
}
