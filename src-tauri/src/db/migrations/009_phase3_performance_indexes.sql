CREATE INDEX IF NOT EXISTS idx_assignments_completed_due ON assignments(is_completed, due_date);
CREATE INDEX IF NOT EXISTS idx_sessions_type_reference_started ON sessions(session_type, reference_type, reference_id, started_at);
CREATE INDEX IF NOT EXISTS idx_workouts_logged_at ON workouts(logged_at);
CREATE INDEX IF NOT EXISTS idx_practice_logs_skill_logged_at ON practice_logs(skill_id, logged_at);
CREATE INDEX IF NOT EXISTS idx_check_ins_checked_in_at ON check_ins(checked_in_at);
CREATE INDEX IF NOT EXISTS idx_week_plan_blocks_week_start_at ON week_plan_blocks(week_start_date, start_at);
