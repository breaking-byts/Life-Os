-- Course Meetings: recurring weekly class times
CREATE TABLE IF NOT EXISTS course_meetings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 0 AND 6), -- 0=Sunday, 6=Saturday
    start_time TEXT NOT NULL, -- HH:MM format (24h)
    end_time TEXT NOT NULL,   -- HH:MM format (24h)
    location TEXT,
    meeting_type TEXT DEFAULT 'lecture', -- lecture, lab, discussion, office_hours
    created_at TEXT DEFAULT (datetime('now')),
    CHECK (start_time < end_time)
);

-- Calendar Events: manual busy blocks and cross-domain events
CREATE TABLE IF NOT EXISTS calendar_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    title TEXT NOT NULL,
    start_at TEXT, -- ISO datetime for one-off events
    end_at TEXT,
    rrule TEXT, -- weekly recurrence: "WEEKLY:0,2,4" = Sun,Tue,Thu; NULL = one-off
    start_time TEXT, -- HH:MM for recurring events
    end_time TEXT,   -- HH:MM for recurring events
    category TEXT NOT NULL DEFAULT 'busy', -- busy, work, personal, health
    domain TEXT, -- academic, physical, skills, wellness (for cross-domain)
    linked_id INTEGER, -- optional link to domain entity
    locked INTEGER DEFAULT 0, -- 1 = cannot be moved by planner
    notes TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    CHECK (
        (rrule IS NOT NULL AND start_time IS NOT NULL AND end_time IS NOT NULL)
        OR (rrule IS NULL AND start_at IS NOT NULL AND end_at IS NOT NULL)
    )
);

-- Weekly Tasks: scoped to specific week, don't roll over
CREATE TABLE IF NOT EXISTS weekly_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    week_start_date TEXT NOT NULL, -- ISO date of Monday (YYYY-MM-DD)
    title TEXT NOT NULL,
    course_id INTEGER REFERENCES courses(id) ON DELETE SET NULL,
    duration_minutes INTEGER DEFAULT 60,
    priority TEXT DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
    notes TEXT,
    completed INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Week Plan Blocks: AI-suggested or user-accepted time blocks
CREATE TABLE IF NOT EXISTS week_plan_blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    week_start_date TEXT NOT NULL, -- ISO date of Monday
    start_at TEXT NOT NULL, -- ISO datetime
    end_at TEXT NOT NULL,   -- ISO datetime
    block_type TEXT NOT NULL, -- study, assignment, exam_prep, break, weekly_task
    course_id INTEGER REFERENCES courses(id) ON DELETE SET NULL,
    weekly_task_id INTEGER REFERENCES weekly_tasks(id) ON DELETE SET NULL,
    title TEXT, -- optional override title
    status TEXT DEFAULT 'suggested' CHECK (status IN ('suggested', 'accepted', 'locked')),
    rationale_json TEXT, -- {"reason": "...", "priority_score": 0.8}
    created_at TEXT DEFAULT (datetime('now')),
    CHECK (start_at < end_at)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_course_meetings_course ON course_meetings(course_id);
CREATE INDEX IF NOT EXISTS idx_course_meetings_day ON course_meetings(day_of_week);
CREATE INDEX IF NOT EXISTS idx_calendar_events_user ON calendar_events(user_id);
CREATE INDEX IF NOT EXISTS idx_calendar_events_category ON calendar_events(category);
CREATE INDEX IF NOT EXISTS idx_weekly_tasks_week ON weekly_tasks(week_start_date);
CREATE INDEX IF NOT EXISTS idx_weekly_tasks_user_week ON weekly_tasks(user_id, week_start_date);
CREATE INDEX IF NOT EXISTS idx_week_plan_blocks_week ON week_plan_blocks(week_start_date);
CREATE INDEX IF NOT EXISTS idx_week_plan_blocks_user_week ON week_plan_blocks(user_id, week_start_date);
