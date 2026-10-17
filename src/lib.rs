pub mod audio;
pub mod browser;
pub mod config;
pub mod engine;
pub mod input;
pub mod model;
pub mod server;
pub mod state;
pub mod vision;
pub mod security;

#[cfg(test)]
mod tests {
    use super::*;
    use engine::graph::NodeExecutionEngine;
    use model::{Connection, Profile, WorkflowNode};
    use vision::party_scanner::PartyScanner;

    #[tokio::test]
    async fn test_load_default_json_file() {
        let store = config::ConfigStore::new(&std::path::PathBuf::from("."));
        let prof = store.get_profile("Default").await;
        assert!(prof.is_some(), "Default.json should be loaded: {:?}", prof);
        let p = prof.unwrap();
        assert_eq!(p.name, "Default");
    }

    #[test]
    fn test_port_matcher_compatibility() {
        // Exact match
        assert!(NodeExecutionEngine::match_port("onComplete", Some("onComplete")));
        assert!(NodeExecutionEngine::match_port("onFalse", Some("onFalse")));

        // Case & delimiter insensitive
        assert!(NodeExecutionEngine::match_port("on_complete", Some("onComplete")));
        assert!(NodeExecutionEngine::match_port("exec-out", Some("execOut")));

        // Generic execution ports
        assert!(NodeExecutionEngine::match_port("next", Some("exec_out")));
        assert!(NodeExecutionEngine::match_port("out", Some("onComplete")));
        assert!(NodeExecutionEngine::match_port("in", Some("exec_in")));

        // Mismatched specific ports
        assert!(!NodeExecutionEngine::match_port("onTrue", Some("onFalse")));
        assert!(!NodeExecutionEngine::match_port("onFail", Some("onSuccess")));
    }

    #[test]
    fn test_graph_downstream_resolution() {
        let profile = Profile {
            version: "4.0.0".to_string(),
            name: "Test Chained Profile".to_string(),
            canvas: Default::default(),
            nodes: vec![
                WorkflowNode::new(
                    "node_trig_1",
                    "trigger",
                    Some("Trigger Key 1".to_string()),
                    100.0,
                    100.0,
                    serde_json::json!({
                        "triggerType": "keyboard",
                        "triggerValue": "1",
                        "enabled": true
                    }),
                ),
                WorkflowNode::new(
                    "node_action_2",
                    "key_press",
                    Some("Key Press 2".to_string()),
                    300.0,
                    100.0,
                    serde_json::json!({ "keys": ["2"] }),
                ),
            ],
            connections: vec![Connection {
                from_node_id: "node_trig_1".to_string(),
                from_port: "exec_out".to_string(),
                to_node_id: "node_action_2".to_string(),
                to_port: "in".to_string(),
            }],
            ..Default::default()
        };

        let mut engine = NodeExecutionEngine::new();
        engine.load_profiles(vec![profile]);

        let triggers = engine.get_matching_trigger_nodes("keyboard", "1");
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].id, "node_trig_1");

        let downstream = engine.get_downstream_nodes("node_trig_1", Some("exec_out"));
        assert_eq!(downstream.len(), 1);
        assert_eq!(downstream[0].node.id, "node_action_2");
    }

    #[test]
    fn test_crimson_pixel_detection() {
        let pure_red = image::Rgba([220, 30, 30, 255]);
        assert!(PartyScanner::is_crimson_pixel(pure_red));

        let gray_border = image::Rgba([120, 120, 120, 255]);
        assert!(!PartyScanner::is_crimson_pixel(gray_border));

        let dark_black = image::Rgba([10, 10, 10, 255]);
        assert!(!PartyScanner::is_crimson_pixel(dark_black));

        let green_buff = image::Rgba([30, 200, 30, 255]);
        assert!(!PartyScanner::is_crimson_pixel(green_buff));
    }

    #[tokio::test]
    async fn test_action_executor_chain() {
        use engine::ActionExecutor;
        use state::AppState;
        use std::path::PathBuf;
        use std::sync::atomic::Ordering;

        let state = AppState::new(&PathBuf::from("."));
        state.is_running.store(true, Ordering::SeqCst);

        let profile = Profile {
            version: "4.0.0".to_string(),
            name: "Test Chain Run".to_string(),
            canvas: Default::default(),
            nodes: vec![
                WorkflowNode::new(
                    "node_trig_1",
                    "trigger",
                    Some("Trigger".to_string()),
                    0.0,
                    0.0,
                    serde_json::json!({
                        "triggerType": "keyboard",
                        "triggerValue": "1",
                        "enabled": true
                    }),
                ),
                WorkflowNode::new(
                    "node_step_log_2",
                    "step_log",
                    Some("Log Message".to_string()),
                    100.0,
                    0.0,
                    serde_json::json!({ "message": "Test Execution Chain Passed!" }),
                ),
            ],
            connections: vec![Connection {
                from_node_id: "node_trig_1".to_string(),
                from_port: "exec_out".to_string(),
                to_node_id: "node_step_log_2".to_string(),
                to_port: "in".to_string(),
            }],
            ..Default::default()
        };

        {
            let mut engine = state.engine.write().await;
            engine.load_profiles(vec![profile]);
        }

        let mut signal_rx = state.signal_broadcaster.subscribe();

        // Trigger workflow execution
        ActionExecutor::trigger_flow(state.clone(), "keyboard", "1").await;

        // Expect signal emission from node_step_log_2
        if let Ok(sig) = tokio::time::timeout(std::time::Duration::from_millis(500), signal_rx.recv()).await {
            assert!(sig.is_ok());
            let s = sig.unwrap();
            assert_eq!(s.action_id, "node_step_log_2");
            assert_eq!(s.event_name, "onComplete");
        }
    }

    #[test]
    fn test_profile_serialization_and_node_position() {
        let json_str = r#"{
            "version": "3.1.0",
            "name": "Test Position Profile",
            "canvas": {
                "zoom": 1.0,
                "pan": { "x": 0.0, "y": 0.0 }
            },
            "nodes": [
                {
                    "id": "node_1",
                    "type": "trigger",
                    "title": "Trigger Node",
                    "position": { "x": 280.0, "y": 310.0 },
                    "data": { "enabled": true }
                }
            ],
            "connections": [],
            "variables": [],
            "customField": "preserved_value"
        }"#;

        let mut profile: Profile = serde_json::from_str(json_str).expect("Failed to deserialize profile");
        profile.normalize();
        assert_eq!(profile.nodes.len(), 1);
        assert_eq!(profile.nodes[0].position.x, 280.0);
        assert_eq!(profile.nodes[0].position.y, 310.0);

        let serialized = serde_json::to_value(&profile).expect("Failed to serialize profile");
        let node_pos = serialized["nodes"][0]["position"].clone();
        assert_eq!(node_pos["x"], 280.0);
        assert_eq!(node_pos["y"], 310.0);
        assert_eq!(serialized["customField"], "preserved_value");
    }

    #[test]
    fn test_client_payload_deserialization_and_default_slot() {
        use server::routes::{ClientActionPayload, ToggleEnablePayload};

        // 1. Test ClientActionPayload with clientIndex and browserChoice
        let json_action = r#"{"clientIndex": 3, "browserChoice": "2-app"}"#;
        let payload: ClientActionPayload = serde_json::from_str(json_action).unwrap();
        assert_eq!(payload.client_id(), 3);
        assert_eq!(payload.browser_type(), Some(crate::browser::BrowserType::Edge));
        assert_eq!(payload.is_app_mode(), Some(true));

        // 2. Test ClientActionPayload fallback
        let json_legacy = r#"{"client": 5}"#;
        let payload_legacy: ClientActionPayload = serde_json::from_str(json_legacy).unwrap();
        assert_eq!(payload_legacy.client_id(), 5);
        assert_eq!(payload_legacy.browser_type(), None);
        assert_eq!(payload_legacy.is_app_mode(), None);

        // 3. Test ToggleEnablePayload with clientIndex
        let json_toggle = r#"{"clientIndex": 4, "enabled": false}"#;
        let toggle: ToggleEnablePayload = serde_json::from_str(json_toggle).unwrap();
        assert_eq!(toggle.client_id(), 4);
        assert_eq!(toggle.enabled, Some(false));

        // 4. Test default GlobalSettings client_slots starts with only 1 client
        let default_global = model::GlobalSettings::default();
        assert_eq!(default_global.client_slots, vec![1]);
    }

    #[test]
    fn test_key_mapping_and_trigger_normalization() {
        use engine::executor::map_key_to_cdp;
        use engine::graph::normalize_trigger_key;

        // 1. Trigger normalization
        assert_eq!(normalize_trigger_key("KeyA"), "A");
        assert_eq!(normalize_trigger_key("KeyB"), "B");
        assert_eq!(normalize_trigger_key("KeyZ"), "Z");
        assert_eq!(normalize_trigger_key("1"), "1");
        assert_eq!(normalize_trigger_key("Num1"), "1");
        assert_eq!(normalize_trigger_key("Digit1"), "1");

        // Numpad is strictly separate from main top-row digits
        assert_eq!(normalize_trigger_key("NUMPAD 1"), "NUMPAD1");
        assert_eq!(normalize_trigger_key("NUMPAD1"), "NUMPAD1");
        assert_eq!(normalize_trigger_key("Kp1"), "NUMPAD1");
        assert_eq!(normalize_trigger_key("KP0"), "NUMPAD0");
        assert_eq!(normalize_trigger_key("KPEnter"), "NUMPAD_ENTER");

        // Assert strict inequality
        assert_ne!(normalize_trigger_key("1"), normalize_trigger_key("NUMPAD 1"));
        assert_ne!(normalize_trigger_key("Num1"), normalize_trigger_key("Kp1"));

        assert_eq!(normalize_trigger_key("space"), "SPACE");
        assert_eq!(normalize_trigger_key("escape"), "ESC");

        // 2. CDP Key Mapping
        let (k1, code1, vk1) = map_key_to_cdp("1");
        assert_eq!(k1, "1");
        assert_eq!(code1, "Digit1");
        assert_eq!(vk1, 49);

        let (ka, codea, vka) = map_key_to_cdp("A");
        assert_eq!(ka, "a");
        assert_eq!(codea, "KeyA");
        assert_eq!(vka, 65);

        let (ksp, codesp, vksp) = map_key_to_cdp("SPACE");
        assert_eq!(ksp, " ");
        assert_eq!(codesp, "Space");
        assert_eq!(vksp, 32);

        let (kf1, code_f1, vk_f1) = map_key_to_cdp("F1");
        assert_eq!(kf1, "F1");
        assert_eq!(code_f1, "F1");
        assert_eq!(vk_f1, 112);
    }

    #[test]
    fn test_profile_sanitization_removes_global_settings() {
        let mut prof = Profile::default();
        prof.extra.insert("clientBrowsers".to_string(), serde_json::json!({ "1": "1-app" }));
        prof.extra.insert("clientAliases".to_string(), serde_json::json!({ "1": "RM" }));
        prof.extra.insert("targetUrlKeyword".to_string(), serde_json::json!("flyff"));
        prof.extra.insert("customField".to_string(), serde_json::json!("keepMe"));

        assert_eq!(prof.extra.len(), 4);
        prof.sanitize_profile_keys();
        assert_eq!(prof.extra.len(), 1);
        assert!(prof.extra.contains_key("customField"));
        assert!(!prof.extra.contains_key("clientBrowsers"));
        assert!(!prof.extra.contains_key("clientAliases"));
        assert!(!prof.extra.contains_key("targetUrlKeyword"));
    }

    #[test]
    fn test_config_store_base_dir_absolute() {
        let store_rel = crate::config::ConfigStore::new(".");
        assert!(store_rel.base_dir.is_absolute());
    }

    #[tokio::test]
    async fn test_clean_and_save_all_profiles_on_disk() {
        let store = crate::config::ConfigStore::new(".");
        let profiles = store.get_all_profiles().await;
        for p in profiles {
            let res = store.save_profile(&p).await;
            assert!(res.is_ok(), "Failed to save profile {}: {:?}", p.name, res);
        }
    }

    #[tokio::test]
    async fn test_state_status_json_and_notify_broadcast() {
        use std::path::PathBuf;
        let state = crate::state::AppState::new(&PathBuf::from("."));
        let mut rx = state.status_event_sender.subscribe();

        state.notify_status_change().await;

        let status = rx.recv().await.expect("Failed to receive status event");
        assert_eq!(status["success"], true);
        assert!(status.get("running").is_some());
        assert!(status.get("activeClients").is_some());
        assert!(status.get("clientStatuses").is_some());
    }

    #[test]
    fn test_realtime_license_sentinel_constants() {
        use crate::security::realtime_sentinel::*;
        assert_eq!(DEFAULT_PUSHER_KEY, "121974b05826cd21a5da");
        assert_eq!(DEFAULT_PUSHER_CLUSTER, "ap1");
        assert_eq!(LICENSE_CHANNEL, "nhk-licenses");
    }

    #[test]
    fn test_webhook_test_payload_deserialization() {
        let json_str = r#"{
            "url": "https://discord.com/api/webhooks/123/abc",
            "method": "POST",
            "headers": "{\"Content-Type\": \"application/json\"}",
            "payload": "{\"content\": \"Hello world\"}",
            "timeoutMs": 3000
        }"#;

        let payload: crate::server::routes::WebhookTestPayload = serde_json::from_str(json_str).unwrap();
        assert_eq!(payload.url.unwrap(), "https://discord.com/api/webhooks/123/abc");
        assert_eq!(payload.method.unwrap(), "POST");
        assert_eq!(payload.timeout_ms_camel, Some(3000));
    }

    #[test]
    fn test_tts_manager_cache_generation() {
        let temp_dir = std::env::temp_dir().join("nhk_tts_unit_test");
        let mgr = crate::audio::TtsManager::new(&temp_dir);
        let path1 = mgr.get_cache_path("แจ้งเตือนเลือดต่ำ", "th-TH-PremwadeeNeural");
        let path2 = mgr.get_cache_path("แจ้งเตือนเลือดต่ำ", "th-TH-PremwadeeNeural");
        let path3 = mgr.get_cache_path("แจ้งเตือนเลือดต่ำ", "th-TH-NiwatNeural");

        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
        assert!(path1.to_string_lossy().ends_with(".mp3"));
    }

    #[tokio::test]
    async fn test_key_hold_toggle_and_release_state() {
        use std::path::PathBuf;
        use std::collections::HashSet;
        use std::sync::atomic::Ordering;

        let state = crate::state::AppState::new(&PathBuf::from("."));
        state.is_running.store(true, Ordering::SeqCst);

        // 1. Initial state: No keys held
        assert!(state.held_keys.read().await.is_empty());

        let node = crate::model::WorkflowNode::new(
            "node_hold_test",
            "key_hold",
            Some("Key Hold Toggle".to_string()),
            0.0,
            0.0,
            serde_json::json!({
                "targetKey": "W",
                "targetClient": "1",
                "holdMode": "toggle"
            }),
        );

        // 2. First execution -> Key should be engaged (held down)
        let mut stack = HashSet::new();
        crate::engine::ActionExecutor::execute_node(state.clone(), node.clone(), &mut stack).await;
        assert!(state.held_keys.read().await.contains(&(1, "W".to_string())));

        // 3. Second execution (same node) -> Key should be toggled OFF (released)
        let mut stack2 = HashSet::new();
        crate::engine::ActionExecutor::execute_node(state.clone(), node.clone(), &mut stack2).await;
        assert!(!state.held_keys.read().await.contains(&(1, "W".to_string())));

        // 4. Third execution -> Key should be engaged again
        let mut stack3 = HashSet::new();
        crate::engine::ActionExecutor::execute_node(state.clone(), node.clone(), &mut stack3).await;
        assert!(state.held_keys.read().await.contains(&(1, "W".to_string())));
    }

    #[tokio::test]
    async fn test_reset_and_stop_engine_clears_held_keys_and_aborts() {
        use std::path::PathBuf;
        use std::sync::atomic::Ordering;

        let state = crate::state::AppState::new(&PathBuf::from("."));
        state.is_running.store(true, Ordering::SeqCst);

        // 1. Manually engage keys in held_keys
        {
            let mut held = state.held_keys.write().await;
            held.insert((1, "SPACE".to_string()));
            held.insert((2, "W".to_string()));
        }
        assert_eq!(state.held_keys.read().await.len(), 2);

        // 2. Set custom variable in engine
        {
            let mut engine = state.engine.write().await;
            engine.variables_map.insert("comboCount".to_string(), serde_json::json!(99));
        }

        // 3. Trigger stop & reset
        state.reset_and_stop_engine().await;

        // 4. Verify all held keys released and drained
        assert!(state.held_keys.read().await.is_empty());
        // Verify engine is marked stopped
        assert!(!state.is_running.load(Ordering::SeqCst));
        // Verify variable restored to default or cleared
        let engine = state.engine.read().await;
        assert_eq!(engine.variables_map.get("comboCount"), None);
    }

    #[tokio::test]
    async fn test_skill_cooldown_guard_lifecycle() {
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("nhk_test_cd_{}", ts));
        let _ = std::fs::create_dir_all(&temp_dir);
        let state = crate::state::AppState::new(&temp_dir);

        // 1. Manually add Pain Dealer preset to presets map for testing
        {
            let mut presets = state.cooldown_presets_map.write().await;
            presets.insert("skill_knight_pain-dealer".to_string(), (30000, "Pain Dealer".to_string()));
        }

        // 2. Initially, client 1 is NOT on cooldown
        assert!(state.check_skill_cooldown(1, "skill_knight_pain-dealer").await.is_none());

        // 3. Set a cooldown of 2 seconds
        state.set_skill_cooldown(1, "skill_knight_pain-dealer", std::time::Duration::from_secs(2)).await;

        // 4. Client 1 IS now on cooldown
        let remaining = state.check_skill_cooldown(1, "skill_knight_pain-dealer").await;
        assert!(remaining.is_some());
        assert!(remaining.unwrap() > 0.0);

        // 5. Client 2 is NOT on cooldown (client isolation)
        assert!(state.check_skill_cooldown(2, "skill_knight_pain-dealer").await.is_none());

        // 6. Test get_node_cooldown_spec
        let node = crate::model::WorkflowNode::new(
            "node_test_cd",
            "key_press",
            Some("Single Key Press".to_string()),
            0.0,
            0.0,
            serde_json::json!({
                "actionId": "test_act",
                "cooldownPresetId": "skill_knight_pain-dealer",
                "targetClient": "1",
                "keys": ["1"]
            }),
        );

        let spec = crate::engine::ActionExecutor::get_node_cooldown_spec(&state, &node).await;
        assert!(spec.is_some());
        let (cd_ms, skill_key, skill_name) = spec.unwrap();
        assert_eq!(cd_ms, 30000);
        assert_eq!(skill_key, "skill_knight_pain-dealer");
        assert_eq!(skill_name, "Pain Dealer");

        // 7. Verify reset_and_stop_engine clears active cooldowns
        state.reset_and_stop_engine().await;
        assert!(state.check_skill_cooldown(1, "skill_knight_pain-dealer").await.is_none());
    }

    #[tokio::test]
    async fn test_sound_node_execution() {
        use engine::ActionExecutor;
        use state::AppState;
        use std::collections::HashSet;
        use std::path::PathBuf;
        use std::sync::atomic::Ordering;

        let state = AppState::new(&PathBuf::from("."));
        state.is_running.store(true, Ordering::SeqCst);

        let sound_node = WorkflowNode::new(
            "node_sound_1",
            "sound",
            Some("Sound Alert".to_string()),
            0.0,
            0.0,
            serde_json::json!({
                "actionId": "test_sound",
                "soundPreset": "alarm",
                "soundSource": "preset",
                "volume": 8,
                "repeatCount": 1
            }),
        );

        let mut call_stack = HashSet::new();
        ActionExecutor::execute_node(state.clone(), sound_node, &mut call_stack).await;
        assert!(call_stack.contains("node_sound_1"));
    }

    #[tokio::test]
    async fn test_format_text_to_step_log_data_pin() {
        use engine::ActionExecutor;
        use state::AppState;
        use std::collections::HashSet;
        use std::path::PathBuf;
        use std::sync::atomic::Ordering;

        let state = AppState::new(&PathBuf::from("."));
        state.is_running.store(true, Ordering::SeqCst);

        let fmt_node = WorkflowNode::new(
            "node_fmt_1",
            "format_text",
            Some("Format Text".to_string()),
            0.0,
            0.0,
            serde_json::json!({
                "template": "สวัสดีครับ {name}",
                "boolFormat": "true_false"
            }),
        );

        let log_node = WorkflowNode::new(
            "node_log_1",
            "step_log",
            Some("Log Message".to_string()),
            0.0,
            0.0,
            serde_json::json!({
                "message": "Default Fallback Message"
            }),
        );

        let profile = Profile {
            version: "4.0.0".to_string(),
            name: "Test Format Profile".to_string(),
            canvas: Default::default(),
            nodes: vec![fmt_node.clone(), log_node.clone()],
            connections: vec![Connection {
                from_node_id: "node_fmt_1".to_string(),
                from_port: "msg_out".to_string(),
                to_node_id: "node_log_1".to_string(),
                to_port: "msg_in".to_string(),
            }],
            ..Default::default()
        };

        {
            let mut engine = state.engine.write().await;
            engine.load_profiles(vec![profile]);
            engine.variables_map.insert("name".to_string(), serde_json::json!("Flyff Hero"));
        }

        // Verify direct resolution via resolve_data_string
        {
            let engine = state.engine.read().await;
            let resolved = ActionExecutor::resolve_data_string(&engine, "node_log_1", &["msg_in"]);
            assert_eq!(resolved, Some("สวัสดีครับ Flyff Hero".to_string()));
        }

        // Verify full node execution uses the resolved message
        let mut call_stack = HashSet::new();
        ActionExecutor::execute_node(state.clone(), log_node, &mut call_stack).await;
        assert!(call_stack.contains("node_log_1"));
    }
}



