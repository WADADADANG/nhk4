use crate::model::{GlobalConfig, Profile};
use crate::server::static_files::serve_file_from_disk;
use crate::state::AppState;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::StatusCode;
use axum::response::{sse::Event, IntoResponse, Json, Response, Sse};
use futures_util::stream::Stream;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Deserialize)]
pub struct NameQuery {
    pub name: Option<String>,
}

pub async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.get_status_json().await)
}

pub async fn get_hwid_handler() -> impl IntoResponse {
    let details = crate::security::get_hwid_details();
    Json(json!({
        "success": true,
        "hwid": details.hwid,
        "machineGuid": details.machine_guid,
        "processor": details.processor,
        "osInfo": details.os_info
    }))
}

#[derive(Deserialize)]
pub struct ActivateLicensePayload {
    pub key: Option<String>,
    #[serde(rename = "licenseKey")]
    pub license_key: Option<String>,
}

pub async fn get_license_status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let status = crate::security::check_license_status(Some(&state.config_store.base_dir));
    Json(json!({
        "success": true,
        "isValid": status.is_valid,
        "status": status.status,
        "tier": status.tier,
        "expiresAt": status.expires_at,
        "daysRemaining": status.days_remaining,
        "hwid": status.hwid,
        "activeKey": status.active_key,
        "active_key": status.active_key
    }))
}

pub async fn activate_license_handler(
    State(state): State<AppState>,
    Json(payload): Json<ActivateLicensePayload>,
) -> impl IntoResponse {
    let key_str = payload.key.or(payload.license_key).unwrap_or_default().trim().to_string();
    if key_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "กรุณาวาง License Key ก่อนกด Activate" })),
        ).into_response();
    }

    let current_hwid = crate::security::generate_hwid();

    // 1. ถ้าเป็นคีย์ Signed Token แบบออฟไลน์ (Base64) ที่สร้างไว้ล่วงหน้า
    if let Ok(claims) = crate::security::verify_license_key(
        &key_str,
        &current_hwid,
        crate::security::DEFAULT_LICENSE_SALT,
    ) {
        if let Err(e) = crate::security::save_license_file(&key_str, Some(&state.config_store.base_dir)) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": e.to_string() })),
            ).into_response();
        }
        let _ = crate::security::save_serial_file(&key_str, Some(&state.config_store.base_dir));
        state.log("system", &format!("🔑 License activated offline. Tier: {}", claims.tier));
        let status = crate::security::check_license_status(Some(&state.config_store.base_dir));
        state.notify_status_change().await;
        return Json(json!({
            "success": true,
            "message": "License activated successfully",
            "tier": claims.tier,
            "expiresAt": claims.expires_at,
            "licenseStatus": status,
            "activeKey": status.active_key
        })).into_response();
    }

    // 2. ถ้าเป็น Serial Key (เช่น NHK-PRO-XXXX) ให้ยิงไป Redeem และล็อก HWID กับ Cloudflare Worker
    let default_worker_url = crate::secure_str!("https://nodehotkey-api.kitsada19972540.workers.dev");
    match crate::security::license::redeem_license_from_server(
        &default_worker_url,
        &key_str,
        &current_hwid,
        Some(&state.config_store.base_dir),
    ).await {
        Ok(claims) => {
            state.log("system", &format!("✨ Serial Key redeemed and bound to HWID on Cloudflare. Tier: {}", claims.tier));
            let status = crate::security::check_license_status(Some(&state.config_store.base_dir));
            state.notify_status_change().await;
            Json(json!({
                "success": true,
                "message": "Product key activated and locked to this machine successfully!",
                "tier": claims.tier,
                "expiresAt": claims.expires_at,
                "licenseStatus": status
            })).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": e.to_string()
            })),
        ).into_response(),
    }
}

// 🛡️ Anti-Spam Rate Limiter for License Online Verification (Hybrid: 5s cooldown + 10 req/min => 10 min lockout)
struct LicenseSyncRateLimiter {
    last_requests: Vec<Instant>,
    cooldown_until: Option<Instant>,
    lockout_until: Option<Instant>,
}

impl LicenseSyncRateLimiter {
    pub const fn new() -> Self {
        Self {
            last_requests: Vec::new(),
            cooldown_until: None,
            lockout_until: None,
        }
    }

    pub fn check(&mut self) -> Result<(), (u64, &'static str)> {
        let now = Instant::now();

        // 1. Check Hard Lockout (10 minutes = 600s)
        if let Some(lockout) = self.lockout_until {
            if now < lockout {
                let remaining_secs = (lockout - now).as_secs() + 1;
                return Err((remaining_secs, "ระบบระงับการตรวจสอบชั่วคราวเนื่องจากมีการเรียกใช้งานถี่เกินไป (10 ครั้ง/นาที) กรุณารอสักครู่"));
            } else {
                self.lockout_until = None;
            }
        }

        // 2. Check 5s Cooldown
        if let Some(cooldown) = self.cooldown_until {
            if now < cooldown {
                let remaining_secs = (cooldown - now).as_secs() + 1;
                return Err((remaining_secs, "กรุณารอสักครู่ (5 วินาที) ก่อนกดตรวจสอบสถานะใหม่อีกครั้ง"));
            } else {
                self.cooldown_until = None;
            }
        }

        // 3. Sliding Window: 10 requests within 60 seconds
        let window = Duration::from_secs(60);
        self.last_requests.retain(|&t| now.duration_since(t) < window);
        self.last_requests.push(now);

        if self.last_requests.len() >= 10 {
            self.lockout_until = Some(now + Duration::from_secs(600));
            self.last_requests.clear();
            return Err((600, "ตรวจพบการส่งคำขอถี่เกินไป (10 ครั้งใน 1 นาที) ระบบจึงระงับการตรวจสอบชั่วคราวเป็นเวลา 10 นาที"));
        }

        // Apply 5s cooldown
        self.cooldown_until = Some(now + Duration::from_secs(5));
        Ok(())
    }
}

static SYNC_RATE_LIMITER: Mutex<LicenseSyncRateLimiter> = Mutex::new(LicenseSyncRateLimiter::new());

#[derive(Deserialize, Default)]
pub struct SyncOnlineQuery {
    pub silent: Option<String>,
}

pub async fn sync_license_online_handler(
    State(state): State<AppState>,
    Query(query): Query<SyncOnlineQuery>,
) -> impl IntoResponse {
    let is_silent = query
        .silent
        .as_deref()
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !is_silent {
        let mut limiter = SYNC_RATE_LIMITER.lock().unwrap();
        if let Err((retry_after, msg)) = limiter.check() {
            let status = crate::security::check_license_status(Some(&state.config_store.base_dir));
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "success": false,
                    "error": msg,
                    "retryAfterSecs": retry_after,
                    "licenseStatus": status,
                    "status": status.status,
                    "isValid": status.is_valid
                })),
            ).into_response();
        }
    }

    let current_hwid = crate::security::generate_hwid();
    let default_worker_url = crate::secure_str!("https://nodehotkey-api.kitsada19972540.workers.dev");

    match crate::security::license::sync_license_from_server(
        &default_worker_url,
        &current_hwid,
        Some(&state.config_store.base_dir),
    )
    .await
    {
        Ok(claims) => {
            state.log("system", &format!("☁️ License synced from Cloudflare Server. Tier: {}", claims.tier));
            let status = crate::security::check_license_status(Some(&state.config_store.base_dir));
            state.notify_status_change().await;
            Json(json!({
                "success": true,
                "message": "License synced successfully from Cloudflare server",
                "tier": claims.tier,
                "expiresAt": claims.expires_at,
                "licenseStatus": status,
                "status": status.status,
                "isValid": status.is_valid
            })).into_response()
        }
        Err(e) => {
            let status = crate::security::check_license_status(Some(&state.config_store.base_dir));
            if !status.is_valid {
                state.is_running.store(false, Ordering::SeqCst);
                state.log("error", &format!("🔒 Engine halted: License is {}", status.status));
            }
            state.notify_status_change().await;
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": e.to_string(),
                    "licenseStatus": status,
                    "status": status.status,
                    "isValid": status.is_valid
                })),
            ).into_response()
        }
    }
}

pub async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let global = state.config_store.get_global_config().await;
    let profiles_list = state.config_store.get_all_profiles().await;

    let mut profiles_map = HashMap::new();
    for p in profiles_list {
        profiles_map.insert(p.name.clone(), p);
    }

    let clients = state.client_manager.get_all_clients().await;
    let cdp_sessions = state.cdp_sessions.read().await;
    let mut active_clients: Vec<String> = clients
        .iter()
        .filter(|c| c.is_running || cdp_sessions.contains_key(&c.id))
        .map(|c| c.id.to_string())
        .collect();
    for &cid in cdp_sessions.keys() {
        let s = cid.to_string();
        if !active_clients.contains(&s) {
            active_clients.push(s);
        }
    }

    let mut client_statuses = serde_json::Map::new();
    for slot_id in &global.global_settings.client_slots {
        let client_str = slot_id.to_string();
        let is_client_active = active_clients.contains(&client_str);
        client_statuses.insert(
            client_str,
            json!({
                "status": if is_client_active { "Standby" } else { "Offline" },
                "type": if is_client_active { "standby" } else { "offline" }
            }),
        );
    }

    Json(json!({
        "activeProfile": global.active_profile,
        "activeProfiles": global.active_profiles,
        "disabledClients": global.disabled_clients,
        "globalSettings": global.global_settings,
        "profiles": profiles_map,
        "creatorIdentity": global.creator_identity,
        "port": global.global_settings.web_port,
        "serverPort": global.global_settings.web_port,
        "activeClients": active_clients,
        "clientStatuses": client_statuses,
        "isSuspended": state.hotkey_listener.is_suspended()
    }))
}

pub async fn save_config(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let existing_global = state.config_store.get_global_config().await;
    if let Ok(mut cfg) = serde_json::from_value::<GlobalConfig>(payload.clone()) {
        if payload.get("globalSettings").and_then(|g| g.get("enableOverlay")).is_none() {
            cfg.global_settings.enable_overlay = existing_global.global_settings.enable_overlay;
        }
        let _ = state.config_store.save_global_config(&cfg).await;
        // Pre-register any newly added client slots dynamically
        for &slot_id in &cfg.global_settings.client_slots {
            if state.client_manager.get_client(slot_id).await.is_none() {
                let alias = cfg.global_settings.client_aliases.get(&slot_id.to_string()).cloned();
                let proxy = cfg.global_settings.client_proxies.get(&slot_id.to_string()).cloned();
                let user_agent = cfg.global_settings.client_user_agents.get(&slot_id.to_string()).cloned();
                state.client_manager.register_client(crate::browser::ClientSlot {
                    id: slot_id,
                    is_running: false,
                    is_paused: false,
                    alias,
                    proxy,
                    user_agent,
                    debug_port: None,
                    pid: None,
                }).await;
            }
        }
    }

    if let Some(profiles_obj) = payload.get("profiles").and_then(|v| v.as_object()) {
        for (_, p_val) in profiles_obj {
            if let Ok(p) = serde_json::from_value::<Profile>(p_val.clone()) {
                let _ = state.config_store.save_profile(&p).await;
            }
        }
    } else if let (Some(name), Some(p_val)) = (
        payload.get("profileName").and_then(|v| v.as_str()),
        payload.get("profileData"),
    ) {
        if let Ok(mut p) = serde_json::from_value::<Profile>(p_val.clone()) {
            p.name = name.to_string();
            let _ = state.config_store.save_profile(&p).await;
            state.broadcast_profile_event("saved", name);
        }
    }

    if state.is_running.load(std::sync::atomic::Ordering::SeqCst) {
        let global = state.config_store.get_global_config().await;
        let mut profiles_to_load = Vec::new();
        let profile_names = if !global.active_profiles.is_empty() {
            global.active_profiles.clone()
        } else if !global.active_profile.is_empty() {
            vec![global.active_profile.clone()]
        } else {
            vec![]
        };

        for name in &profile_names {
            if let Some(mut prof) = state.config_store.get_profile(name).await {
                prof.normalize();
                prof.sanitize_profile_keys();
                profiles_to_load.push(prof);
            }
        }

        let loaded_count = profiles_to_load.len();
        {
            let mut engine = state.engine.write().await;
            engine.load_profiles(profiles_to_load);
        }
        state.log("system", &format!("🔄 Reloaded {} active profile(s) into running engine.", loaded_count));
    }

    state.log("info", "Configuration saved successfully.");
    let global_after = state.config_store.get_global_config().await;
    if state.is_running.load(std::sync::atomic::Ordering::SeqCst) && global_after.global_settings.enable_overlay {
        let _ = state.window_action_sender.send(crate::state::WindowAction::OverlayShow);
    }
    state.notify_status_change().await;
    Json(json!({ "success": true }))
}

pub async fn get_profile_data(
    State(state): State<AppState>,
    Query(query): Query<NameQuery>,
) -> impl IntoResponse {
    let name = query.name.unwrap_or_else(|| "Default".to_string());
    if let Some(profile) = state.config_store.get_profile(&name).await {
        Json(json!({ "success": true, "profile": profile })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Profile not found" })),
        )
            .into_response()
    }
}

pub async fn toggle_suspend(State(state): State<AppState>) -> impl IntoResponse {
    let curr = state.hotkey_listener.is_suspended();
    state.hotkey_listener.set_suspended(!curr);
    let new_state = !curr;
    state.log(
        "system",
        &format!(
            "Hotkey Sentinel state toggled: {}",
            if new_state { "SUSPENDED" } else { "ACTIVE" }
        ),
    );
    state.notify_status_change().await;

    Json(json!({ "success": true, "isSuspended": new_state }))
}

pub async fn start_engine(State(state): State<AppState>) -> impl IntoResponse {
    let license = crate::security::check_license_status(Some(&state.config_store.base_dir));
    if !license.is_valid {
        state.log("error", &format!("🔒 Cannot start engine: License is {}", license.status));
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error": format!("Cannot start engine: License status is '{}'. Please activate a valid license key in Settings.", license.status),
                "licenseStatus": license
            })),
        ).into_response();
    }

    // Load currently active profiles from disk into the Execution Engine
    let global = state.config_store.get_global_config().await;
    let mut profiles_to_load = Vec::new();
    let profile_names = if !global.active_profiles.is_empty() {
        global.active_profiles.clone()
    } else if !global.active_profile.is_empty() {
        vec![global.active_profile.clone()]
    } else {
        vec![]
    };

    for name in &profile_names {
        if let Some(mut prof) = state.config_store.get_profile(name).await {
            prof.normalize();
            profiles_to_load.push(prof);
        }
    }

    let loaded_count = profiles_to_load.len();
    {
        let mut engine = state.engine.write().await;
        engine.load_profiles(profiles_to_load);
    }

    state.is_running.store(true, Ordering::SeqCst);
    state.log(
        "system",
        &format!("🚀 Bot Execution Engine started (Loaded {} active profile(s): {}).", loaded_count, profile_names.join(", ")),
    );
    if global.global_settings.enable_overlay {
        let _ = state.window_action_sender.send(crate::state::WindowAction::OverlayShow);
    }
    state.notify_status_change().await;
    Json(json!({ "success": true, "running": true, "isRunning": true, "loadedProfiles": loaded_count })).into_response()
}

pub async fn stop_engine(State(state): State<AppState>) -> impl IntoResponse {
    state.reset_and_stop_engine().await;
    let _ = state.window_action_sender.send(crate::state::WindowAction::OverlayHide);
    Json(json!({ "success": true, "running": false, "isRunning": false }))
}

pub async fn get_active_clients(State(state): State<AppState>) -> impl IntoResponse {
    let clients = state.client_manager.get_all_clients().await;
    Json(json!({ "success": true, "clients": clients }))
}

#[derive(Deserialize)]
pub struct ClientActionPayload {
    pub client: Option<u32>,
    pub id: Option<u32>,
    #[serde(rename = "clientIndex")]
    pub client_index: Option<u32>,
    #[serde(rename = "browserChoice")]
    pub browser_choice: Option<String>,
}

impl ClientActionPayload {
    pub fn client_id(&self) -> u32 {
        self.client_index.or(self.client).or(self.id).unwrap_or(1)
    }

    pub fn browser_type(&self) -> Option<crate::browser::BrowserType> {
        self.browser_choice.as_ref().and_then(|b| {
            if b.starts_with('1') {
                Some(crate::browser::BrowserType::Chrome)
            } else if b.starts_with('2') {
                Some(crate::browser::BrowserType::Edge)
            } else if b.starts_with('3') {
                Some(crate::browser::BrowserType::Firefox)
            } else {
                None
            }
        })
    }

    pub fn is_app_mode(&self) -> Option<bool> {
        self.browser_choice.as_ref().map(|b| b.contains("app"))
    }
}

pub async fn launch_client(
    State(state): State<AppState>,
    Json(payload): Json<ClientActionPayload>,
) -> impl IntoResponse {
    let client_id = payload.client_id();
    let debug_port = 9222 + (client_id as u16);
    let user_data = state
        .config_store
        .base_dir
        .join(format!("profiles_data/client_{}", client_id));
    let target_url = "https://universe.flyff.com/play";

    let global = state.config_store.get_global_config().await;
    let proxy = global
        .global_settings
        .client_proxies
        .get(&client_id.to_string())
        .map(|s| s.as_str());
    let user_agent = global
        .global_settings
        .client_user_agents
        .get(&client_id.to_string())
        .map(|s| s.as_str());

    let browser_pref = payload.browser_type().or_else(|| {
        global
            .global_settings
            .client_browsers
            .get(&client_id.to_string())
            .and_then(|b| {
                if b.starts_with('1') {
                    Some(crate::browser::BrowserType::Chrome)
                } else if b.starts_with('2') {
                    Some(crate::browser::BrowserType::Edge)
                } else if b.starts_with('3') {
                    Some(crate::browser::BrowserType::Firefox)
                } else {
                    None
                }
            })
    });

    let app_mode = payload
        .is_app_mode()
        .or_else(|| {
            global
                .global_settings
                .client_browsers
                .get(&client_id.to_string())
                .map(|b| b.contains("app"))
        })
        .unwrap_or(true);

    let window_bounds = global
        .global_settings
        .client_window_bounds
        .get(&client_id.to_string())
        .and_then(|v| {
            let x = v.get("x").and_then(|n| n.as_i64())? as i32;
            let y = v.get("y").and_then(|n| n.as_i64())? as i32;
            let w = v.get("w").or_else(|| v.get("width")).and_then(|n| n.as_u64())? as u32;
            let h = v.get("h").or_else(|| v.get("height")).and_then(|n| n.as_u64())? as u32;
            if w >= 200 && h >= 200 {
                Some((x, y, w, h))
            } else {
                None
            }
        });

    if let Some((x, y, w, h)) = window_bounds {
        state.log(
            "system",
            &format!(
                "📐 Restoring Client {} window bounds: Position ({}, {}), Size ({}x{})",
                client_id, x, y, w, h
            ),
        );
    }

    match crate::browser::BrowserLauncher::launch(
        client_id,
        debug_port,
        &user_data,
        target_url,
        proxy,
        user_agent,
        browser_pref,
        app_mode,
        window_bounds,
    ) {
        Ok(child) => {
            let pid = child.id();
            state
                .client_manager
                .set_running(client_id, true, Some(debug_port), Some(pid))
                .await;
            state.notify_status_change().await;
            state.log(
                "system",
                &format!(
                    "🚀 Client {} launched (PID: {}, Debug Port: {})",
                    client_id, pid, debug_port
                ),
            );

            // Connect CDP in background (Pure Event-Driven WebSocket Sentinel)
            let state_cdp = state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                if let Ok(session) = crate::browser::CdpSession::connect(client_id, debug_port, state_cdp.clone()).await {
                    let global = state_cdp.config_store.get_global_config().await;
                    let alias = global
                        .global_settings
                        .client_aliases
                        .get(&client_id.to_string())
                        .cloned()
                        .unwrap_or_default();
                    let prefix = if !alias.trim().is_empty() {
                        format!("[{}] ", alias.trim())
                    } else {
                        format!("[Client {}] ", client_id)
                    };
                    let _ = session.set_tab_title(&prefix).await;
                    let _ = session.inject_window_bounds_binding().await;

                    // Re-apply after 2.5s once game scripts finish initial DOM/title boot
                    let session_clone = session.clone();
                    let prefix_clone = prefix.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(2500)).await;
                        let _ = session_clone.set_tab_title(&prefix_clone).await;
                        let _ = session_clone.inject_window_bounds_binding().await;
                    });

                    {
                        let mut lock = state_cdp.cdp_sessions.write().await;
                        lock.insert(client_id, session);
                    }
                    state_cdp
                        .client_manager
                        .set_running(client_id, true, Some(debug_port), Some(pid))
                        .await;
                    state_cdp.log("system", &format!("⚡ Attached CDP to Client {} (Title: {}Flyff Universe)", client_id, prefix));
                    state_cdp.notify_status_change().await;
                }
            });

            // Watch process exit in background to detect when user closes the browser window
            let state_watcher = state.clone();
            tokio::task::spawn_blocking(move || {
                let mut c = child;
                let _ = c.wait(); // Blocks until browser window is closed by user

                tokio::spawn(async move {
                    // Grace period check: If CDP session is actively running, do not prematurely mark offline
                    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                    let is_cdp_active = {
                        let lock = state_watcher.cdp_sessions.read().await;
                        lock.contains_key(&client_id)
                    };
                    if !is_cdp_active {
                        state_watcher.client_manager.set_running(client_id, false, None, None).await;
                        state_watcher.log("system", &format!("Client {} window closed by user.", client_id));
                        state_watcher.notify_status_change().await;
                    }
                });
            });

            Json(json!({ "success": true, "clientId": client_id, "pid": pid })).into_response()
        }
        Err(e) => {
            state.log("error", &format!("Failed to launch Client {}: {}", client_id, e));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": e })),
            )
                .into_response()
        }
    }
}

pub async fn close_client(
    State(state): State<AppState>,
    Json(payload): Json<ClientActionPayload>,
) -> impl IntoResponse {
    let client_id = payload.client_id();
    state.terminate_client(client_id).await;
    Json(json!({ "success": true, "clientId": client_id }))
}

#[derive(Debug, Deserialize)]
pub struct WindowBoundsPayload {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

pub async fn update_client_bounds(
    State(state): State<AppState>,
    AxumPath(client_id): AxumPath<u32>,
    Json(payload): Json<WindowBoundsPayload>,
) -> impl IntoResponse {
    if payload.w >= 200 && payload.h >= 200 && payload.x > -10000 && payload.y > -10000 {
        crate::browser::save_client_bounds_if_changed(
            &state,
            client_id,
            payload.x,
            payload.y,
            payload.w,
            payload.h,
            false,
        )
        .await;
    }
    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub struct ToggleEnablePayload {
    pub client: Option<u32>,
    #[serde(rename = "clientIndex")]
    pub client_index: Option<u32>,
    pub enabled: Option<bool>,
}

impl ToggleEnablePayload {
    pub fn client_id(&self) -> u32 {
        self.client_index.or(self.client).unwrap_or(1)
    }
}

pub async fn toggle_client_enable(
    State(state): State<AppState>,
    Json(payload): Json<ToggleEnablePayload>,
) -> impl IntoResponse {
    let client_id = payload.client_id();
    let mut global = state.config_store.get_global_config().await;
    let currently_disabled = global.disabled_clients.contains(&client_id);
    let should_enable = payload.enabled.unwrap_or(currently_disabled);

    if should_enable {
        global.disabled_clients.retain(|&c| c != client_id);
    } else if !currently_disabled {
        global.disabled_clients.push(client_id);
    }

    let _ = state.config_store.save_global_config(&global).await;
    Json(json!({ "success": true, "disabledClients": global.disabled_clients }))
}

pub async fn reset_client_bounds(
    State(state): State<AppState>,
    Json(payload): Json<ClientActionPayload>,
) -> impl IntoResponse {
    let client_id = payload.client_id();
    let mut global = state.config_store.get_global_config().await;
    global
        .global_settings
        .client_window_bounds
        .remove(&client_id.to_string());
    let _ = state.config_store.save_global_config(&global).await;
    Json(json!({ "success": true }))
}

pub async fn clear_client_profile(
    State(state): State<AppState>,
    Json(payload): Json<ClientActionPayload>,
) -> impl IntoResponse {
    let client_id = payload.client_id();
    let base = &state.config_store.base_dir;

    // 1. Remove V4 isolated profiles_data/client_{client_id}
    let profile_dir = base.join(format!("profiles_data/client_{}", client_id));
    if profile_dir.exists() {
        let _ = std::fs::remove_dir_all(&profile_dir);
    }

    // 2. Remove legacy profile directories if present
    let legacy_candidates = [
        if client_id == 1 { "chrome-profile".to_string() } else { format!("chrome-profile-{}", client_id) },
        if client_id == 1 { "edge-profile".to_string() } else { format!("edge-profile-{}", client_id) },
        if client_id == 1 { "firefox-profile".to_string() } else { format!("firefox-profile-{}", client_id) },
    ];
    for dir_name in &legacy_candidates {
        let target = base.join("profiles").join(dir_name);
        if target.exists() {
            let _ = std::fs::remove_dir_all(&target);
        }
    }

    state.log("system", &format!("🗑️ Cleared browser profile data for Client {}", client_id));
    Json(json!({ "success": true, "clientId": client_id }))
}

#[derive(serde::Deserialize)]
pub struct ClientAliasPayload {
    #[serde(rename = "clientIndex")]
    pub client_index: u32,
    pub alias: Option<String>,
}

pub async fn set_client_alias(
    State(state): State<AppState>,
    Json(payload): Json<ClientAliasPayload>,
) -> impl IntoResponse {
    let client_id = payload.client_index;
    let clean_alias = payload.alias.unwrap_or_default().trim().to_string();

    // 1. Update config_store
    let mut global = state.config_store.get_global_config().await;
    global.global_settings.client_aliases.insert(client_id.to_string(), clean_alias.clone());
    let _ = state.config_store.save_global_config(&global).await;
    state.notify_status_change().await;

    // 2. Format Option 2: [Knight] Flyff Universe
    let prefix = if !clean_alias.is_empty() {
        format!("[{}] ", clean_alias)
    } else {
        format!("[Client {}] ", client_id)
    };

    // 3. If client has an active CDP session, update tab title in real-time
    let sessions = state.cdp_sessions.read().await;
    let updated_live = if let Some(session) = sessions.get(&client_id) {
        let _ = session.set_tab_title(&prefix).await;
        state.log("system", &format!("🏷️ Updated Client {} tab title to '{}Flyff Universe'", client_id, prefix));
        true
    } else {
        false
    };

    Json(json!({
        "success": true,
        "clientId": client_id,
        "alias": clean_alias,
        "tabTitle": format!("{}Flyff Universe", prefix),
        "updatedLive": updated_live
    }))
}

pub async fn get_client_frame(AxumPath(client_id): AxumPath<u32>) -> impl IntoResponse {
    // 1x1 black JPEG fallback placeholder
    let fallback_jpg: &[u8] = &[
        0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00,
        0x60, 0x00, 0x60, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06,
        0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0a, 0x0c, 0x14, 0x0d, 0x0c, 0x0b, 0x0b,
        0x0c, 0x19, 0x12, 0x13, 0x0f, 0x14, 0x1d, 0x1a, 0x1f, 0x1e, 0x1d, 0x1a, 0x1c, 0x1c, 0x20,
        0x24, 0x2e, 0x27, 0x20, 0x22, 0x2c, 0x23, 0x1c, 0x1c, 0x28, 0x37, 0x29, 0x2c, 0x30, 0x31,
        0x34, 0x34, 0x34, 0x1f, 0x27, 0x39, 0x3d, 0x38, 0x32, 0x3c, 0x2e, 0x33, 0x34, 0x32, 0xff,
        0xc0, 0x00, 0x0b, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xff, 0xc4, 0x00,
        0x1f, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00, 0xbf, 0x00, 0xff, 0xd9,
    ];

    let _ = client_id;
    (
        [
            ("Content-Type", "image/jpeg"),
            ("Access-Control-Allow-Origin", "*"),
            ("Cache-Control", "no-cache, no-store"),
        ],
        fallback_jpg,
    )
}

pub async fn get_nodes() -> impl IntoResponse {
    let nodes = crate::engine::NodeRegistry::get_all_definitions();
    Json(json!({ "success": true, "nodes": nodes }))
}

pub async fn get_cooldown_presets_handler(State(state): State<AppState>) -> impl IntoResponse {
    let data = crate::server::cooldown::get_cooldown_presets(&state.config_store.base_dir);
    Json(data)
}

pub async fn get_update_check() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "hasUpdate": false,
        "currentVersion": "4.0.0",
        "latestVersion": "4.0.0"
    }))
}

// ─── Community Hub Endpoints ───
pub async fn get_creator_identity_handler(State(state): State<AppState>) -> impl IntoResponse {
    let identity = state.config_store.get_creator_identity().await;
    Json(json!({ "success": true, "identity": identity }))
}

#[derive(Deserialize)]
pub struct UpdateIdentityPayload {
    #[serde(rename = "authorName")]
    pub author_name: Option<String>,
}

pub async fn update_creator_identity_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdateIdentityPayload>,
) -> impl IntoResponse {
    if let Some(name) = payload.author_name {
        match state.config_store.update_creator_identity(&name).await {
            Ok(id) => Json(json!({ "success": true, "identity": id })).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))).into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "authorName required" })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub struct CommunityInstallPayload {
    #[serde(rename = "profileName")]
    pub profile_name: Option<String>,
    #[serde(rename = "profileData")]
    pub profile_data: Option<serde_json::Value>,
    pub overwrite: Option<bool>,
}

pub async fn install_community_profile(
    State(state): State<AppState>,
    Json(payload): Json<CommunityInstallPayload>,
) -> impl IntoResponse {
    let p_name = match payload.profile_name {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "profileName required" })),
            )
                .into_response()
        }
    };
    let p_data = match payload.profile_data {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "profileData required" })),
            )
                .into_response()
        }
    };

    if let Ok(mut profile) = serde_json::from_value::<Profile>(p_data) {
        profile.name = p_name.clone();
        let _ = state.config_store.save_profile(&profile).await;
        state.broadcast_profile_event("created", &p_name);
        Json(json!({ "success": true, "installedName": p_name })).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid profileData JSON" })),
        )
            .into_response()
    }
}

pub async fn get_profile_for_share(
    State(state): State<AppState>,
    Query(q): Query<NameQuery>,
) -> impl IntoResponse {
    let global = state.config_store.get_global_config().await;
    let target_name = q.name.unwrap_or(global.active_profile);
    if let Some(profile) = state.config_store.get_profile(&target_name).await {
        let identity = state.config_store.get_creator_identity().await;
        Json(json!({
            "success": true,
            "profileName": target_name,
            "profile": profile,
            "identity": identity
        }))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Profile not found" })),
        )
            .into_response()
    }
}

// ─── Profile Operations ───
#[derive(Deserialize)]
pub struct ActivateProfilePayload {
    pub name: String,
}

pub async fn activate_profile(
    State(state): State<AppState>,
    Json(payload): Json<ActivateProfilePayload>,
) -> impl IntoResponse {
    let mut global = state.config_store.get_global_config().await;
    global.active_profile = payload.name.clone();
    global.active_profiles = vec![payload.name.clone()];
    let _ = state.config_store.save_global_config(&global).await;
    state.broadcast_profile_event("activate", &payload.name);
    Json(json!({ "success": true, "activeProfiles": global.active_profiles }))
}

#[derive(Deserialize)]
pub struct SetActiveListPayload {
    #[serde(rename = "activeProfiles")]
    pub active_profiles: Vec<String>,
}

pub async fn set_active_list(
    State(state): State<AppState>,
    Json(payload): Json<SetActiveListPayload>,
) -> impl IntoResponse {
    let mut global = state.config_store.get_global_config().await;
    global.active_profiles = payload.active_profiles;
    if let Some(first) = global.active_profiles.first() {
        global.active_profile = first.clone();
    }
    let _ = state.config_store.save_global_config(&global).await;
    state.broadcast_profile_event("set_active_list", &global.active_profile);
    Json(json!({
        "success": true,
        "activeProfiles": global.active_profiles,
        "activeProfile": global.active_profile
    }))
}

#[derive(Deserialize)]
pub struct ToggleActivePayload {
    pub name: String,
    pub active: Option<bool>,
}

pub async fn toggle_active_profile(
    State(state): State<AppState>,
    Json(payload): Json<ToggleActivePayload>,
) -> impl IntoResponse {
    let mut global = state.config_store.get_global_config().await;
    let currently_active = global.active_profiles.contains(&payload.name);
    let should_be_active = payload.active.unwrap_or(!currently_active);

    if should_be_active {
        if !currently_active {
            global.active_profiles.push(payload.name.clone());
        }
    } else {
        global.active_profiles.retain(|p| p != &payload.name);
    }
    if let Some(first) = global.active_profiles.first() {
        global.active_profile = first.clone();
    }
    let _ = state.config_store.save_global_config(&global).await;
    state.broadcast_profile_event("toggle_active", &payload.name);
    Json(json!({
        "success": true,
        "activeProfiles": global.active_profiles,
        "activeProfile": global.active_profile
    }))
}

#[derive(Deserialize)]
pub struct NewProfilePayload {
    pub name: String,
    #[serde(rename = "copyFrom")]
    pub copy_from: Option<String>,
}

pub async fn new_profile(
    State(state): State<AppState>,
    Json(payload): Json<NewProfilePayload>,
) -> impl IntoResponse {
    let mut base_profile = if let Some(ref source) = payload.copy_from {
        state
            .config_store
            .get_profile(source)
            .await
            .unwrap_or_else(|| Profile {
                name: payload.name.clone(),
                ..Default::default()
            })
    } else {
        Profile {
            name: payload.name.clone(),
            ..Default::default()
        }
    };

    base_profile.name = payload.name.clone();
    let _ = state.config_store.save_profile(&base_profile).await;
    state.broadcast_profile_event("created", &payload.name);
    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub struct DeleteProfilePayload {
    pub name: String,
}

pub async fn delete_profile(
    State(state): State<AppState>,
    Json(payload): Json<DeleteProfilePayload>,
) -> impl IntoResponse {
    let _ = state.config_store.delete_profile(&payload.name).await;
    let mut global = state.config_store.get_global_config().await;
    global.active_profiles.retain(|p| p != &payload.name);
    if global.active_profile == payload.name {
        global.active_profile = global
            .active_profiles
            .first()
            .cloned()
            .unwrap_or_else(|| "Default".to_string());
    }
    let _ = state.config_store.save_global_config(&global).await;
    state.broadcast_profile_event("deleted", &payload.name);
    Json(json!({ "success": true, "activeProfiles": global.active_profiles }))
}

#[derive(Deserialize)]
pub struct RenameProfilePayload {
    #[serde(rename = "oldName")]
    pub old_name: String,
    #[serde(rename = "newName")]
    pub new_name: String,
}

pub async fn rename_profile(
    State(state): State<AppState>,
    Json(payload): Json<RenameProfilePayload>,
) -> impl IntoResponse {
    let _ = state
        .config_store
        .rename_profile(&payload.old_name, &payload.new_name)
        .await;
    let mut global = state.config_store.get_global_config().await;
    for p in &mut global.active_profiles {
        if *p == payload.old_name {
            *p = payload.new_name.clone();
        }
    }
    if global.active_profile == payload.old_name {
        global.active_profile = payload.new_name.clone();
    }
    let _ = state.config_store.save_global_config(&global).await;
    state.broadcast_profile_event("renamed", &payload.new_name);
    Json(json!({ "success": true, "activeProfiles": global.active_profiles }))
}

#[derive(Deserialize)]
pub struct OverwriteProfilePayload {
    pub name: String,
    #[serde(rename = "profileData")]
    pub profile_data: Profile,
}

pub async fn overwrite_profile(
    State(state): State<AppState>,
    Json(payload): Json<OverwriteProfilePayload>,
) -> impl IntoResponse {
    let mut p = payload.profile_data;
    p.name = payload.name.clone();
    let _ = state.config_store.save_profile(&p).await;
    state.broadcast_profile_event("overwrite", &payload.name);
    Json(json!({ "success": true }))
}

// ─── Utility Endpoints ───
pub async fn disable_overlay(State(state): State<AppState>) -> impl IntoResponse {
    let mut global = state.config_store.get_global_config().await;
    global.global_settings.enable_overlay = false;
    let _ = state.config_store.save_global_config(&global).await;
    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub struct SoundUploadPayload {
    pub filename: String,
    #[serde(rename = "base64Data")]
    pub base64_data: String,
}

pub async fn upload_sound(
    State(state): State<AppState>,
    Json(payload): Json<SoundUploadPayload>,
) -> impl IntoResponse {
    let sounds_dir = state.config_store.base_dir.join("public/sounds");
    let _ = std::fs::create_dir_all(&sounds_dir);
    let clean_name = std::path::Path::new(&payload.filename)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "sound.mp3".to_string());

    let target = sounds_dir.join(&clean_name);
    use base64::Engine;
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(payload.base64_data.trim())
    {
        let _ = std::fs::write(&target, bytes);
        Json(json!({ "success": true, "url": format!("/sounds/{}", clean_name), "filename": clean_name })).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid base64 audio data" })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub struct OpenFolderPayload {
    pub subfolder: Option<String>,
}

pub async fn open_folder(
    State(state): State<AppState>,
    Json(payload): Json<OpenFolderPayload>,
) -> impl IntoResponse {
    let sub = payload.subfolder.unwrap_or_default();
    let target = state.config_store.base_dir.join("screenshots").join(sub);
    let _ = std::fs::create_dir_all(&target);

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(&target).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&target).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open")
        .arg(&target)
        .spawn();

    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub struct WebhookTestPayload {
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub payload: Option<serde_json::Value>,
    pub timeout_ms: Option<u64>,
    #[serde(rename = "timeoutMs")]
    pub timeout_ms_camel: Option<u64>,
}

pub async fn test_webhook(Json(payload): Json<WebhookTestPayload>) -> impl IntoResponse {
    let url = match payload.url.filter(|u| !u.trim().is_empty()) {
        Some(u) => u.trim().to_string(),
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "URL required" }))).into_response(),
    };

    let method_str = payload.method.unwrap_or_else(|| "POST".to_string()).to_uppercase();
    let timeout_ms = payload.timeout_ms_camel.or(payload.timeout_ms).unwrap_or(5000);

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build() {
            Ok(c) => c,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
        };

    let method = match method_str.as_str() {
        "GET" => reqwest::Method::GET,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        _ => reqwest::Method::POST,
    };

    let mut req = client.request(method, &url);

    // Apply headers
    if let Some(h) = payload.headers {
        if let Some(obj) = h.as_object() {
            for (k, v) in obj {
                if let Some(val_str) = v.as_str() {
                    req = req.header(k, val_str);
                }
            }
        } else if let Some(s) = h.as_str() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(s) {
                for (k, v) in parsed {
                    if let Some(val_str) = v.as_str() {
                        req = req.header(&k, val_str);
                    }
                }
            }
        }
    }

    // Apply payload body for POST/PUT
    if method_str == "POST" || method_str == "PUT" {
        if let Some(p) = payload.payload {
            if let Some(s) = p.as_str() {
                if s.trim().starts_with('{') || s.trim().starts_with('[') {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(s) {
                        req = req.header("Content-Type", "application/json").json(&json_val);
                    } else {
                        req = req.body(s.to_string());
                    }
                } else {
                    req = req.body(s.to_string());
                }
            } else if p.is_object() || p.is_array() {
                req = req.header("Content-Type", "application/json").json(&p);
            }
        } else {
            req = req.header("Content-Type", "application/json").json(&json!({
                "content": "🔔 [NodeHotkey] Webhook Test Alert from Rust Native Engine!"
            }));
        }
    }

    match req.send().await {
        Ok(res) => {
            let status = res.status();
            let is_success = status.is_success();
            let status_code = status.as_u16();
            let reason = status.canonical_reason().unwrap_or("").to_string();
            Json(json!({
                "success": is_success,
                "status": status_code,
                "statusText": reason,
                "error": if is_success { None } else { Some(format!("HTTP error: {}", status_code)) }
            })).into_response()
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "status": 0,
                "error": e.to_string()
            })).into_response()
        }
    }
}

pub async fn trigger_event(
    State(state): State<AppState>,
    AxumPath(event_name): AxumPath<String>,
) -> impl IntoResponse {
    state.handle_key_trigger(&event_name).await;
    Json(json!({ "success": true, "triggered": event_name }))
}

// ─── Real-time SSE Streams ───

/// Canvas Real-time Signal Stream (/api/signals/stream)
/// Emits array batches [sig] exactly as canvas.js expects
pub async fn signals_sse_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.signal_broadcaster.subscribe();
    let stream = async_stream::stream! {
        yield Ok(Event::default().comment("connected"));

        loop {
            match tokio::time::timeout(tokio::time::Duration::from_secs(10), rx.recv()).await {
                Ok(Ok(sig)) => {
                    let arr = serde_json::json!([sig]);
                    yield Ok(Event::default().data(arr.to_string()));
                }
                Ok(Err(_)) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(_) => {
                    // Periodic keepalive comment
                    yield Ok(Event::default().comment("ping"));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

/// Profile External Change Stream (/api/profile-events/stream)
/// Notifies state.js when profiles are edited or reloaded
pub async fn profile_events_sse_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.profile_event_sender.subscribe();
    let mut status_rx = state.status_event_sender.subscribe();
    let stream = async_stream::stream! {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let init = serde_json::json!({
            "type": "CONNECTED",
            "timestamp": now
        });
        yield Ok(Event::default().data(init.to_string()));

        loop {
            tokio::select! {
                Ok(msg) = rx.recv() => {
                    yield Ok(Event::default().data(msg));
                }
                Ok(status) = status_rx.recv() => {
                    let evt = serde_json::json!({
                        "type": "STATUS_UPDATE",
                        "activeClients": status.get("activeClients"),
                        "disabledClients": status.get("disabledClients"),
                        "enableOverlay": status.get("enableOverlay")
                    });
                    yield Ok(Event::default().data(evt.to_string()));
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {
                    yield Ok(Event::default().comment("ping"));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

/// Unified Telemetry & Logs SSE Stream (/api/profiles/stream)
pub async fn profiles_sse_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut sig_rx = state.signal_broadcaster.subscribe();
    let mut log_rx = state.log_sender.subscribe();
    let mut status_rx = state.status_event_sender.subscribe();

    let stream = async_stream::stream! {
        loop {
            tokio::select! {
                Ok(sig) = sig_rx.recv() => {
                    let data = json!({
                        "type": "signal",
                        "actionId": sig.action_id,
                        "eventName": sig.event_name,
                        "targetId": sig.target_id,
                        "timestamp": sig.timestamp
                    });
                    yield Ok(Event::default().data(data.to_string()));
                }
                Ok(log) = log_rx.recv() => {
                    let data = json!({
                        "type": "log",
                        "level": log.level,
                        "message": log.message,
                        "text": log.message,
                        "timestamp": log.timestamp
                    });
                    yield Ok(Event::default().data(data.to_string()));
                }
                Ok(status) = status_rx.recv() => {
                    let data = json!({
                        "type": "status_update",
                        "data": status
                    });
                    yield Ok(Event::default().data(data.to_string()));
                }
                else => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        }
    };

    Sse::new(stream)
}

/// Fallback route to serve static files from `public/` and `launcher/ui/`
pub async fn static_fallback(
    State(state): State<AppState>,
    uri: axum::http::Uri,
) -> Response {
    let path_str = uri.path();
    let base_dir = &state.config_store.base_dir;

    // Reject unknown API endpoints with JSON 404 instead of returning index.html
    if path_str.starts_with("/api/") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": format!("API endpoint not found: {}", path_str)
            })),
        )
            .into_response();
    }

    // HTML entry points
    if path_str == "/launcher" || path_str == "/launcher/" || path_str == "/launcher/index.html" {
        return serve_file_from_disk(base_dir.join("launcher/ui/index.html")).await;
    }
    if path_str == "/" || path_str.is_empty() || path_str == "/index.html" {
        return serve_file_from_disk(base_dir.join("public/index.html")).await;
    }

    let clean_path = path_str.trim_start_matches('/');

    // 🔒 Path Traversal & Sensitive Directory Protection
    if clean_path.contains("..")
        || clean_path.starts_with("configs")
        || clean_path.starts_with("src")
        || clean_path.starts_with(".git")
        || clean_path.starts_with(".agents")
        || clean_path == "Cargo.toml"
        || clean_path == "Cargo.lock"
    {
        return (
            StatusCode::FORBIDDEN,
            [("Content-Type", "text/plain")],
            "Access forbidden",
        )
            .into_response();
    }

    // 1. Look inside public/
    let public_subpath = clean_path.strip_prefix("public/").unwrap_or(clean_path);
    let p_public = base_dir.join("public").join(public_subpath);
    if p_public.is_file() {
        return serve_file_from_disk(p_public).await;
    }

    // 2. Look inside launcher/ui/
    let launcher_subpath = clean_path
        .strip_prefix("launcher/ui/")
        .or_else(|| clean_path.strip_prefix("launcher/"))
        .unwrap_or(clean_path);
    let p_launcher = base_dir.join("launcher/ui").join(launcher_subpath);
    if p_launcher.is_file() {
        return serve_file_from_disk(p_launcher).await;
    }

    // 3. Allowed root icons
    if clean_path == "icon.png" || clean_path == "icon.ico" || clean_path == "favicon.ico" {
        let p_icon = base_dir.join(clean_path);
        if p_icon.is_file() {
            return serve_file_from_disk(p_icon).await;
        }
    }

    // 5. If the request was for an asset file (has extension), return 404 NOT HTML!
    if clean_path.contains('.') {
        return (
            StatusCode::NOT_FOUND,
            [("Content-Type", "text/plain")],
            format!("Asset not found: {}", path_str),
        )
            .into_response();
    }

    // 6. SPA fallback for client-side routing
    serve_file_from_disk(base_dir.join("public/index.html")).await
}

// Window control handlers
pub async fn window_minimize(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.window_action_sender.send(crate::state::WindowAction::Minimize);
    Json(json!({ "success": true }))
}

pub async fn window_maximize(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.window_action_sender.send(crate::state::WindowAction::ToggleMaximize);
    Json(json!({ "success": true }))
}

pub async fn window_hide(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.window_action_sender.send(crate::state::WindowAction::Hide);
    Json(json!({ "success": true }))
}

pub async fn window_show(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.window_action_sender.send(crate::state::WindowAction::Show);
    Json(json!({ "success": true }))
}

pub async fn window_close(State(state): State<AppState>) -> impl IntoResponse {
    state.terminate_all_clients().await;
    let _ = state.window_action_sender.send(crate::state::WindowAction::Close);
    Json(json!({ "success": true }))
}

// Overlay window handlers
pub async fn overlay_show(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.window_action_sender.send(crate::state::WindowAction::OverlayShow);
    Json(json!({ "success": true }))
}

pub async fn overlay_hide(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.window_action_sender.send(crate::state::WindowAction::OverlayHide);
    Json(json!({ "success": true }))
}

pub async fn overlay_toggle(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.window_action_sender.send(crate::state::WindowAction::OverlayToggle);
    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub struct OverlayResizePayload {
    pub width: Option<f64>,
    pub height: Option<f64>,
}

pub async fn overlay_resize(
    State(state): State<AppState>,
    Json(payload): Json<OverlayResizePayload>,
) -> impl IntoResponse {
    let width = payload.width.unwrap_or(210.0);
    let height = payload.height.unwrap_or(180.0);
    let _ = state.window_action_sender.send(crate::state::WindowAction::OverlayResize { width, height });
    Json(json!({ "success": true, "width": width, "height": height }))
}

