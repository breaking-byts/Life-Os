use std::path::PathBuf;

use tauri::Manager;

mod db;
mod models;
mod commands;
mod agent;
mod ml;
mod services;
mod utils;
mod error;
#[cfg(test)]
mod error_test;

use db::connection::establish_pool;
use db::migrations::run_migrations;

pub struct DbState(pub sqlx::Pool<sqlx::Sqlite>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().build())
    .setup(|app| {

      let app_handle = app.handle().clone();
      tauri::async_runtime::block_on(async move {
        let mut app_dir = app_handle
          .path()
          .app_config_dir()
          .unwrap_or_else(|_| PathBuf::from("."));
        app_dir.push("life-os.sqlite");

        log::info!("SQLite DB path: {}", app_dir.to_string_lossy());

        let pool = establish_pool(app_dir).await.expect("failed to connect to sqlite");
        run_migrations(&pool).await.expect("failed to run migrations");
        crate::db::connection::ensure_default_user(&pool)
          .await
          .expect("failed to ensure default user");

        app_handle.manage(DbState(pool));
        app_handle.manage(commands::google_calendar::GoogleState::default());
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::courses::create_course,
      commands::courses::get_courses,
      commands::courses::get_course,
      commands::courses::update_course,
      commands::courses::delete_course,
      commands::courses::get_courses_with_progress,
      commands::courses::get_course_analytics,
      commands::exams::create_exam,
      commands::exams::get_exams,
      commands::exams::get_exam,
      commands::exams::update_exam,
      commands::exams::delete_exam,
      commands::exams::get_upcoming_exams,
      commands::course_meetings::create_course_meeting,
      commands::course_meetings::get_course_meetings,
      commands::course_meetings::update_course_meeting,
      commands::course_meetings::delete_course_meeting,
      // Calendar Events
      commands::calendar_events::create_calendar_event,
      commands::calendar_events::get_calendar_events,
      commands::calendar_events::get_calendar_event,
      commands::calendar_events::update_calendar_event,
      commands::calendar_events::delete_calendar_event,
      // Weekly Tasks
      commands::weekly_tasks::create_weekly_task,
      commands::weekly_tasks::get_weekly_tasks,
      commands::weekly_tasks::update_weekly_task,
      commands::weekly_tasks::toggle_weekly_task,
      commands::weekly_tasks::delete_weekly_task,
      // Week Plan Blocks
      commands::week_plan_blocks::create_week_plan_block,
      commands::week_plan_blocks::get_week_plan_blocks,
      commands::week_plan_blocks::update_week_plan_block,
      commands::week_plan_blocks::accept_week_plan_block,
      commands::week_plan_blocks::lock_week_plan_block,
      commands::week_plan_blocks::delete_week_plan_block,
      commands::week_plan_blocks::clear_suggested_blocks,
      commands::week_plan_blocks::bulk_create_plan_blocks,
      // Calendar Aggregation
      commands::calendar::get_calendar_items,
      commands::assignments::create_assignment,
      commands::assignments::get_assignments,
      commands::assignments::update_assignment,
      commands::assignments::delete_assignment,
      commands::assignments::toggle_assignment,
      commands::sessions::start_session,
      commands::sessions::end_session,
      commands::sessions::get_sessions,
      commands::skills::create_skill,
      commands::skills::get_skills,
      commands::skills::update_skill,
      commands::skills::delete_skill,
      commands::practice::log_practice,
      commands::practice::get_practice_logs,
      commands::workouts::create_workout,
      commands::workouts::get_workouts,
      commands::workouts::get_workout,
      commands::workouts::delete_workout,
      commands::workout_exercises::add_exercise_to_workout,
      commands::workout_exercises::update_workout_exercise,
      commands::workout_exercises::remove_exercise,
      commands::workout_exercises::get_workout_exercises,
      commands::workouts::update_workout,
      commands::workout_templates::get_workout_templates,
      commands::workout_templates::get_template_exercises,
      commands::workout_templates::create_workout_template,
      commands::workout_templates::update_workout_template,
      commands::workout_templates::delete_workout_template,
      commands::exercises::search_exercises,
      commands::exercises::fetch_and_cache_exercises,
      commands::exercises::create_custom_exercise,
      commands::checkins::create_checkin,
      commands::checkins::get_today_checkin,
      commands::checkins::get_checkins,
      commands::weekly_reviews::create_weekly_review,
      commands::weekly_reviews::get_weekly_reviews,
       commands::analytics::get_stats,
       commands::analytics::get_streaks,
       commands::analytics::get_user_settings,
       commands::analytics::update_user_settings,
       commands::analytics::get_detailed_stats,
       commands::analytics::get_workout_heatmap,
       commands::analytics::get_personal_records,
       commands::analytics::check_and_update_prs,
       commands::analytics::get_achievements,
       commands::analytics::check_achievements,
       commands::debug::get_db_path,
       commands::debug::reset_local_db,
       commands::debug::clear_exercises_cache,
       commands::debug::get_exercise_cache_stats,
       agent::insights::get_insights,
       agent::insights::record_insight_feedback,
       agent::insights::run_pattern_analysis,
       agent::insights::get_user_profile,
       // Intelligence Agent commands
       commands::intelligence::get_agent_recommendations,
       commands::intelligence::get_agent_recommendation,
       commands::intelligence::record_recommendation_feedback,
       commands::intelligence::record_action_completed,
       commands::intelligence::get_agent_status,
       commands::intelligence::get_rich_context,
       commands::intelligence::get_big_three,
       commands::intelligence::set_big_three,
       commands::intelligence::complete_big_three,
       commands::intelligence::run_agent_maintenance,
       commands::intelligence::get_feature_names,
       commands::intelligence::search_similar_experiences,
       commands::intelligence::set_reward_weights,
       commands::intelligence::set_exploration_rate,
       // Google Calendar sync
       commands::google_calendar::set_google_client_id,
       commands::google_calendar::google_oauth_begin,
       commands::google_calendar::google_oauth_complete,
       commands::google_calendar::google_sync_now,
       commands::google_calendar::get_google_sync_status,
       commands::google_calendar::disconnect_google,

    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
