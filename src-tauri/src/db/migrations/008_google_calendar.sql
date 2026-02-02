-- Google Calendar sync tables

CREATE TABLE IF NOT EXISTS google_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    google_user_id TEXT NOT NULL,
    email TEXT,
    connected_at TEXT DEFAULT (datetime('now')),
    UNIQUE(user_id, google_user_id)
);

CREATE TABLE IF NOT EXISTS google_calendar_prefs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 1,
    import_all INTEGER DEFAULT 1,
    export_calendar_id TEXT,
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(user_id)
);

CREATE TABLE IF NOT EXISTS google_event_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_type TEXT NOT NULL,
    local_id INTEGER NOT NULL,
    google_calendar_id TEXT NOT NULL,
    google_event_id TEXT NOT NULL,
    etag TEXT,
    last_synced_at TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(local_type, local_id),
    UNIQUE(google_calendar_id, google_event_id)
);

CREATE TABLE IF NOT EXISTS google_sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    google_calendar_id TEXT NOT NULL UNIQUE,
    sync_token TEXT,
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_google_event_links_local ON google_event_links(local_type, local_id);
CREATE INDEX IF NOT EXISTS idx_google_event_links_google ON google_event_links(google_calendar_id, google_event_id);

-- Store user-provided Google OAuth client id
ALTER TABLE user_settings ADD COLUMN google_client_id TEXT;
