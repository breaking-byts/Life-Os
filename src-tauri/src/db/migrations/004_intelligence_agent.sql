-- Maximum Intelligence Productivity Agent Schema
-- Implements semantic memory, hybrid bandit, multi-scale rewards, and 50+ dimensional features

-- ============================================================================
-- SEMANTIC MEMORY EVENTS
-- Stores every user action with embedding for similarity-based retrieval
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_memory_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    event_type TEXT NOT NULL,           -- 'session', 'checkin', 'workout', 'assignment', 'skill_practice', 'insight_feedback'
    content TEXT NOT NULL,              -- Human-readable description
    embedding BLOB,                     -- 1024-dim float32 vector (stored as blob for efficiency)
    metadata_json TEXT,                 -- Flexible JSON metadata
    context_features BLOB,              -- 50-dim context vector at event time
    outcome_immediate REAL,             -- Immediate satisfaction (0-1)
    outcome_daily REAL,                 -- Daily contribution (filled in later)
    outcome_weekly REAL,                -- Weekly contribution (filled in later)
    outcome_monthly REAL                -- Monthly contribution (filled in later)
);

CREATE INDEX IF NOT EXISTS idx_memory_events_type ON agent_memory_events(event_type);
CREATE INDEX IF NOT EXISTS idx_memory_events_time ON agent_memory_events(timestamp);

-- ============================================================================
-- LINEAR BANDIT ARMS (Phase 1: Day 1-90)
-- Bayesian linear regression for each action with 50+ dimensional features
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_linear_bandit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_name TEXT NOT NULL UNIQUE,
    category TEXT,
    description TEXT,
    -- Bayesian Linear Regression parameters (stored as blobs for efficiency)
    -- theta: mean vector (50-dim)
    theta BLOB,
    -- precision_matrix: inverse covariance (50x50, stored as lower triangular)
    precision_matrix BLOB,
    -- Prior parameters
    prior_mean REAL DEFAULT 0.0,
    prior_precision REAL DEFAULT 1.0,
    noise_precision REAL DEFAULT 1.0,   -- Lambda in literature
    -- Statistics
    total_pulls INTEGER DEFAULT 0,
    total_reward REAL DEFAULT 0.0,
    avg_reward REAL DEFAULT 0.0,
    last_pulled TEXT,
    is_enabled INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Seed initial actions with categories
INSERT OR IGNORE INTO agent_linear_bandit (action_name, category, description) VALUES
    -- Productivity actions
    ('start_pomodoro', 'productivity', 'Start a focused Pomodoro session'),
    ('start_study_session', 'productivity', 'Begin a study session for a course'),
    ('tackle_assignment', 'productivity', 'Work on a specific assignment'),
    ('deep_work_block', 'productivity', 'Extended deep work session (90min+)'),
    
    -- Physical wellness actions
    ('do_workout', 'physical', 'Complete a workout session'),
    ('take_walk', 'physical', 'Go for a walk or light movement'),
    ('stretch_break', 'physical', 'Take a stretching break'),
    
    -- Mental wellness actions
    ('do_checkin', 'wellness', 'Complete a mood/energy check-in'),
    ('take_break', 'wellness', 'Take a recovery break'),
    ('meditation', 'wellness', 'Do a meditation session'),
    
    -- Skill development actions
    ('practice_skill', 'skills', 'Practice a tracked skill'),
    ('learn_new', 'skills', 'Learn something new'),
    
    -- Reflection actions
    ('weekly_review', 'reflection', 'Complete weekly review'),
    ('plan_tomorrow', 'reflection', 'Plan for tomorrow'),
    ('review_goals', 'reflection', 'Review and adjust goals');

-- ============================================================================
-- NEURAL BANDIT MODELS (Phase 2: Month 3+)
-- Stores trained ONNX models for neural upgrade
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_neural_models (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_name TEXT NOT NULL UNIQUE,
    model_version INTEGER DEFAULT 1,
    model_type TEXT NOT NULL,           -- 'reward_predictor', 'uncertainty_estimator'
    onnx_path TEXT,                     -- Path to ONNX model file
    input_dim INTEGER DEFAULT 50,
    output_dim INTEGER DEFAULT 1,
    training_samples INTEGER DEFAULT 0,
    validation_mse REAL,
    created_at TEXT DEFAULT (datetime('now')),
    is_active INTEGER DEFAULT 0
);

-- ============================================================================
-- MULTI-SCALE REWARDS
-- Track rewards at different timescales for balanced optimization
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_reward_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_name TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    context_features BLOB,              -- 50-dim context at decision time
    -- Multi-scale rewards (each 0-1)
    reward_immediate REAL DEFAULT 0,    -- User satisfaction, completion
    reward_daily REAL,                  -- Big 3 progress, balance, recovery (filled at end of day)
    reward_weekly REAL,                 -- Skill gain, goal alignment, consistency (filled weekly)
    reward_monthly REAL,                -- Goal achieved, retention, wellbeing (filled monthly)
    -- Weighted total (computed when all scales available)
    reward_total REAL,
    -- Metadata
    feedback_type TEXT,                 -- 'explicit', 'implicit', 'inferred'
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_reward_log_action ON agent_reward_log(action_name);
CREATE INDEX IF NOT EXISTS idx_reward_log_time ON agent_reward_log(timestamp);

-- ============================================================================
-- EXTENDED CONTEXT FEATURES (50+ dimensions)
-- Richer feature snapshots for ML training
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_rich_context (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    captured_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- Temporal features (6)
    hour_of_day INTEGER,
    day_of_week INTEGER,
    week_of_year INTEGER,
    is_weekend INTEGER,
    time_since_wake REAL,               -- Hours since first activity today
    time_until_sleep REAL,              -- Estimated hours until typical sleep
    
    -- Physiological features (6)
    energy_level INTEGER,
    energy_trajectory REAL,             -- Change from previous check-in
    mood_level INTEGER,
    mood_trajectory REAL,
    fatigue_score REAL,                 -- Inferred from session patterns
    recovery_need REAL,                 -- Based on recent activity intensity
    
    -- Learning/Skill features (6)
    skill_momentum REAL,                -- Recent skill practice consistency
    practice_diversity REAL,            -- How varied practice has been
    learning_rate_estimate REAL,        -- Skill improvement velocity
    focus_trend REAL,                   -- Average session quality trend
    pomodoros_today INTEGER,
    study_minutes_today INTEGER,
    
    -- Goal features (6)
    big_3_completion REAL,              -- % of today's Big 3 completed
    weekly_goal_progress REAL,          -- % toward weekly goals
    assignment_urgency_max REAL,        -- Most urgent assignment score
    overdue_count INTEGER,
    streak_days INTEGER,
    goal_alignment_score REAL,          -- How aligned actions are with goals
    
    -- Circadian features (4)
    circadian_phase REAL,               -- 0-1 within typical wake cycle
    peak_focus_probability REAL,        -- Probability this is a peak hour
    optimal_for_creative REAL,          -- Good time for creative work
    optimal_for_analytical REAL,        -- Good time for analytical work
    
    -- Historical/Memory features (6)
    similar_context_outcomes REAL,      -- Avg outcome from similar past contexts
    same_hour_avg_productivity REAL,
    same_day_avg_energy REAL,
    last_break_hours_ago REAL,
    last_workout_hours_ago REAL,
    last_checkin_hours_ago REAL,
    
    -- Workload features (6)
    active_assignments INTEGER,
    assignments_due_today INTEGER,
    assignments_due_week INTEGER,
    study_hours_this_week REAL,
    target_study_hours_week REAL,
    workload_balance REAL,              -- Actual vs target ratio
    
    -- Interaction features (10)
    energy_x_hour REAL,                 -- Energy * circadian alignment
    mood_x_workload REAL,               -- Mood * workload factor
    streak_x_momentum REAL,             -- Streak * recent consistency
    fatigue_x_time REAL,                -- Fatigue * time of day
    focus_x_complexity REAL,            -- Focus trend * task complexity
    recovery_x_intensity REAL,          -- Recovery need * planned intensity
    energy_trajectory_x_goals REAL,     -- Energy change * goal urgency
    mood_trajectory_x_social REAL,      -- Mood change * social context
    circadian_x_task_type REAL,         -- Circadian fit * task type match
    historical_x_current REAL           -- Past success * current state similarity
);

CREATE INDEX IF NOT EXISTS idx_rich_context_time ON agent_rich_context(captured_at);

-- ============================================================================
-- BIG 3 DAILY GOALS
-- Track the 3 most important tasks for each day
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_big_three (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    priority INTEGER NOT NULL CHECK(priority >= 1 AND priority <= 3),
    title TEXT NOT NULL,
    description TEXT,
    category TEXT,                      -- 'academic', 'physical', 'skills', 'personal'
    linked_assignment_id INTEGER,
    linked_skill_id INTEGER,
    is_completed INTEGER DEFAULT 0,
    completed_at TEXT,
    difficulty_rating INTEGER,          -- 1-5, set after completion
    satisfaction_rating INTEGER,        -- 1-5, set after completion
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(date, priority)
);

CREATE INDEX IF NOT EXISTS idx_big_three_date ON agent_big_three(date);

-- ============================================================================
-- AGENT RECOMMENDATIONS LOG
-- Track what the agent recommended and outcomes
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action_recommended TEXT NOT NULL,
    confidence REAL,
    uncertainty REAL,
    ucb_score REAL,                     -- Upper Confidence Bound score
    context_id INTEGER,                 -- Reference to agent_rich_context
    was_accepted INTEGER,               -- Did user follow recommendation?
    alternative_chosen TEXT,            -- What user did instead (if different)
    feedback_score INTEGER,             -- -1, 0, 1 explicit feedback
    outcome_score REAL,                 -- Computed outcome
    explanation_json TEXT,              -- Why this was recommended
    FOREIGN KEY (context_id) REFERENCES agent_rich_context(id)
);

CREATE INDEX IF NOT EXISTS idx_recommendations_time ON agent_recommendations(timestamp);
CREATE INDEX IF NOT EXISTS idx_recommendations_action ON agent_recommendations(action_recommended);

-- ============================================================================
-- FEATURE IMPORTANCE TRACKING
-- For explainability - which features matter most
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_feature_importance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_name TEXT NOT NULL,
    feature_name TEXT NOT NULL,
    importance_score REAL,              -- Absolute importance
    direction TEXT,                     -- 'positive', 'negative'
    sample_count INTEGER DEFAULT 0,
    last_updated TEXT DEFAULT (datetime('now')),
    UNIQUE(action_name, feature_name)
);

-- ============================================================================
-- AGENT STATE
-- Global agent configuration and state
-- ============================================================================
CREATE TABLE IF NOT EXISTS agent_state (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Initialize agent state
INSERT OR IGNORE INTO agent_state (key, value_json) VALUES
    ('bandit_mode', '"linear"'),        -- 'linear' or 'neural'
    ('exploration_rate', '0.2'),        -- UCB exploration parameter (beta)
    ('min_samples_for_neural', '100'),  -- Samples needed before neural upgrade
    ('reward_weights', '{"immediate": 0.2, "daily": 0.3, "weekly": 0.3, "monthly": 0.2}'),
    ('last_neural_training', 'null'),
    ('embedding_model_path', '"/Users/leelanshkharbanda/Library/Application Support/com.tauri.dev/models/qwen3-embedding"'),
    ('embedding_dim', '1024'),
    ('feature_dim', '50');
