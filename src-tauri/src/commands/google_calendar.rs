use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine as _;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use keyring::Entry;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tauri::State;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use url::Url;

use crate::{DbState, models::google_account::GoogleAccount};

const GOOGLE_AUTH_BASE: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const GOOGLE_CALENDAR_API: &str = "https://www.googleapis.com/calendar/v3";
const GOOGLE_SCOPES: &str = "https://www.googleapis.com/auth/calendar openid email";
const LIFE_OS_PLAN_CALENDAR: &str = "Life OS Plan";
const TOKEN_EXPIRY_BUFFER_SECONDS: i64 = 60;
const WINDOW_PAST_DAYS: i64 = 30;
const WINDOW_FUTURE_DAYS: i64 = 90;

#[derive(Clone)]
pub struct GoogleState {
    oauth: Arc<Mutex<Option<OAuthSession>>>,
    token: Arc<Mutex<Option<TokenState>>>,
}

impl Default for GoogleState {
    fn default() -> Self {
        Self {
            oauth: Arc::new(Mutex::new(None)),
            token: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Debug, Clone)]
struct OAuthSession {
    state: String,
    code_verifier: String,
    redirect_uri: String,
    callback_url: Option<String>,
}

#[derive(Debug, Clone)]
struct TokenState {
    access_token: String,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GoogleAuthBeginResponse {
    pub auth_url: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize)]
pub struct GoogleSyncStatus {
    pub connected: bool,
    pub email: Option<String>,
    pub last_sync: Option<String>,
    pub client_id_set: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
    token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleCalendarList {
    items: Option<Vec<GoogleCalendarListItem>>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct GoogleCalendarListItem {
    id: String,
    summary: Option<String>,
    primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GoogleEventList {
    items: Option<Vec<GoogleEvent>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct GoogleEvent {
    id: String,
    summary: Option<String>,
    status: Option<String>,
    updated: Option<String>,
    etag: Option<String>,
    start: GoogleEventTime,
    end: GoogleEventTime,
    #[serde(rename = "extendedProperties")]
    extended_properties: Option<GoogleExtendedProperties>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct GoogleEventTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
    #[serde(rename = "timeZone")]
    time_zone: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct GoogleExtendedProperties {
    #[serde(rename = "private")]
    private_props: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
struct GoogleEventInsert {
    summary: String,
    start: GoogleEventTimeInsert,
    end: GoogleEventTimeInsert,
    #[serde(rename = "extendedProperties")]
    extended_properties: GoogleExtendedPropertiesInsert,
}

#[derive(Debug, Serialize)]
struct GoogleEventTimeInsert {
    #[serde(rename = "dateTime")]
    date_time: String,
}

#[derive(Debug, Serialize)]
struct GoogleExtendedPropertiesInsert {
    #[serde(rename = "private")]
    private_props: HashMap<String, String>,
}

#[tauri::command]
pub async fn set_google_client_id(
    state: State<'_, DbState>,
    client_id: String,
) -> Result<bool, String> {
    let pool = &state.0;

    sqlx::query(
        r#"
        INSERT INTO user_settings (id, user_id, google_client_id, updated_at)
        VALUES (1, 1, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            google_client_id = excluded.google_client_id,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(client_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub async fn google_oauth_begin(
    state: State<'_, DbState>,
    google_state: State<'_, GoogleState>,
) -> Result<GoogleAuthBeginResponse, String> {
    let client_id = get_google_client_id(&state.0).await?
        .ok_or_else(|| "Google client ID not set".to_string())?;

    let code_verifier = generate_code_verifier();
    let code_challenge = code_challenge_from_verifier(&code_verifier);
    let state_token = random_token(32);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;
    let local_addr = listener.local_addr().map_err(|e| e.to_string())?;
    let redirect_uri = format!("http://127.0.0.1:{}/callback", local_addr.port());

    let auth_url = format!(
        "{base}?client_id={client_id}&response_type=code&redirect_uri={redirect}&scope={scope}&state={state}&code_challenge={challenge}&code_challenge_method=S256&access_type=offline&prompt=consent",
        base = GOOGLE_AUTH_BASE,
        client_id = urlencoding::encode(&client_id),
        redirect = urlencoding::encode(&redirect_uri),
        scope = urlencoding::encode(GOOGLE_SCOPES),
        state = urlencoding::encode(&state_token),
        challenge = urlencoding::encode(&code_challenge),
    );

    let session = OAuthSession {
        state: state_token.clone(),
        code_verifier,
        redirect_uri: redirect_uri.clone(),
        callback_url: None,
    };

    {
        let mut lock = google_state.oauth.lock().await;
        *lock = Some(session);
    }

    let oauth_handle = google_state.oauth.clone();
    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 2048];
            if let Ok(n) = socket.read(&mut buf).await {
                let req = String::from_utf8_lossy(&buf[..n]);
                if let Some(path_line) = req.lines().next() {
                    let parts: Vec<&str> = path_line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let path = parts[1];
                        if path.contains("code=") {
                            let callback_url = format!("http://{}{}", local_addr, path);
                            let mut lock = oauth_handle.lock().await;
                            if let Some(ref mut session) = *lock {
                                session.callback_url = Some(callback_url);
                            }
                        }
                    }
                }
            }
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body><h3>Google Calendar connected.</h3><p>You can close this window.</p></body></html>";
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    Ok(GoogleAuthBeginResponse { auth_url, redirect_uri })
}

#[tauri::command]
pub async fn google_oauth_complete(
    state: State<'_, DbState>,
    google_state: State<'_, GoogleState>,
    callback_url: Option<String>,
) -> Result<GoogleAccount, String> {
    let client_id = get_google_client_id(&state.0).await?
        .ok_or_else(|| "Google client ID not set".to_string())?;

    let mut session_opt = google_state.oauth.lock().await;
    let session = session_opt.clone().ok_or_else(|| "OAuth session not initialized".to_string())?;

    let effective_callback = if let Some(url) = callback_url {
        url
    } else if let Some(url) = session.callback_url.clone() {
        url
    } else {
        return Err("OAuth callback not received yet".to_string());
    };

    let parsed = Url::parse(&effective_callback).map_err(|e| e.to_string())?;
    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| "Missing code in callback URL".to_string())?;
    let returned_state = parsed
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| "Missing state in callback URL".to_string())?;

    if returned_state != session.state {
        return Err("OAuth state mismatch".to_string());
    }

    let client = Client::new();
    let token_res = client
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("code_verifier", session.code_verifier.as_str()),
            ("redirect_uri", session.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GoogleTokenResponse>()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(refresh) = token_res.refresh_token.clone() {
        store_refresh_token(&refresh)?;
    }

    let expires_at = Utc::now() + Duration::seconds(token_res.expires_in);

    {
        let mut token_lock = google_state.token.lock().await;
        *token_lock = Some(TokenState {
            access_token: token_res.access_token.clone(),
            expires_at,
        });
    }

    let user_info = client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(&token_res.access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GoogleUserInfo>()
        .await
        .map_err(|e| e.to_string())?;

    let account = upsert_google_account(&state.0, &user_info).await?;

    // Initialize calendar prefs row if missing
    sqlx::query(
        r#"INSERT INTO google_calendar_prefs (user_id, import_all, export_calendar_id, updated_at)
           VALUES (1, 1, NULL, datetime('now'))
           ON CONFLICT(user_id) DO NOTHING"#,
    )
    .execute(&state.0)
    .await
    .ok();

    // Clear session after completion
    *session_opt = None;

    Ok(account)
}

#[tauri::command]
pub async fn google_sync_now(
    state: State<'_, DbState>,
    google_state: State<'_, GoogleState>,
) -> Result<bool, String> {
    let client_id = get_google_client_id(&state.0).await?
        .ok_or_else(|| "Google client ID not set".to_string())?;

    let access_token = ensure_access_token(&google_state, &client_id).await?;
    let client = Client::new();

    let calendar_list = client
        .get(format!("{}/users/me/calendarList", GOOGLE_CALENDAR_API))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GoogleCalendarList>()
        .await
        .map_err(|e| e.to_string())?;

    let calendars = calendar_list.items.unwrap_or_default();
    if calendars.is_empty() {
        return Ok(true);
    }

    let export_calendar_id = ensure_life_os_plan_calendar(&state.0, &client, &access_token, &calendars).await?;

    let (time_min, time_max, date_min, date_max) = sync_window_range();

    for calendar in &calendars {
        let events = fetch_events(&client, &access_token, &calendar.id, &time_min, &time_max).await?;
        if calendar.id == export_calendar_id {
            sync_plan_calendar(&state.0, &client, &access_token, &calendar.id, &date_min, &date_max, events).await?;
        } else {
            sync_external_calendar(&state.0, &calendar.id, events).await?;
        }
    }

    sqlx::query("UPDATE google_calendar_prefs SET updated_at = datetime('now') WHERE user_id = 1")
        .execute(&state.0)
        .await
        .ok();

    Ok(true)
}

#[tauri::command]
pub async fn get_google_sync_status(state: State<'_, DbState>) -> Result<GoogleSyncStatus, String> {
    let pool = &state.0;

    let client_id = get_google_client_id(pool).await?;
    let account = sqlx::query_as::<_, GoogleAccount>(
        "SELECT * FROM google_accounts WHERE user_id = 1 ORDER BY connected_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let last_sync = sqlx::query_scalar::<_, Option<String>>(
        "SELECT updated_at FROM google_calendar_prefs WHERE user_id = 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .flatten();

    Ok(GoogleSyncStatus {
        connected: account.is_some(),
        email: account.as_ref().and_then(|a| a.email.clone()),
        last_sync,
        client_id_set: client_id.is_some(),
    })
}

#[tauri::command]
pub async fn disconnect_google(state: State<'_, DbState>) -> Result<bool, String> {
    let pool = &state.0;

    sqlx::query("DELETE FROM google_accounts WHERE user_id = 1")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM google_calendar_prefs WHERE user_id = 1")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM google_event_links")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM google_sync_state")
        .execute(pool)
        .await
        .ok();

    clear_refresh_token()?;

    Ok(true)
}

async fn get_google_client_id(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Option<String>, String> {
    let row = sqlx::query_scalar::<_, Option<String>>(
        "SELECT google_client_id FROM user_settings WHERE user_id = 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.flatten())
}

fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    let verifier: String = (0..64).map(|_| rng.sample(Alphanumeric) as char).collect();
    verifier
}

fn code_challenge_from_verifier(verifier: &str) -> String {
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

fn random_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn keyring_entry() -> Result<Entry, String> {
    Entry::new("life-os", "google_refresh_token").map_err(|e| e.to_string())
}

fn store_refresh_token(token: &str) -> Result<(), String> {
    keyring_entry()?
        .set_password(token)
        .map_err(|e| e.to_string())
}

fn load_refresh_token() -> Result<Option<String>, String> {
    match keyring_entry()?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn clear_refresh_token() -> Result<(), String> {
    match keyring_entry()?.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

async fn ensure_access_token(
    google_state: &GoogleState,
    client_id: &str,
) -> Result<String, String> {
    let now = Utc::now();
    if let Some(token) = google_state.token.lock().await.clone() {
        if token.expires_at > now + Duration::seconds(TOKEN_EXPIRY_BUFFER_SECONDS) {
            return Ok(token.access_token);
        }
    }

    let refresh_token = load_refresh_token()?.ok_or_else(|| "Missing refresh token".to_string())?;
    let client = Client::new();
    let token_res = client
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GoogleTokenResponse>()
        .await
        .map_err(|e| e.to_string())?;

    let expires_at = Utc::now() + Duration::seconds(token_res.expires_in);

    let mut token_lock = google_state.token.lock().await;
    *token_lock = Some(TokenState {
        access_token: token_res.access_token.clone(),
        expires_at,
    });

    Ok(token_res.access_token)
}

async fn upsert_google_account(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    user_info: &GoogleUserInfo,
) -> Result<GoogleAccount, String> {
    let rec = sqlx::query_as::<_, GoogleAccount>(
        r#"INSERT INTO google_accounts (user_id, google_user_id, email, connected_at)
           VALUES (1, ?, ?, datetime('now'))
           ON CONFLICT(user_id, google_user_id) DO UPDATE SET
             email = excluded.email,
             connected_at = datetime('now')
           RETURNING *"#,
    )
    .bind(&user_info.sub)
    .bind(&user_info.email)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rec)
}

fn sync_window_range() -> (String, String, String, String) {
    let start = Utc::now() - Duration::days(WINDOW_PAST_DAYS);
    let end = Utc::now() + Duration::days(WINDOW_FUTURE_DAYS);
    let start_date = start.date_naive().format("%Y-%m-%d").to_string();
    let end_date = end.date_naive().format("%Y-%m-%d").to_string();
    (start.to_rfc3339(), end.to_rfc3339(), start_date, end_date)
}

async fn fetch_events(
    client: &Client,
    access_token: &str,
    calendar_id: &str,
    time_min: &str,
    time_max: &str,
) -> Result<Vec<GoogleEvent>, String> {
    let url = format!(
        "{}/calendars/{}/events",
        GOOGLE_CALENDAR_API,
        urlencoding::encode(calendar_id)
    );
    let mut events: Vec<GoogleEvent> = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let mut req = client
            .get(&url)
            .bearer_auth(access_token)
            .query(&[
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
                ("timeMin", time_min),
                ("timeMax", time_max),
            ]);

        if let Some(ref token) = page_token {
            req = req.query(&[("pageToken", token.as_str())]);
        }

        let res = req
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<GoogleEventList>()
            .await
            .map_err(|e| e.to_string())?;

        events.extend(res.items.unwrap_or_default());
        if let Some(next) = res.next_page_token {
            page_token = Some(next);
        } else {
            break;
        }
    }

    Ok(events)
}

async fn ensure_life_os_plan_calendar(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &Client,
    access_token: &str,
    calendars: &[GoogleCalendarListItem],
) -> Result<String, String> {
    let existing_pref = sqlx::query_scalar::<_, Option<String>>(
        "SELECT export_calendar_id FROM google_calendar_prefs WHERE user_id = 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .flatten();

    if let Some(id) = existing_pref {
        return Ok(id);
    }

    if let Some(found) = calendars.iter().find(|c| c.summary.as_deref() == Some(LIFE_OS_PLAN_CALENDAR)) {
        sqlx::query(
            "INSERT INTO google_calendar_prefs (user_id, import_all, export_calendar_id, updated_at)
             VALUES (1, 1, ?, datetime('now'))
             ON CONFLICT(user_id) DO UPDATE SET
                export_calendar_id = excluded.export_calendar_id,
                updated_at = datetime('now')",
        )
        .bind(&found.id)
        .execute(pool)
        .await
        .ok();
        return Ok(found.id.clone());
    }

    let create_url = format!("{}/calendars", GOOGLE_CALENDAR_API);
    let created = client
        .post(create_url)
        .bearer_auth(access_token)
        .json(&serde_json::json!({
            "summary": LIFE_OS_PLAN_CALENDAR,
            "timeZone": "UTC",
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GoogleCalendarListItem>()
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO google_calendar_prefs (user_id, import_all, export_calendar_id, updated_at)
         VALUES (1, 1, ?, datetime('now'))
         ON CONFLICT(user_id) DO UPDATE SET
            export_calendar_id = excluded.export_calendar_id,
            updated_at = datetime('now')",
    )
    .bind(&created.id)
    .execute(pool)
    .await
    .ok();

    Ok(created.id)
}

async fn sync_external_calendar(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    calendar_id: &str,
    events: Vec<GoogleEvent>,
) -> Result<(), String> {
    for event in events {
        if event.status.as_deref() == Some("cancelled") {
            delete_linked_event(pool, calendar_id, &event.id, "calendar_event").await?;
            continue;
        }

        let (start_at, end_at) = match event_times(&event) {
            Some(times) => times,
            None => continue,
        };

        let title = event.summary.clone().unwrap_or_else(|| "(No title)".to_string());
        let link = sqlx::query_as::<_, crate::models::google_event_link::GoogleEventLink>(
            "SELECT * FROM google_event_links WHERE google_calendar_id = ? AND google_event_id = ? AND local_type = 'calendar_event'",
        )
        .bind(calendar_id)
        .bind(&event.id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(existing) = link {
            sqlx::query(
                "UPDATE calendar_events SET title = ?, start_at = ?, end_at = ?, locked = 1, category = 'busy', domain = 'google' WHERE id = ?",
            )
            .bind(&title)
            .bind(&start_at)
            .bind(&end_at)
            .bind(existing.local_id)
            .execute(pool)
            .await
            .ok();

            update_link(pool, existing.id, event.etag.as_deref()).await?;
        } else {
            let rec_id: i64 = sqlx::query_scalar(
                r#"INSERT INTO calendar_events (user_id, title, start_at, end_at, category, domain, locked)
                   VALUES (1, ?, ?, ?, 'busy', 'google', 1)
                   RETURNING id"#,
            )
            .bind(&title)
            .bind(&start_at)
            .bind(&end_at)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query(
                r#"INSERT INTO google_event_links (local_type, local_id, google_calendar_id, google_event_id, etag, last_synced_at)
                   VALUES ('calendar_event', ?, ?, ?, ?, datetime('now'))"#,
            )
            .bind(rec_id)
            .bind(calendar_id)
            .bind(&event.id)
            .bind(&event.etag)
            .execute(pool)
            .await
            .ok();
        }
    }

    Ok(())
}

async fn sync_plan_calendar(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &Client,
    access_token: &str,
    calendar_id: &str,
    date_min: &str,
    date_max: &str,
    events: Vec<GoogleEvent>,
) -> Result<(), String> {
    let mut google_events_by_id: HashMap<String, GoogleEvent> = HashMap::new();
    for event in events.into_iter() {
        if event.status.as_deref() == Some("cancelled") {
            continue;
        }
        google_events_by_id.insert(event.id.clone(), event);
    }

    // Update local plan blocks from Google events
    for event in google_events_by_id.values() {
        if let Some(lifeos_id) = extract_lifeos_id(event) {
            if let Some(local_id) = parse_lifeos_id(&lifeos_id) {
                let (start_at, end_at) = match event_times(event) {
                    Some(times) => times,
                    None => continue,
                };
                let title = event.summary.clone().unwrap_or_else(|| "Planned block".to_string());
                let _ = apply_google_update_to_plan_block(
                    pool,
                    local_id,
                    &title,
                    &start_at,
                    &end_at,
                )
                .await?;
                upsert_plan_link(pool, local_id, calendar_id, &event.id, event.etag.as_deref()).await?;
            }
        }
    }

    // Push accepted/locked blocks to Google
    let blocks = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<String>)>(
        "SELECT id, start_at, end_at, title, status, block_type FROM week_plan_blocks WHERE status IN ('accepted', 'locked') AND date(start_at) >= ? AND date(start_at) <= ?",
    )
    .bind(date_min)
    .bind(date_max)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for (id, start_at, end_at, title, status, block_type) in blocks {
        let event_title = title.unwrap_or_else(|| block_type.unwrap_or_else(|| "Planned block".to_string()));
        let link = sqlx::query_as::<_, crate::models::google_event_link::GoogleEventLink>(
            "SELECT * FROM google_event_links WHERE local_type = 'week_plan_block' AND local_id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(existing) = link {
            if let Some(google_event) = google_events_by_id.get(&existing.google_event_id) {
                let (g_start, g_end) = match event_times(google_event) {
                    Some(times) => times,
                    None => continue,
                };

                if g_start != start_at || g_end != end_at || google_event.summary.clone().unwrap_or_default() != event_title {
                    // Update Google with local changes
                    patch_google_event(
                        client,
                        access_token,
                        calendar_id,
                        &existing.google_event_id,
                        &event_title,
                        &start_at,
                        &end_at,
                        id,
                    )
                    .await?;
                    update_link(pool, existing.id, google_event.etag.as_deref()).await?;
                }
            } else {
                // Event missing from Google within window, recreate
                let new_id = insert_google_event(
                    client,
                    access_token,
                    calendar_id,
                    &event_title,
                    &start_at,
                    &end_at,
                    id,
                )
                .await?;
                sqlx::query(
                    r#"UPDATE google_event_links SET google_event_id = ?, google_calendar_id = ?, last_synced_at = datetime('now') WHERE id = ?"#,
                )
                .bind(new_id)
                .bind(calendar_id)
                .bind(existing.id)
                .execute(pool)
                .await
                .ok();
            }
        } else {
            let google_event_id = insert_google_event(
                client,
                access_token,
                calendar_id,
                &event_title,
                &start_at,
                &end_at,
                id,
            )
            .await?;

            sqlx::query(
                r#"INSERT INTO google_event_links (local_type, local_id, google_calendar_id, google_event_id, last_synced_at)
                   VALUES ('week_plan_block', ?, ?, ?, datetime('now'))"#,
            )
            .bind(id)
            .bind(calendar_id)
            .bind(&google_event_id)
            .execute(pool)
            .await
            .ok();
        }

        if status.as_deref() == Some("locked") {
            // No-op for now; future: set transparency/busy
        }
    }

    Ok(())
}

fn extract_lifeos_id(event: &GoogleEvent) -> Option<String> {
    event
        .extended_properties
        .as_ref()?
        .private_props
        .as_ref()?
        .get("lifeos_id")
        .cloned()
}

fn parse_lifeos_id(value: &str) -> Option<i64> {
    if let Some(stripped) = value.strip_prefix("wpb_") {
        stripped.parse().ok()
    } else {
        value.parse().ok()
    }
}

async fn apply_google_update_to_plan_block(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    local_id: i64,
    title: &str,
    start_at: &str,
    end_at: &str,
) -> Result<bool, String> {
    let existing = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT start_at, end_at, title FROM week_plan_blocks WHERE id = ?",
    )
    .bind(local_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some((curr_start, curr_end, curr_title)) = existing {
        if curr_start == start_at && curr_end == end_at && curr_title.as_deref() == Some(title) {
            return Ok(false);
        }

        let week_start = week_start_date_from(start_at);
        sqlx::query(
            "UPDATE week_plan_blocks SET week_start_date = ?, start_at = ?, end_at = ?, title = COALESCE(?, title) WHERE id = ?",
        )
        .bind(week_start)
        .bind(start_at)
        .bind(end_at)
        .bind(title)
        .bind(local_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(true)
    } else {
        // Create new plan block if missing
        let week_start = week_start_date_from(start_at);
        sqlx::query(
            "INSERT INTO week_plan_blocks (id, user_id, week_start_date, start_at, end_at, block_type, title, status)
             VALUES (?, 1, ?, ?, ?, 'study', ?, 'accepted')",
        )
        .bind(local_id)
        .bind(week_start)
        .bind(start_at)
        .bind(end_at)
        .bind(title)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(true)
    }
}

fn week_start_date_from(start_at: &str) -> String {
    let date = parse_date(start_at).unwrap_or_else(|| Utc::now().date_naive());
    let weekday = date.weekday().num_days_from_monday() as i64;
    let monday = date - Duration::days(weekday);
    monday.format("%Y-%m-%d").to_string()
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.date_naive());
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.date());
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        return Some(dt.date());
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Some(date);
    }
    None
}

fn event_times(event: &GoogleEvent) -> Option<(String, String)> {
    if let Some(start_dt) = &event.start.date_time {
        let end_dt = event.end.date_time.clone().unwrap_or_else(|| start_dt.clone());
        return Some((start_dt.clone(), end_dt));
    }

    if let Some(date) = &event.start.date {
        let end_date = event.end.date.clone().unwrap_or_else(|| date.clone());
        return Some((date.clone(), end_date));
    }

    None
}

async fn delete_linked_event(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    calendar_id: &str,
    google_event_id: &str,
    local_type: &str,
) -> Result<(), String> {
    let link = sqlx::query_as::<_, crate::models::google_event_link::GoogleEventLink>(
        "SELECT * FROM google_event_links WHERE google_calendar_id = ? AND google_event_id = ? AND local_type = ?",
    )
    .bind(calendar_id)
    .bind(google_event_id)
    .bind(local_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(existing) = link {
        if local_type == "calendar_event" {
            sqlx::query("DELETE FROM calendar_events WHERE id = ?")
                .bind(existing.local_id)
                .execute(pool)
                .await
                .ok();
        }
        sqlx::query("DELETE FROM google_event_links WHERE id = ?")
            .bind(existing.id)
            .execute(pool)
            .await
            .ok();
    }

    Ok(())
}

async fn update_link(pool: &sqlx::Pool<sqlx::Sqlite>, link_id: i64, etag: Option<&str>) -> Result<(), String> {
    sqlx::query("UPDATE google_event_links SET etag = ?, last_synced_at = datetime('now') WHERE id = ?")
        .bind(etag)
        .bind(link_id)
        .execute(pool)
        .await
        .ok();
    Ok(())
}

async fn upsert_plan_link(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    local_id: i64,
    calendar_id: &str,
    google_event_id: &str,
    etag: Option<&str>,
) -> Result<(), String> {
    sqlx::query(
        r#"INSERT INTO google_event_links (local_type, local_id, google_calendar_id, google_event_id, etag, last_synced_at)
           VALUES ('week_plan_block', ?, ?, ?, ?, datetime('now'))
           ON CONFLICT(local_type, local_id) DO UPDATE SET
             google_calendar_id = excluded.google_calendar_id,
             google_event_id = excluded.google_event_id,
             etag = excluded.etag,
             last_synced_at = datetime('now')"#,
    )
    .bind(local_id)
    .bind(calendar_id)
    .bind(google_event_id)
    .bind(etag)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

async fn insert_google_event(
    client: &Client,
    access_token: &str,
    calendar_id: &str,
    title: &str,
    start_at: &str,
    end_at: &str,
    local_id: i64,
) -> Result<String, String> {
    let mut private_props = HashMap::new();
    private_props.insert("lifeos_id".to_string(), format!("wpb_{}", local_id));
    private_props.insert("lifeos_type".to_string(), "week_plan_block".to_string());

    let payload = GoogleEventInsert {
        summary: title.to_string(),
        start: GoogleEventTimeInsert {
            date_time: normalize_datetime(start_at),
        },
        end: GoogleEventTimeInsert {
            date_time: normalize_datetime(end_at),
        },
        extended_properties: GoogleExtendedPropertiesInsert { private_props },
    };

    let url = format!(
        "{}/calendars/{}/events",
        GOOGLE_CALENDAR_API,
        urlencoding::encode(calendar_id)
    );

    let res = client
        .post(url)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GoogleEvent>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(res.id)
}

async fn patch_google_event(
    client: &Client,
    access_token: &str,
    calendar_id: &str,
    event_id: &str,
    title: &str,
    start_at: &str,
    end_at: &str,
    local_id: i64,
) -> Result<(), String> {
    let mut private_props = HashMap::new();
    private_props.insert("lifeos_id".to_string(), format!("wpb_{}", local_id));
    private_props.insert("lifeos_type".to_string(), "week_plan_block".to_string());

    let payload = GoogleEventInsert {
        summary: title.to_string(),
        start: GoogleEventTimeInsert {
            date_time: normalize_datetime(start_at),
        },
        end: GoogleEventTimeInsert {
            date_time: normalize_datetime(end_at),
        },
        extended_properties: GoogleExtendedPropertiesInsert { private_props },
    };

    let url = format!(
        "{}/calendars/{}/events/{}",
        GOOGLE_CALENDAR_API,
        urlencoding::encode(calendar_id),
        urlencoding::encode(event_id)
    );

    client
        .patch(url)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn normalize_datetime(value: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return dt.to_rfc3339();
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        if let Some(local) = Local.from_local_datetime(&dt).single() {
            return local.to_rfc3339();
        }
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        if let Some(local) = Local.from_local_datetime(&dt).single() {
            return local.to_rfc3339();
        }
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let dt = date.and_hms_opt(9, 0, 0).unwrap_or_else(|| date.and_hms_opt(0, 0, 0).unwrap());
        if let Some(local) = Local.from_local_datetime(&dt).single() {
            return local.to_rfc3339();
        }
    }
    Utc::now().to_rfc3339()
}
