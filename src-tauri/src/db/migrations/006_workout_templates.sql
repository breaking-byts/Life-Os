-- Add name field to workouts
ALTER TABLE workouts ADD COLUMN name TEXT;

-- Workout templates table
CREATE TABLE IF NOT EXISTS workout_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Template exercises (stores the exercises for each template)
CREATE TABLE IF NOT EXISTS workout_template_exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER NOT NULL,
    exercise_id INTEGER,
    exercise_name TEXT NOT NULL,
    default_sets INTEGER,
    default_reps INTEGER,
    default_weight REAL,
    order_index INTEGER DEFAULT 0,
    FOREIGN KEY (template_id) REFERENCES workout_templates(id) ON DELETE CASCADE,
    FOREIGN KEY (exercise_id) REFERENCES exercises_cache(id)
);

CREATE INDEX IF NOT EXISTS idx_template_exercises ON workout_template_exercises(template_id);
CREATE INDEX IF NOT EXISTS idx_workout_templates_user ON workout_templates(user_id);
