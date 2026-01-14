-- Comprehensive Revamp Migration
-- Adds: user_settings, academic enhancements, exercise PRs, achievements, skill target_hours

-- ============================================================================
-- USER SETTINGS
-- Global user preferences and targets
-- ============================================================================
CREATE TABLE IF NOT EXISTS user_settings (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL DEFAULT 1,
    weekly_workout_target INTEGER DEFAULT 3,
    weekly_active_skills_target INTEGER DEFAULT 5,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Insert default settings for user 1
INSERT OR IGNORE INTO user_settings (id, user_id, weekly_workout_target, weekly_active_skills_target)
VALUES (1, 1, 3, 5);

-- ============================================================================
-- ACADEMIC ENHANCEMENTS
-- Grade tracking, exams, and estimated time for assignments
-- ============================================================================

-- Add grade tracking to courses
ALTER TABLE courses ADD COLUMN current_grade REAL;
ALTER TABLE courses ADD COLUMN target_grade REAL DEFAULT 90.0;

-- Add estimated time to assignments
ALTER TABLE assignments ADD COLUMN estimated_minutes INTEGER;

-- Exams table
CREATE TABLE IF NOT EXISTS exams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    exam_date TIMESTAMP,
    location TEXT,
    duration_minutes INTEGER,
    notes TEXT,
    grade REAL,
    weight REAL,  -- percentage of final grade
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_exams_course ON exams(course_id);
CREATE INDEX IF NOT EXISTS idx_exams_date ON exams(exam_date);

-- ============================================================================
-- SKILLS ENHANCEMENTS
-- Add target_hours for mastery goal (separate from weekly target)
-- ============================================================================
ALTER TABLE skills ADD COLUMN target_hours REAL DEFAULT 100.0;

-- ============================================================================
-- PHYSICAL ENHANCEMENTS
-- Personal records and achievements
-- ============================================================================

-- Personal records table
CREATE TABLE IF NOT EXISTS exercise_prs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    exercise_id INTEGER,
    exercise_name TEXT NOT NULL,
    pr_type TEXT NOT NULL,  -- 'weight', 'volume', 'reps'
    value REAL NOT NULL,
    achieved_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    workout_id INTEGER,
    FOREIGN KEY (workout_id) REFERENCES workouts(id),
    FOREIGN KEY (exercise_id) REFERENCES exercises_cache(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_exercise_prs_user ON exercise_prs(user_id);
CREATE INDEX IF NOT EXISTS idx_exercise_prs_exercise ON exercise_prs(exercise_name);
CREATE INDEX IF NOT EXISTS idx_exercise_prs_type ON exercise_prs(pr_type);

-- Achievements table (app-wide achievements)
CREATE TABLE IF NOT EXISTS achievements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    achievement_type TEXT NOT NULL,  -- 'workout_streak', 'study_milestone', 'skill_level', 'consistency', etc.
    title TEXT NOT NULL,
    description TEXT,
    category TEXT,  -- 'physical', 'academic', 'skills', 'wellness'
    achieved_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT,  -- JSON for extra data like the streak count, hours, etc.
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_achievements_user ON achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_achievements_type ON achievements(achievement_type);
CREATE INDEX IF NOT EXISTS idx_achievements_category ON achievements(category);
