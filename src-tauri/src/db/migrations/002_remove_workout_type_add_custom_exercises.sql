-- Migration 002: remove workout_type from workouts, and extend exercises_cache for custom exercises

PRAGMA foreign_keys=OFF;

-- Rebuild workouts table without workout_type
CREATE TABLE IF NOT EXISTS workouts_new (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL DEFAULT 1,
  duration_minutes INTEGER,
  notes TEXT,
  logged_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

INSERT INTO workouts_new (id, user_id, duration_minutes, notes, logged_at)
SELECT id, user_id, duration_minutes, notes, logged_at
FROM workouts;

DROP TABLE workouts;
ALTER TABLE workouts_new RENAME TO workouts;

-- Recreate indexes lost during rebuild
CREATE INDEX IF NOT EXISTS idx_workouts_user ON workouts(user_id);
CREATE INDEX IF NOT EXISTS idx_workouts_logged ON workouts(logged_at);

-- Extend exercises_cache for custom exercises (id is stable)
ALTER TABLE exercises_cache ADD COLUMN source TEXT NOT NULL DEFAULT 'wger';
ALTER TABLE exercises_cache ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- Ensure we have a fast case-insensitive name index
CREATE INDEX IF NOT EXISTS idx_exercises_cache_name_lower ON exercises_cache(lower(name));

-- Prevent duplicates for custom entries by lowering name
CREATE UNIQUE INDEX IF NOT EXISTS idx_exercises_cache_custom_name_unique
  ON exercises_cache(lower(name))
  WHERE source = 'custom';

PRAGMA foreign_keys=ON;

