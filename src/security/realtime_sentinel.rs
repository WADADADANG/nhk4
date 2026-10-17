use std::sync::atomic::Ordering;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::state::AppState;

/// Default Pusher credentials for NodeHotKeyV4 Real-time License Sentinel
pub const DEFAULT_PUSHER_KEY: &str = "121974b05826cd21a5da";
pub const DEFAULT_PUSHER_CLUSTER: &str = "ap1";
pub const LICENSE_CHANNEL: &str = "nhk-licenses";

/// Autonomous Real-time License Sentinel using Pusher Channels (WebSocket)
/// Provides sub-second (0.1 - 0.3s) reaction time when admin suspends/bans/activates a license.
/// Zero polling, zero Cloudflare requests consumed during idle/running.
pub async fn start_realtime_license_sentinel(state: AppState) {
    let pusher_key = DEFAULT_PUSHER_KEY;
    let pusher_cluster = DEFAULT_PUSHER_CLUSTER;
    let ws_url = format!(
        "wss://ws-{}.pusher.com/app/{}?protocol=7&client=js&version=8.4.0-rc2&flash=false",
        pusher_cluster, pusher_key
    );

    println!("⚡ [License Sentinel] Initializing Real-time WebSocket Sentinel (Cluster: {})...", pusher_cluster);

    tokio::spawn(async move {
        // Initial grace delay 2 seconds after server boot
        tokio::time::sleep(Duration::from_secs(2)).await;

        loop {
            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    println!("✨ [License Sentinel] Connected to Real-time Sentinel (Pusher WSS)");
                    let (mut write, mut read) = ws_stream.split();

                    // Wait for connection_established, then subscribe to channel
                    while let Some(msg_res) = read.next().await {
                        match msg_res {
                            Ok(Message::Text(text)) => {
                                if let Ok(val) = serde_json::from_str::<Value>(&text) {
                                    let event_name = val.get("event").and_then(|e| e.as_str()).unwrap_or("");

                                    match event_name {
                                        "pusher:connection_established" => {
                                            // Subscribe to nhk-licenses channel
                                            let sub_payload = json!({
                                                "event": "pusher:subscribe",
                                                "data": {
                                                    "channel": LICENSE_CHANNEL
                                                }
                                            });
                                            let _ = write.send(Message::Text(sub_payload.to_string())).await;
                                        }
                                        "pusher:ping" => {
                                            // Respond to Pusher heartbeat ping
                                            let pong_payload = json!({
                                                "event": "pusher:pong",
                                                "data": {}
                                            });
                                            let _ = write.send(Message::Text(pong_payload.to_string())).await;
                                        }
                                        "license_updated" => {
                                            // Real-time license event triggered by admin
                                            handle_license_event(&state, &val).await;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Ok(Message::Ping(payload)) => {
                                let _ = write.send(Message::Pong(payload)).await;
                            }
                            Ok(Message::Close(_)) => {
                                eprintln!("⚠️ [License Sentinel] Real-time Sentinel connection closed by server.");
                                break;
                            }
                            Err(e) => {
                                eprintln!("⚠️ [License Sentinel] WebSocket read error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ [License Sentinel] Connection to Pusher failed: {}. Retrying in 10s...", e);
                }
            }

            // Exponential / fixed backoff before reconnecting
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
}

/// Process incoming real-time license updates from Pusher
async fn handle_license_event(state: &AppState, payload: &Value) {
    // Pusher sends data either as stringified JSON or an object
    let raw_data = payload.get("data");
    let data_val: Value = if let Some(str_val) = raw_data.and_then(|d| d.as_str()) {
        serde_json::from_str(str_val).unwrap_or_default()
    } else if let Some(obj) = raw_data {
        obj.clone()
    } else {
        Value::Null
    };

    let target_hwid = data_val.get("hwid").and_then(|h| h.as_str()).unwrap_or("").trim();
    let target_key = data_val.get("key").and_then(|k| k.as_str()).unwrap_or("").trim();
    let new_status = data_val.get("status").and_then(|s| s.as_str()).unwrap_or("").trim().to_uppercase();

    if target_hwid.is_empty() && target_key.is_empty() {
        return;
    }

    let current_hwid = crate::security::generate_hwid();
    let local_serial = crate::security::load_serial_file(Some(&state.config_store.base_dir));

    let is_targeted_hwid = !target_hwid.is_empty() && target_hwid.eq_ignore_ascii_case(&current_hwid);
    let is_targeted_key = !target_key.is_empty()
        && local_serial.as_ref().map(|s| s.eq_ignore_ascii_case(target_key)).unwrap_or(false);

    // If event is not addressed to this machine, ignore quietly
    if !is_targeted_hwid && !is_targeted_key {
        return;
    }

    println!("⚡ [License Sentinel] Real-time event received for this machine: Status -> {}", new_status);

    match new_status.as_str() {
        "SUSPENDED" => {
            let _ = crate::security::save_license_file("SUSPENDED", Some(&state.config_store.base_dir));

            if state.is_running.load(Ordering::SeqCst) {
                state.is_running.store(false, Ordering::SeqCst);
                state.log("error", "🔒 [Sentinel 0.2s] Engine halted: License was temporarily suspended by administrator.");
            } else {
                state.log("warn", "🔒 [Sentinel 0.2s] License status changed to SUSPENDED by administrator.");
            }

            // Real-time broadcast to all UIs (Launcher & Studio Canvas) via unified SSE
            state.notify_status_change().await;
        }
        "BANNED" | "REVOKED" | "DELETED" => {
            crate::security::purge_license_files(Some(&state.config_store.base_dir));

            if state.is_running.load(Ordering::SeqCst) {
                state.is_running.store(false, Ordering::SeqCst);
                state.log("error", "🔒 [Sentinel 0.2s] Engine halted: License was revoked or banned by administrator.");
            } else {
                state.log("error", "🔒 [Sentinel 0.2s] License was revoked or banned by administrator.");
            }

            state.notify_status_change().await;
        }
        "ACTIVATED" => {
            // Give Cloudflare D1 edge database 300ms to commit before requesting signed token
            tokio::time::sleep(Duration::from_millis(300)).await;

            let default_worker_url = crate::secure_str!("https://nodehotkey-api.kitsada19972540.workers.dev");
            let mut sync_res = crate::security::sync_license_from_server(
                &default_worker_url,
                &current_hwid,
                Some(&state.config_store.base_dir),
            ).await;

            // If first attempt caught edge cache before flush, retry once after 700ms
            if sync_res.is_err() {
                tokio::time::sleep(Duration::from_millis(700)).await;
                sync_res = crate::security::sync_license_from_server(
                    &default_worker_url,
                    &current_hwid,
                    Some(&state.config_store.base_dir),
                ).await;
            }

            match sync_res {
                Ok(claims) => {
                    state.log("system", &format!("✨ [Sentinel 0.2s] License reactivated successfully by administrator. Tier: {}", claims.tier));
                }
                Err(e) => {
                    state.log("warn", &format!("⚠️ [Sentinel 0.2s] License reactivated signal received, but sync returned: {}", e));
                }
            }

            state.notify_status_change().await;
        }
        _ => {}
    }
}
