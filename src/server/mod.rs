pub mod cooldown;
pub mod routes;
pub mod static_files;

use crate::state::AppState;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Status & Config
        .route("/api/status", get(routes::get_status))
        .route("/api/active-clients", get(routes::get_active_clients))
        .route("/api/config", get(routes::get_config).post(routes::save_config))
        .route("/api/update-check", get(routes::get_update_check))
        .route("/api/security/hwid", get(routes::get_hwid_handler))
        .route("/api/security/license/status", get(routes::get_license_status_handler))
        .route("/api/security/license/activate", post(routes::activate_license_handler))
        .route("/api/security/license/sync-online", post(routes::sync_license_online_handler))

        // Real-time SSE Streams
        .route("/api/signals/stream", get(routes::signals_sse_stream))
        .route("/api/profile-events/stream", get(routes::profile_events_sse_stream))
        .route("/api/profiles/stream", get(routes::profiles_sse_stream))

        // Cooldown Presets
        .route("/api/cooldown-presets", get(routes::get_cooldown_presets_handler))

        // Modular Nodes & Profiles
        .route("/api/nodes", get(routes::get_nodes))
        .route("/api/profile-data", get(routes::get_profile_data))
        .route("/api/profile/activate", post(routes::activate_profile))
        .route("/api/profile/set-active-list", post(routes::set_active_list))
        .route("/api/profile/toggle-active", post(routes::toggle_active_profile))
        .route("/api/profile/new", post(routes::new_profile))
        .route("/api/profile/delete", post(routes::delete_profile))
        .route("/api/profile/rename", post(routes::rename_profile))
        .route("/api/profile/overwrite", post(routes::overwrite_profile))

        // Client Management
        .route("/api/client/launch", post(routes::launch_client))
        .route("/api/client/close", post(routes::close_client))
        .route("/api/client/toggle-enable", post(routes::toggle_client_enable))
        .route("/api/client/reset-bounds", post(routes::reset_client_bounds))
        .route("/api/client/clear-profile", post(routes::clear_client_profile))
        .route("/api/client/alias", post(routes::set_client_alias))
        .route("/api/client/{id}/bounds", post(routes::update_client_bounds))
        .route("/api/client-frame/{id}", get(routes::get_client_frame))

        // Community Hub
        .route(
            "/api/community/identity",
            get(routes::get_creator_identity_handler).post(routes::update_creator_identity_handler),
        )
        .route("/api/community/install", post(routes::install_community_profile))
        .route("/api/community/profile-for-share", get(routes::get_profile_for_share))

        // Engine Control
        .route("/api/engine/start", post(routes::start_engine))
        .route("/api/engine/stop", post(routes::stop_engine))
        .route("/api/suspend/toggle", post(routes::toggle_suspend))
        .route("/api/overlay/disable", post(routes::disable_overlay))
        .route("/api/trigger/{event_name}", get(routes::trigger_event).post(routes::trigger_event))

        // Sound, Storage & Webhook
        .route("/api/sound/upload", post(routes::upload_sound))
        .route("/api/open-folder", post(routes::open_folder))
        .route("/api/webhook/test", post(routes::test_webhook))

        // Native Window Control
        .route("/api/window/minimize", post(routes::window_minimize))
        .route("/api/window/maximize", post(routes::window_maximize))
        .route("/api/window/hide", post(routes::window_hide))
        .route("/api/window/show", post(routes::window_show))
        .route("/api/window/close", post(routes::window_close))
        .route("/api/window/overlay/show", post(routes::overlay_show))
        .route("/api/window/overlay/hide", post(routes::overlay_hide))
        .route("/api/window/overlay/toggle", post(routes::overlay_toggle))
        .route("/api/window/overlay/resize", post(routes::overlay_resize))

        // Fallback for static assets & 404 handler
        .fallback(routes::static_fallback)
        .layer(cors)
        .with_state(state)
}

pub async fn start_web_server(state: AppState, port: u16) -> Result<(), String> {
    let app = create_router(state.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    println!("🌐 [Web Server] Embedded Axum Server listening on http://127.0.0.1:{}", port);
    println!("   └─ Web Studio:   http://127.0.0.1:{}/", port);
    println!("   └─ Launcher UI:  http://127.0.0.1:{}/launcher", port);

    // ⚡ Real-time License Sentinel (Pusher Channels WSS - 0.2s sub-second reaction)
    crate::security::start_realtime_license_sentinel(state.clone()).await;

    // 🔐 Autonomous License Heartbeat & Fallback Safety Net (Every 10 min)
    let heartbeat_state = state.clone();
    tokio::spawn(async move {
        // Initial warmup delay 200ms on boot
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        loop {
            let current_hwid = crate::security::generate_hwid();
            let default_worker_url = crate::secure_str!("https://nodehotkey-api.kitsada19972540.workers.dev");

            let has_local_key = crate::security::license::load_license_file(Some(&heartbeat_state.config_store.base_dir)).is_some()
                || crate::security::license::load_serial_file(Some(&heartbeat_state.config_store.base_dir)).is_some();

            if has_local_key {
                let _ = crate::security::sync_license_from_server(
                    &default_worker_url,
                    &current_hwid,
                    Some(&heartbeat_state.config_store.base_dir),
                ).await;
            }

            let status = crate::security::check_license_status(Some(&heartbeat_state.config_store.base_dir));

            if !status.is_valid {
                if heartbeat_state.is_running.load(std::sync::atomic::Ordering::SeqCst) {
                    heartbeat_state.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
                    heartbeat_state.log("error", &format!("🔒 Engine halted: License is {}", status.status));
                }
            }

            // Real-time broadcast updated license/engine status to connected UI clients
            heartbeat_state.notify_status_change().await;

            // Periodic Heartbeat check every 10 minutes (600 seconds)
            tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
        }
    });

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("⚠️ [Web Server] Server error: {}", e);
        }
    });

    Ok(())
}
