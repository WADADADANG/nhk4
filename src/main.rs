use nodehotkey_rust::browser::ClientSlot;
use nodehotkey_rust::input::HotkeyAction;
use nodehotkey_rust::server::start_web_server;
use nodehotkey_rust::state::AppState;
use sha2::Digest;
use std::env;
use std::path::PathBuf;

fn hex_dump(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================================");
    println!("🦀 NodeHotkey v4.0.0 - Rust Native Core Engine");
    println!("   Multi-Client Background WebGL Automation Suite");
    println!("   Cross-Platform: Windows, Linux, macOS");
    println!("========================================================\n");

    let args: Vec<String> = env::args().collect();

    // CLI Security & License Tools
    if args.iter().any(|a| a == "--hwid") {
        let hwid = nodehotkey_rust::security::generate_hwid();
        println!("🔐 Machine HWID: {}", hwid);
        return Ok(());
    }

    if args.iter().any(|a| a == "--keys-info") {
        let signing_key = nodehotkey_rust::security::get_master_signing_key();
        let verifying_key = nodehotkey_rust::security::get_master_verifying_key();
        println!("========================================================");
        println!("🔐 Ed25519 Cryptographic Master Keys:");
        println!("   Public Key (Embed in .exe / Client Verification):");
        println!("   {:X}", sha2::Sha256::digest(verifying_key.as_bytes()));
        println!("   Raw Verifying Bytes (Hex):");
        println!("   {}", hex_dump(verifying_key.as_bytes()));
        println!("\n   Private Signing Key (Put in Cloudflare Worker Secret):");
        println!("   {}", hex_dump(&signing_key.to_bytes()));
        println!("========================================================");
        return Ok(());
    }

    if let Some(pos) = args.iter().position(|a| a == "--gen-key") {
        let target_hwid = args.get(pos + 1).cloned().unwrap_or_else(nodehotkey_rust::security::generate_hwid);
        let claims = nodehotkey_rust::security::LicenseClaims {
            hwid: target_hwid.clone(),
            tier: "PRO".to_string(),
            expires_at: -1, // Lifetime
            issued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            user_id: Some("ADMIN_GENERATED".to_string()),
        };
        let signing_key = nodehotkey_rust::security::get_master_signing_key();
        let key = nodehotkey_rust::security::generate_license_key_ed25519(&claims, &signing_key);
        println!("🔑 Generated Ed25519 License Key for HWID [{}]:\n{}", target_hwid, key);
        return Ok(());
    }

    if args.iter().any(|a| a == "--gen-key-universal") {
        let claims = nodehotkey_rust::security::LicenseClaims {
            hwid: "UNIVERSAL".to_string(),
            tier: "LIFETIME_DEV".to_string(),
            expires_at: -1, // Lifetime
            issued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            user_id: Some("MASTER_DEVELOPER".to_string()),
        };
        let signing_key = nodehotkey_rust::security::get_master_signing_key();
        let key = nodehotkey_rust::security::generate_license_key_ed25519(&claims, &signing_key);
        println!("🔑 Generated Universal Ed25519 Developer Key (Works on any machine):\n{}", key);
        return Ok(());
    }

    if let Some(pos) = args.iter().position(|a| a == "--gen-serial") {
        let count: usize = args.get(pos + 1).and_then(|s| s.parse().ok()).unwrap_or(5);
        let tier = args.get(pos + 2).cloned().unwrap_or_else(|| "PRO".to_string());
        let duration_days: i64 = args.get(pos + 3).and_then(|s| s.parse().ok()).unwrap_or(-1);

        println!("========================================================");
        println!("✨ Generating {} Serial Keys (Tier: {}, Duration: {})", count, tier, if duration_days == -1 { "Lifetime".to_string() } else { format!("{} days", duration_days) });
        println!("========================================================");

        let mut sql_values = Vec::new();
        for i in 1..=count {
            let key = nodehotkey_rust::security::generate_random_serial_key(&tier);
            println!("{}. {}", i, key);
            sql_values.push(format!("('{}', '{}', {}, 'UNUSED', 'Lot generated')", key, tier, duration_days));
        }

        println!("\n📋 SQL Insert command for Cloudflare D1 Console:");
        println!("--------------------------------------------------------");
        println!(
            "INSERT INTO license_keys (key, tier, duration_days, status, user_note) VALUES\n{};",
            sql_values.join(",\n")
        );
        println!("--------------------------------------------------------");
        return Ok(());
    }

    let is_headless = args
        .iter()
        .any(|a| a == "--headless" || a == "--server" || a == "--no-gui")
        || (env::var("DISPLAY").is_err()
            && env::var("WAYLAND_DISPLAY").is_err()
            && cfg!(target_os = "linux"));

    let rt = tokio::runtime::Runtime::new()?;

    // 1. Locate project root directory
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let base_path = if current_dir.join("configs").exists() {
        current_dir
    } else if current_dir.join("../configs").exists() {
        current_dir.parent().map(|p| p.to_path_buf()).unwrap_or(current_dir)
    } else {
        current_dir
    };

    println!(
        "📂 Project Root Directory: {}",
        base_path.canonicalize().unwrap_or(base_path.clone()).display()
    );

    // 2. Initialize AppState
    let state = AppState::new(&base_path);

    // Run async setup in Tokio runtime
    let port = rt.block_on(async {
        let global_config = state.config_store.get_global_config().await;
        println!("⚙️  Active Profile: \"{}\"", global_config.active_profile);
        println!("🎮 Client Slots: {:?}", global_config.global_settings.client_slots);
        println!("🛑 Suspend Hotkey: \"{}\"", global_config.global_settings.suspend_hotkey);

        // Pre-register all configured client slots
        for &slot_id in &global_config.global_settings.client_slots {
            let alias = global_config
                .global_settings
                .client_aliases
                .get(&slot_id.to_string())
                .cloned();
            let proxy = global_config
                .global_settings
                .client_proxies
                .get(&slot_id.to_string())
                .cloned();
            let user_agent = global_config
                .global_settings
                .client_user_agents
                .get(&slot_id.to_string())
                .cloned();

            state
                .client_manager
                .register_client(ClientSlot {
                    id: slot_id,
                    is_running: false,
                    is_paused: false,
                    alias,
                    proxy,
                    user_agent,
                    debug_port: None,
                    pid: None,
                })
                .await;
        }

        let profiles = state.config_store.get_all_profiles().await;
        println!("📋 Loaded {} profiles from disk.", profiles.len());

        {
            let mut engine = state.engine.write().await;
            engine.load_profiles(profiles);
            println!(
                "🧩 Execution Engine initialized with {} active nodes.",
                engine.nodes_map.len()
            );
        }

        // Set engine running state to true so hotkeys are active immediately
        state.is_running.store(true, std::sync::atomic::Ordering::SeqCst);

        // Start Global Hotkey Listener and wire to workflow execution
        let (hotkey_tx, mut hotkey_rx) = tokio::sync::broadcast::channel(256);
        let suspend_key = global_config.global_settings.suspend_hotkey.clone();
        let _ = state.hotkey_listener.start_listener(suspend_key, hotkey_tx);

        let state_hotkey = state.clone();
        tokio::spawn(async move {
            while let Ok(action) = hotkey_rx.recv().await {
                match action {
                    HotkeyAction::ToggleSuspend => {
                        let is_suspended = state_hotkey.hotkey_listener.is_suspended();
                        state_hotkey.signal_broadcaster.emit(
                            "sentinel",
                            if is_suspended { "suspended" } else { "active" },
                            None,
                        );
                        state_hotkey.log(
                            "system",
                            &format!(
                                "Hotkey Sentinel state toggled: {}",
                                if is_suspended { "SUSPENDED" } else { "ACTIVE" }
                            ),
                        );
                        state_hotkey.notify_status_change().await;
                    }
                    HotkeyAction::TriggerKey(key) => {
                        state_hotkey.handle_key_trigger(&key).await;
                    }
                }
            }
        });

        state.signal_broadcaster.emit("system_init", "ready", None);
        println!("⚡ Realtime Telemetry & Signal Broadcaster online.");

        if state.audio_player.is_some() {
            println!("🔊 Native Audio Player initialized (WASAPI/CoreAudio/ALSA).");
        }

        println!(
            "⌨️  Global Hotkey Sentinel ready on key: \"{}\"",
            global_config.global_settings.suspend_hotkey
        );

        let p = global_config.global_settings.web_port;
        let _ = start_web_server(state.clone(), p).await;
        state.log("info", "NodeHotkey Rust Core Engine & Web Server online.");

        p
    });

    println!("\n🚀 NodeHotkey Rust Core is running! Access the UI via:");
    println!("   🏠 Launcher Dashboard: http://127.0.0.1:{}/launcher", port);
    println!("   🎮 Action Node Studio: http://127.0.0.1:{}/", port);

    if is_headless {
        println!("\n🖥️ Running in headless / server mode. Press Ctrl+C to exit.");
        let state_headless = state.clone();
        rt.block_on(async {
            let _ = tokio::signal::ctrl_c().await;
            println!("\n🛑 Closing all active browser screens...");
            state_headless.terminate_all_clients().await;
        });
        println!("\n🛑 Shutting down NodeHotkey Rust Core cleanly.");
    } else {
        println!("\n🖥️ Initializing Tauri Native GUI Desktop Window with System Tray...");
        use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
        use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
        use tauri::Manager;

        let state_tauri = state.clone();
        tauri::Builder::default()
            .setup(move |app| {
                let state_broker = state_tauri.clone();
                let state_tray = state_tauri.clone();

                // Window Action Broker (handles minimize, maximize, hide, show, close via HTTP API)
                let mut window_rx = state_tauri.window_action_sender.subscribe();
                let app_handle_broker = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    use nodehotkey_rust::state::WindowAction;
                    while let Ok(action) = window_rx.recv().await {
                        match action {
                            WindowAction::Minimize => {
                                if let Some(w) = app_handle_broker.get_webview_window("main") {
                                    let _ = w.minimize();
                                }
                            }
                            WindowAction::ToggleMaximize => {
                                if let Some(w) = app_handle_broker.get_webview_window("main") {
                                    if w.is_maximized().unwrap_or(false) {
                                        let _ = w.unminimize();
                                    } else {
                                        let _ = w.maximize();
                                    }
                                }
                            }
                            WindowAction::Hide => {
                                if let Some(w) = app_handle_broker.get_webview_window("main") {
                                    let _ = w.hide();
                                }
                            }
                            WindowAction::Show => {
                                if let Some(w) = app_handle_broker.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.unminimize();
                                    let _ = w.set_focus();
                                }
                            }
                            WindowAction::Close => {
                                let state_term = state_broker.clone();
                                let app_exit = app_handle_broker.clone();
                                tauri::async_runtime::spawn(async move {
                                    println!("\n🛑 Window Close requested: Terminating all browser screens...");
                                    state_term.terminate_all_clients().await;
                                    app_exit.exit(0);
                                });
                            }
                            WindowAction::OverlayShow => {
                                if let Some(ow) = app_handle_broker.get_webview_window("overlay") {
                                    let _ = ow.show();
                                    let _ = ow.set_always_on_top(true);
                                }
                            }
                            WindowAction::OverlayHide => {
                                if let Some(ow) = app_handle_broker.get_webview_window("overlay") {
                                    let _ = ow.hide();
                                }
                            }
                            WindowAction::OverlayToggle => {
                                if let Some(ow) = app_handle_broker.get_webview_window("overlay") {
                                    if ow.is_visible().unwrap_or(false) {
                                        let _ = ow.hide();
                                    } else {
                                        let _ = ow.show();
                                        let _ = ow.set_always_on_top(true);
                                    }
                                }
                            }
                            WindowAction::OverlayResize { width, height } => {
                                if let Some(ow) = app_handle_broker.get_webview_window("overlay") {
                                    let _ = ow.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
                                }
                            }
                        }
                    }
                });

                // Window Icon
                let icon = if let Some(i) = app.default_window_icon() {
                    i.clone()
                } else {
                    let img = image::load_from_memory(include_bytes!("../icons/icon.png"))
                        .expect("failed to decode icon.png")
                        .to_rgba8();
                    let (width, height) = img.dimensions();
                    tauri::image::Image::new_owned(img.into_raw(), width, height)
                };

                // Tray Menu Items
                let status_item = MenuItemBuilder::with_id("status", "⚡ NodeHotKey V4 (Active)")
                    .enabled(false)
                    .build(app)?;
                let sep1 = PredefinedMenuItem::separator(app)?;
                let show_item = MenuItemBuilder::with_id("show", "🖥️ Show Launcher").build(app)?;
                let studio_item = MenuItemBuilder::with_id(
                    "studio",
                    "🎮 Action Node Studio",
                )
                .build(app)?;
                let sep2 = PredefinedMenuItem::separator(app)?;
                let exit_item = MenuItemBuilder::with_id("exit", "❌ Exit").build(app)?;

                let menu = MenuBuilder::new(app)
                    .item(&status_item)
                    .item(&sep1)
                    .item(&show_item)
                    .item(&studio_item)
                    .item(&sep2)
                    .item(&exit_item)
                    .build()?;

                let tray_port = port;
                let _tray = TrayIconBuilder::with_id("main-tray")
                    .tooltip("NodeHotKey V4 Control Center")
                    .icon(icon)
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_tray_icon_event(|tray, event| match event {
                        TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let is_visible = window.is_visible().unwrap_or(false);
                                if is_visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.unminimize();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        _ => {}
                    })
                    .on_menu_event(move |app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "studio" => {
                            let studio_url = format!("http://127.0.0.1:{}/", tray_port);
                            #[cfg(target_os = "windows")]
                            {
                                let _ = std::process::Command::new("cmd")
                                    .args(["/C", "start", "", &studio_url])
                                    .spawn();
                            }
                            #[cfg(target_os = "macos")]
                            {
                                let _ = std::process::Command::new("open")
                                    .arg(&studio_url)
                                    .spawn();
                            }
                            #[cfg(target_os = "linux")]
                            {
                                let _ = std::process::Command::new("xdg-open")
                                    .arg(&studio_url)
                                    .spawn();
                            }
                        }
                        "exit" => {
                            println!("\n🛑 Tray Exit selected: Closing all browser screens and shutting down cleanly.");
                            let state_term = state_tray.clone();
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                state_term.terminate_all_clients().await;
                                app_handle.exit(0);
                            });
                        }
                        _ => {}
                    })
                    .build(app)?;

                Ok(())
            })
            .on_window_event(|window, event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }

    Ok(())
}

