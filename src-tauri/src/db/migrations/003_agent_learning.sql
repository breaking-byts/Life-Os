-- Agent Learning Tables
-- Stores insight impressions, patterns, and user profile for adaptive learning

-- Insight impressions and feedback for contextual bandit learning
CREATE TABLE IF NOT EXISTS agent_insights (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  insight_type TEXT NOT NULL,           -- e.g., 'checkin_reminder', 'overdue_alert', 'productivity_tip'
  context_json TEXT,                    -- Encoded context at generation time (mood, energy, time, etc.)
  arm_index INTEGER,                    -- Which bandit arm was selected
  reward REAL DEFAULT 0,                -- Reward signal (0-1, based on user action)
  shown_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  dismissed_at TIMESTAMP,               -- When user dismissed without action
  acted_on BOOLEAN DEFAULT 0,           -- Did user take the suggested action?
  feedback_score INTEGER                -- Optional explicit feedback (-1, 0, 1)
);

-- Discovered behavioral patterns for pattern mining
CREATE TABLE IF NOT EXISTS agent_patterns (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  pattern_type TEXT NOT NULL,           -- e.g., 'temporal', 'sequence', 'correlation'
  pattern_name TEXT,                    -- Human-readable name e.g., 'morning_productivity_peak'
  pattern_json TEXT NOT NULL,           -- Encoded pattern data
  support REAL DEFAULT 0,               -- How often this pattern holds
  confidence REAL DEFAULT 0,            -- Confidence score (0-1)
  last_validated TIMESTAMP,             -- When pattern was last confirmed
  is_active BOOLEAN DEFAULT 1,          -- Whether pattern is still valid
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Evolving user profile dimensions
CREATE TABLE IF NOT EXISTS agent_profile (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  dimension TEXT NOT NULL UNIQUE,       -- e.g., 'peak_hours', 'preferred_session_length', 'habit_stability'
  value_json TEXT NOT NULL,             -- Current value (flexible schema)
  confidence REAL DEFAULT 0.5,          -- How confident we are in this value
  sample_count INTEGER DEFAULT 0,       -- Number of observations
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Bandit arm statistics (Thompson Sampling)
CREATE TABLE IF NOT EXISTS agent_bandit_arms (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  arm_name TEXT NOT NULL UNIQUE,        -- e.g., 'suggest_pomodoro', 'remind_workout', 'celebrate_streak'
  category TEXT,                        -- Grouping for arms
  alpha REAL DEFAULT 1,                 -- Beta distribution parameter (successes + 1)
  beta REAL DEFAULT 1,                  -- Beta distribution parameter (failures + 1)
  total_pulls INTEGER DEFAULT 0,        -- Total times this arm was selected
  total_reward REAL DEFAULT 0,          -- Cumulative reward
  last_pulled TIMESTAMP,
  is_enabled BOOLEAN DEFAULT 1
);

-- Feature snapshots for learning (periodic captures of user state)
CREATE TABLE IF NOT EXISTS agent_feature_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  captured_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  hour_of_day INTEGER,                  -- 0-23
  day_of_week INTEGER,                  -- 0-6 (Sunday = 0)
  mood INTEGER,                         -- Most recent mood (1-10)
  energy INTEGER,                       -- Most recent energy (1-10)
  recent_study_minutes INTEGER,         -- Study minutes in last 24h
  recent_workout_count INTEGER,         -- Workouts in last 24h
  active_assignments INTEGER,           -- Count of incomplete assignments
  overdue_assignments INTEGER,          -- Count of overdue assignments
  current_streak_days INTEGER,          -- Current best streak
  session_in_progress BOOLEAN DEFAULT 0 -- Is user currently in a session?
);

-- Indexes for query performance
CREATE INDEX IF NOT EXISTS idx_agent_insights_type ON agent_insights(insight_type);
CREATE INDEX IF NOT EXISTS idx_agent_insights_shown ON agent_insights(shown_at);
CREATE INDEX IF NOT EXISTS idx_agent_patterns_type ON agent_patterns(pattern_type);
CREATE INDEX IF NOT EXISTS idx_agent_patterns_active ON agent_patterns(is_active);
CREATE INDEX IF NOT EXISTS idx_agent_snapshots_time ON agent_feature_snapshots(captured_at);
CREATE INDEX IF NOT EXISTS idx_agent_snapshots_day ON agent_feature_snapshots(day_of_week, hour_of_day);

-- Seed initial bandit arms with default priors
INSERT OR IGNORE INTO agent_bandit_arms (arm_name, category) VALUES
  ('remind_checkin', 'wellness'),
  ('suggest_pomodoro', 'productivity'),
  ('celebrate_streak', 'motivation'),
  ('warn_overdue', 'academic'),
  ('recommend_break', 'wellness'),
  ('suggest_workout', 'physical'),
  ('practice_reminder', 'skills'),
  ('productivity_tip', 'general'),
  ('weekly_reflection', 'reflection');
