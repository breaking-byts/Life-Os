use std::path::PathBuf;

use tauri::Manager;

mod db;
mod models;
mod commands;
mod agent;
mod services;

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
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::courses::create_course,
      commands::courses::get_courses,
      commands::courses::get_course,
      commands::courses::update_course,
      commands::courses::delete_course,
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
       commands::debug::get_db_path,
       commands::debug::reset_local_db,
       commands::debug::clear_exercises_cache,
       commands::debug::get_exercise_cache_stats,
       agent::insights::get_insights,

    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
