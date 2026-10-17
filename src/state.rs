use crate::audio::AudioPlayer;
use crate::browser::{CdpSession, ClientManager};
use crate::config::ConfigStore;
use crate::engine::{ActionExecutor, NodeExecutionEngine, SignalBroadcaster};
use crate::input::GlobalHotkeyListener;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: String, // "info" | "warn" | "error" | "action" | "log"
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowAction {
    Minimize,
    ToggleMaximize,
    Hide,
    Show,
    Close,
    OverlayShow,
    OverlayHide,
    OverlayToggle,
    OverlayResize { width: f64, height: f64 },
}

#[derive(Clone)]
pub struct AppState {
    pub config_store: ConfigStore,
    pub engine: Arc<RwLock<NodeExecutionEngine>>,
    pub client_manager: ClientManager,
    pub cdp_sessions: Arc<RwLock<HashMap<u32, CdpSession>>>,
    pub audio_player: Option<AudioPlayer>,
    pub tts_manager: crate::audio::TtsManager,
    pub hotkey_listener: Arc<GlobalHotkeyListener>,
    pub signal_broadcaster: SignalBroadcaster,
    pub log_sender: broadcast::Sender<LogMessage>,
    pub profile_event_sender: broadcast::Sender<String>,
    pub window_action_sender: broadcast::Sender<WindowAction>,
    pub status_event_sender: broadcast::Sender<serde_json::Value>,
    pub start_time: Instant,
    pub is_running: Arc<AtomicBool>,
    pub last_trigger: Arc<tokio::sync::Mutex<(String, Instant)>>,
    pub held_keys: Arc<RwLock<HashSet<(u32, String)>>>,
    pub skill_cooldowns: Arc<RwLock<HashMap<(u32, String), Instant>>>,
    pub cooldown_presets_map: Arc<RwLock<HashMap<String, (u64, String)>>>,
    pub stop_sender: broadcast::Sender<()>,
}

impl AppState {
    pub fn new(base_path: &std::path::Path) -> Self {
        let (log_sender, _) = broadcast::channel(500);
        let (profile_event_sender, _) = broadcast::channel(100);
        let (window_action_sender, _) = broadcast::channel(32);
        let (status_event_sender, _) = broadcast::channel(64);
        let (stop_sender, _) = broadcast::channel(128);

        Self {
            config_store: ConfigStore::new(base_path),
            engine: Arc::new(RwLock::new(NodeExecutionEngine::new())),
            client_manager: ClientManager::new(),
            cdp_sessions: Arc::new(RwLock::new(HashMap::new())),
            audio_player: AudioPlayer::try_new().ok(),
            tts_manager: crate::audio::TtsManager::new(base_path.join("cache").join("tts")),
            hotkey_listener: Arc::new(GlobalHotkeyListener::new()),
            signal_broadcaster: SignalBroadcaster::new(),
            log_sender,
            profile_event_sender,
            window_action_sender,
            status_event_sender,
            start_time: Instant::now(),
            is_running: Arc::new(AtomicBool::new(false)),
            last_trigger: Arc::new(tokio::sync::Mutex::new((
                String::new(),
                Instant::now() - std::time::Duration::from_secs(1),
            ))),
            held_keys: Arc::new(RwLock::new(HashSet::new())),
            skill_cooldowns: Arc::new(RwLock::new(HashMap::new())),
            cooldown_presets_map: Arc::new(RwLock::new(crate::server::cooldown::get_cooldown_presets_map(base_path))),
            stop_sender,
        }
    }

    pub fn log(&self, level: &str, msg: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let log_item = LogMessage {
            level: level.to_string(),
            message: msg.to_string(),
            timestamp: now,
        };

        println!("[{}] {}", level.to_uppercase(), msg);
        let _ = self.log_sender.send(log_item);
    }

    pub fn broadcast_profile_event(&self, change_type: &str, profile_name: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let payload = serde_json::json!({
            "type": "PROFILE_EXTERNAL_CHANGE",
            "profileName": profile_name,
            "changeType": change_type,
            "timestamp": now
        });

        let _ = self.profile_event_sender.send(payload.to_string());
    }

    pub async fn handle_key_trigger(&self, pressed_key: &str) {
        let norm_key = crate::engine::graph::normalize_trigger_key(pressed_key);

        // Deduplication Guard: Ignore identical key events fired within 80ms (e.g. DOM fallback + OS hook)
        {
            let mut last = self.last_trigger.lock().await;
            let now = Instant::now();
            if last.0 == norm_key && now.duration_since(last.1) < std::time::Duration::from_millis(80) {
                return;
            }
            *last = (norm_key.clone(), now);
        }

        // Suspend toggle check (e.g. END key)
        let global = self.config_store.get_global_config().await;
        let suspend_norm = crate::engine::graph::normalize_trigger_key(&global.global_settings.suspend_hotkey);
        if norm_key == suspend_norm {
            let prev = self.hotkey_listener.is_suspended();
            self.hotkey_listener.set_suspended(!prev);
            self.signal_broadcaster.emit(
                "sentinel",
                if !prev { "suspended" } else { "active" },
                None,
            );
            self.log(
                "system",
                &format!(
                    "Hotkey Sentinel state toggled: {}",
                    if !prev { "SUSPENDED" } else { "ACTIVE" }
                ),
            );
            if !prev {
                // If transitioning to SUSPENDED, halt all active action nodes and reset states
                let state_reset = self.clone();
                tokio::spawn(async move {
                    state_reset.reset_and_stop_engine().await;
                });
            } else {
                let state_notify = self.clone();
                tokio::spawn(async move {
                    state_notify.notify_status_change().await;
                });
            }
            return;
        }

        if self.hotkey_listener.is_suspended() {
            return;
        }
        if !self.is_running.load(Ordering::SeqCst) {
            return;
        }

        let state_clone = self.clone();
        let key = pressed_key.to_string();
        tokio::spawn(async move {
            crate::engine::ActionExecutor::trigger_flow(state_clone, "keyboard", &key).await;
        });
    }

    /// Fetch a full status snapshot JSON for Dashboard and SSE broadcasts
    pub async fn get_status_json(&self) -> serde_json::Value {
        let global = self.config_store.get_global_config().await;
        let is_running = self.is_running.load(Ordering::SeqCst);
        let is_suspended = self.hotkey_listener.is_suspended();
        let uptime = self.start_time.elapsed().as_secs();

        let clients = self.client_manager.get_all_clients().await;
        let cdp_sessions = self.cdp_sessions.read().await;
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
                serde_json::json!({
                    "status": if is_client_active { "Standby" } else { "Offline" },
                    "type": if is_client_active { "standby" } else { "offline" }
                }),
            );
        }

        serde_json::json!({
            "success": true,
            "running": is_running,
            "isRunning": is_running,
            "isSuspended": is_suspended,
            "port": global.global_settings.web_port,
            "serverPort": global.global_settings.web_port,
            "activeClients": active_clients,
            "clientStatuses": client_statuses,
            "clientAliases": global.global_settings.client_aliases,
            "disabledClients": global.disabled_clients,
            "enableOverlay": global.global_settings.enable_overlay,
            "activeProfiles": global.active_profiles,
            "activeProfile": global.active_profile,
            "uptime": uptime,
            "engineVersion": "4.0.0-rust",
            "license": crate::security::check_license_status(Some(&self.config_store.base_dir))
        })
    }

    /// Broadcast status change to all connected SSE clients (Launcher, PiP, Canvas)
    pub async fn notify_status_change(&self) {
        let status = self.get_status_json().await;
        let _ = self.status_event_sender.send(status);
    }

    /// Gracefully and forcefully terminate a specific browser client
    pub async fn terminate_client(&self, client_id: u32) {
        // 0. Snapshot and persist window bounds before termination
        if let Some(slot) = self.client_manager.get_client(client_id).await {
            if let Some(pid) = slot.pid {
                if let Some((x, y, w, h)) = crate::browser::get_client_window_bounds_from_pid(pid) {
                    crate::browser::save_client_bounds_if_changed(self, client_id, x, y, w, h, true).await;
                }
            }
        }

        // 1. Graceful close via CDP
        let session = {
            let mut lock = self.cdp_sessions.write().await;
            lock.remove(&client_id)
        };
        if let Some(s) = session {
            let _ = s.close_browser().await;
        }

        // 2. Forceful process tree termination via PID
        if let Some(slot) = self.client_manager.get_client(client_id).await {
            if let Some(pid) = slot.pid {
                crate::browser::BrowserLauncher::kill_process_tree(pid);
            }
        }

        // 3. Mark client as stopped in state
        self.client_manager.set_running(client_id, false, None, None).await;
        {
            let mut held = self.held_keys.write().await;
            held.retain(|(cid, _)| *cid != client_id);
        }
        self.log("system", &format!("🛑 Client {} terminated cleanly.", client_id));
        self.notify_status_change().await;
    }

    /// Terminate all running browser clients
    pub async fn terminate_all_clients(&self) {
        let clients = self.client_manager.get_all_clients().await;
        let mut count = 0;
        for c in clients {
            if c.is_running || c.pid.is_some() {
                self.terminate_client(c.id).await;
                count += 1;
            }
        }
        if count > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            self.log("system", &format!("🛑 Closed all {} active browser client(s).", count));
            self.notify_status_change().await;
        }
    }

    /// Immediately halts engine execution, cancels in-flight node delays/loops,
    /// releases all held keys across all CDP clients, resets variables to defaults,
    /// stops audio playback, and notifies UI clients.
    pub async fn reset_and_stop_engine(&self) {
        // 1. Mark engine as stopped
        self.is_running.store(false, Ordering::SeqCst);

        // 2. Broadcast cancellation token to all active loops/delays/schedulers
        let _ = self.stop_sender.send(());

        // 3. Stop audio/TTS playback if active
        if let Some(ref player) = self.audio_player {
            player.stop();
        }

        // 4. Release all held keys (key_hold) across all clients via CDP
        let held_list: Vec<(u32, String)> = {
            let mut lock = self.held_keys.write().await;
            lock.drain().collect()
        };
        for (client_id, key) in held_list {
            ActionExecutor::dispatch_cdp_key_hold(self, client_id, &key, false).await;
        }

        // 5. Reset all variables to profile default values
        {
            let mut engine = self.engine.write().await;
            engine.reset_variables_to_defaults();
        }

        // 6. Emit signal to clear UI visual indicators / running node glows
        self.signal_broadcaster.emit("engine", "stopped", None);

        // 7. Clear all active skill cooldowns
        {
            let mut cds = self.skill_cooldowns.write().await;
            cds.clear();
        }

        // 8. Log & notify status change to all SSE subscribers
        self.log("system", "⏹️ Bot Engine stopped: All active nodes aborted, held keys released & states reset.");
        self.notify_status_change().await;
    }

    /// Check if a skill or node is on cooldown for client_id.
    /// Returns Some(remaining_seconds) if currently on cooldown.
    /// Returns None if not on cooldown.
    pub async fn check_skill_cooldown(&self, client_id: u32, skill_id: &str) -> Option<f64> {
        let lock = self.skill_cooldowns.read().await;
        if let Some(expire) = lock.get(&(client_id, skill_id.to_string())) {
            let now = Instant::now();
            if now < *expire {
                let remaining = *expire - now;
                return Some(remaining.as_secs_f64());
            }
        }
        None
    }

    /// Set skill cooldown duration for client_id.
    pub async fn set_skill_cooldown(&self, client_id: u32, skill_id: &str, duration: Duration) {
        let mut lock = self.skill_cooldowns.write().await;
        lock.insert((client_id, skill_id.to_string()), Instant::now() + duration);
    }
}

