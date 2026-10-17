use crate::model::WorkflowNode;
use crate::state::AppState;
use rand::Rng;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::time::Duration;

/// Helper to map common game hotkeys to CDP Key, Code, and Windows VirtualKeyCode
pub fn map_key_to_cdp(raw_key: &str) -> (String, String, u32) {
    let k = raw_key.trim();
    let upper = k.to_uppercase();

    match upper.as_str() {
        "1" => ("1".to_string(), "Digit1".to_string(), 49),
        "2" => ("2".to_string(), "Digit2".to_string(), 50),
        "3" => ("3".to_string(), "Digit3".to_string(), 51),
        "4" => ("4".to_string(), "Digit4".to_string(), 52),
        "5" => ("5".to_string(), "Digit5".to_string(), 53),
        "6" => ("6".to_string(), "Digit6".to_string(), 54),
        "7" => ("7".to_string(), "Digit7".to_string(), 55),
        "8" => ("8".to_string(), "Digit8".to_string(), 56),
        "9" => ("9".to_string(), "Digit9".to_string(), 57),
        "0" => ("0".to_string(), "Digit0".to_string(), 48),
        "-" => ("-".to_string(), "Minus".to_string(), 189),
        "=" => ("=".to_string(), "Equal".to_string(), 187),
        "SPACE" | "SPACEBAR" | " " => (" ".to_string(), "Space".to_string(), 32),
        "ENTER" | "RETURN" => ("Enter".to_string(), "Enter".to_string(), 13),
        "ESC" | "ESCAPE" => ("Escape".to_string(), "Escape".to_string(), 27),
        "TAB" => ("Tab".to_string(), "Tab".to_string(), 9),
        "F1" => ("F1".to_string(), "F1".to_string(), 112),
        "F2" => ("F2".to_string(), "F2".to_string(), 113),
        "F3" => ("F3".to_string(), "F3".to_string(), 114),
        "F4" => ("F4".to_string(), "F4".to_string(), 115),
        "F5" => ("F5".to_string(), "F5".to_string(), 116),
        "F6" => ("F6".to_string(), "F6".to_string(), 117),
        "F7" => ("F7".to_string(), "F7".to_string(), 118),
        "F8" => ("F8".to_string(), "F8".to_string(), 119),
        "F9" => ("F9".to_string(), "F9".to_string(), 120),
        "F10" => ("F10".to_string(), "F10".to_string(), 121),
        "F11" => ("F11".to_string(), "F11".to_string(), 122),
        "F12" => ("F12".to_string(), "F12".to_string(), 123),
        _ => {
            if upper.len() == 1 {
                let ch = upper.chars().next().unwrap();
                if ch.is_ascii_alphabetic() {
                    let code = format!("Key{}", ch);
                    let vk = ch as u32;
                    return (k.to_lowercase(), code, vk);
                }
            }
            (k.to_string(), format!("Key{}", upper), 0)
        }
    }
}

/// Comprehensive Asynchronous Action Executor for all 29 Blueprint Nodes
pub struct ActionExecutor;

impl ActionExecutor {
    /// Resolve target client string ("1", "2", "all", "1,2") into client IDs
    pub async fn resolve_target_clients(state: &AppState, target: &str) -> Vec<u32> {
        let t = target.trim().to_lowercase();
        if t.is_empty() || t == "none" || t == "-" {
            return vec![];
        }
        if t == "all" {
            let lock = state.cdp_sessions.read().await;
            let mut ids: Vec<u32> = lock.keys().cloned().collect();
            ids.sort();
            ids
        } else {
            let parsed: Vec<u32> = t
                .split(',')
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect();
            parsed
        }
    }

    /// Dispatch Key Press to CDP Client
    pub async fn dispatch_cdp_key(state: &AppState, client_id: u32, key: &str, hold_ms: u64) {
        let (cdp_key, cdp_code, vk) = map_key_to_cdp(key);
        let lock = state.cdp_sessions.read().await;
        if let Some(session) = lock.get(&client_id) {
            let _ = session.send_key_press(&cdp_key, &cdp_code, vk, hold_ms).await;
        }
    }

    /// Dispatch Key Hold to CDP Client
    pub async fn dispatch_cdp_key_hold(state: &AppState, client_id: u32, key: &str, is_down: bool) {
        let (cdp_key, cdp_code, vk) = map_key_to_cdp(key);
        let lock = state.cdp_sessions.read().await;
        if let Some(session) = lock.get(&client_id) {
            let _ = session.send_key_hold(&cdp_key, &cdp_code, vk, is_down).await;
        }
    }

    /// Base Interruptible Sleep Helper:
    /// Automatically watches the engine's cancellation token (`stop_sender`)
    /// and hotkey sentinel (`is_suspended()`).
    /// Returns `true` if sleep completed normally.
    /// Returns `false` if interrupted by engine stop or suspend (0ms abort).
    pub async fn sleep_interruptible(state: &AppState, duration: Duration) -> bool {
        if !state.is_running.load(Ordering::SeqCst) || state.hotkey_listener.is_suspended() {
            return false;
        }
        let mut stop_rx = state.stop_sender.subscribe();
        tokio::select! {
            _ = tokio::time::sleep(duration) => {
                state.is_running.load(Ordering::SeqCst) && !state.hotkey_listener.is_suspended()
            }
            _ = stop_rx.recv() => {
                false
            }
        }
    }

    /// Base Interruptible Sleep Helper taking milliseconds directly
    pub async fn sleep_ms(state: &AppState, ms: u64) -> bool {
        Self::sleep_interruptible(state, Duration::from_millis(ms)).await
    }

    /// Base Finish & Propagate Helper:
    /// Emits pulse signal to Canvas UI and propagates execution downstream to connected nodes.
    /// Safely halts if engine has been stopped or suspended.
    pub async fn finish(state: &AppState, node: &WorkflowNode, port: &str, payload: Option<&str>) {
        if !state.is_running.load(Ordering::SeqCst) || state.hotkey_listener.is_suspended() {
            return;
        }
        state.signal_broadcaster.emit(&node.id, port, payload);
        Self::propagate_downstream(state.clone(), &node.id, port).await;
    }

    /// Propagate downstream execution to connected nodes
    pub async fn propagate_downstream(state: AppState, from_node_id: &str, port: &str) {
        if !state.is_running.load(Ordering::SeqCst) || state.hotkey_listener.is_suspended() {
            return;
        }
        let downstream = {
            let engine = state.engine.read().await;
            engine.get_downstream_nodes(from_node_id, Some(port))
        };
        for target in downstream {
            let state_clone = state.clone();
            let target_node = target.node.clone();
            tokio::spawn(async move {
                let mut call_stack = HashSet::new();
                Self::execute_node(state_clone, target_node, &mut call_stack).await;
            });
        }
    }

    /// Trigger workflow execution from a physical hotkey or event
    pub async fn trigger_flow(state: AppState, trigger_type: &str, trigger_value: &str) {
        if !state.is_running.load(Ordering::SeqCst) || state.hotkey_listener.is_suspended() {
            return;
        }

        let triggers = {
            let engine = state.engine.read().await;
            engine.get_matching_trigger_nodes(trigger_type, trigger_value)
        };

        if !triggers.is_empty() {
            let msg = if trigger_type.eq_ignore_ascii_case("event") {
                format!("📡 Event [{}] activated {} matching flow(s)", trigger_value, triggers.len())
            } else if trigger_type.eq_ignore_ascii_case("webhook") {
                format!("🌐 Inbound Webhook [{}] activated {} matching flow(s)", trigger_value, triggers.len())
            } else {
                let norm = crate::engine::graph::normalize_trigger_key(trigger_value);
                format!("⚡ Hotkey [{}] (Raw: '{}') activated {} matching flow(s)", norm, trigger_value, triggers.len())
            };
            println!("{}", msg);
            state.log("action", &msg);
        }

        for trig in triggers {
            let downstream = {
                let engine = state.engine.read().await;
                engine.get_downstream_nodes(&trig.id, Some("exec_out"))
            };

            for target in downstream {
                let state_clone = state.clone();
                let target_node = target.node.clone();
                tokio::spawn(async move {
                    let mut call_stack = HashSet::new();
                    Self::execute_node(state_clone, target_node, &mut call_stack).await;
                });
            }
        }
    }

    /// Evaluates if a node has an active Cooldown Guard configured.
    /// Returns `Some((cooldown_ms, skill_key, skill_name))` if cooldown guard is active (duration > 0),
    /// or `None` if no cooldown guard is configured.
    pub async fn get_node_cooldown_spec(state: &AppState, node: &WorkflowNode) -> Option<(u64, String, String)> {
        let preset_id = node.get_str("cooldownPresetId", "");
        let custom_ms = if let Some(v) = node.data.get("customCooldownMs") {
            if let Some(n) = v.as_u64() {
                n
            } else if let Some(s) = v.as_str() {
                s.parse::<u64>().unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        if preset_id.is_empty() && custom_ms == 0 {
            return None;
        }

        let (preset_ms, preset_name) = {
            let presets = state.cooldown_presets_map.read().await;
            presets.get(&preset_id).cloned().unwrap_or((0, preset_id.clone()))
        };

        let effective_ms = if custom_ms > 0 { custom_ms } else { preset_ms };
        if effective_ms == 0 {
            return None;
        }

        let skill_name = if !preset_name.is_empty() {
            preset_name
        } else if !preset_id.is_empty() {
            preset_id.clone()
        } else {
            "Custom Skill".to_string()
        };

        let skill_key = if !preset_id.is_empty() && preset_id != "custom" {
            preset_id
        } else {
            node.id.clone()
        };

        Some((effective_ms, skill_key, skill_name))
    }

    /// Recursively resolve dynamic string inputs (Data Pins: msg_in, text_in, val_in)
    /// Supports Unreal-style pure nodes: format_text, var_get, and variable
    pub fn resolve_data_string(
        engine: &crate::engine::NodeExecutionEngine,
        target_node_id: &str,
        port_names: &[&str],
    ) -> Option<String> {
        let in_conns = engine.connections_to_map.get(target_node_id)?;

        for conn in in_conns {
            let matches_port = port_names.iter().any(|p| {
                conn.to_port.eq_ignore_ascii_case(p)
            });
            if !matches_port {
                continue;
            }

            if let Some(from_node) = engine.nodes_map.get(&conn.from_node_id) {
                match from_node.node_type.as_str() {
                    "format_text" => {
                        let template = from_node.data.get("template")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let bool_format = from_node.data.get("boolFormat")
                            .and_then(|v| v.as_str())
                            .unwrap_or("true_false");

                        let mut result = template.to_string();

                        // Parse all {token} occurrences in template
                        let mut start = 0;
                        let mut tokens = Vec::new();
                        while let Some(open) = template[start..].find('{') {
                            let open_idx = start + open;
                            if let Some(close) = template[open_idx..].find('}') {
                                let close_idx = open_idx + close;
                                let token = &template[open_idx + 1..close_idx];
                                if !token.is_empty() {
                                    tokens.push(token.to_string());
                                }
                                start = close_idx + 1;
                            } else {
                                break;
                            }
                        }

                        for token in tokens {
                            let token_val = Self::resolve_data_string(engine, &from_node.id, &[&token])
                                .or_else(|| {
                                    engine.variables_map.get(&token).map(|v| match v {
                                        serde_json::Value::String(s) => s.clone(),
                                        serde_json::Value::Bool(b) => match bool_format {
                                            "yes_no" => if *b { "Yes".to_string() } else { "No".to_string() },
                                            "on_off" => if *b { "ON".to_string() } else { "OFF".to_string() },
                                            _ => b.to_string(),
                                        },
                                        serde_json::Value::Number(n) => n.to_string(),
                                        _ => v.to_string(),
                                    })
                                })
                                .unwrap_or_default();

                            result = result.replace(&format!("{{{}}}", token), &token_val);
                        }
                        return Some(result);
                    }
                    "var_get" => {
                        let var_name = from_node.data.get("varName").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(val) = engine.variables_map.get(var_name) {
                            return Some(match val {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                serde_json::Value::Number(n) => n.to_string(),
                                _ => val.to_string(),
                            });
                        }
                        if let Some(def_val) = from_node.data.get("defaultValue").and_then(|v| v.as_str()) {
                            return Some(def_val.to_string());
                        }
                    }
                    "variable" => {
                        let var_name = from_node.data.get("name")
                            .or_else(|| from_node.data.get("varName"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if let Some(val) = engine.variables_map.get(var_name) {
                            return Some(match val {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                serde_json::Value::Number(n) => n.to_string(),
                                _ => val.to_string(),
                            });
                        }
                        if let Some(def_val) = from_node.data.get("value")
                            .or_else(|| from_node.data.get("defaultValue"))
                            .and_then(|v| v.as_str())
                        {
                            return Some(def_val.to_string());
                        }
                    }
                    _ => {
                        if let Some(msg) = from_node.data.get("message")
                            .or_else(|| from_node.data.get("text"))
                            .and_then(|v| v.as_str())
                        {
                            return Some(msg.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Recursively execute a node and propagate downstream signals across all 29 node types
    pub fn execute_node<'a>(
        state: AppState,
        node: WorkflowNode,
        call_stack: &'a mut HashSet<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if !state.is_running.load(Ordering::SeqCst) || state.hotkey_listener.is_suspended() {
                return;
            }

            // Cycle Guard
            if call_stack.contains(&node.id) {
                state.log("warn", &format!("Cycle detected at node '{}', aborting recursion.", node.id));
                return;
            }
            call_stack.insert(node.id.clone());

            let mut output_port = "onComplete";

            // Dispatch all 29 node types
            match node.node_type.as_str() {
                // 1. Key Press
                "key_press" => {
                    let target_str = node.get_target_client();
                    let client_ids = Self::resolve_target_clients(&state, &target_str).await;

                    // Skill Cooldown Guard
                    let cd_spec = Self::get_node_cooldown_spec(&state, &node).await;
                    let mut eligible_clients = Vec::new();

                    if let Some((cd_ms, ref skill_key, ref skill_name)) = cd_spec {
                        let clients_to_check = if client_ids.is_empty() {
                            vec![1]
                        } else {
                            client_ids.clone()
                        };

                        for &cid in &clients_to_check {
                            if let Some(remaining_secs) = state.check_skill_cooldown(cid, skill_key).await {
                                state.log(
                                    "action",
                                    &format!(
                                        "⏳ [Client {}] Skill '{}' on cooldown ({:.1}s remaining) - Guarded!",
                                        cid, skill_name, remaining_secs
                                    ),
                                );
                            } else {
                                eligible_clients.push(cid);
                                state.set_skill_cooldown(cid, skill_key, Duration::from_millis(cd_ms)).await;
                            }
                        }

                        if eligible_clients.is_empty() {
                            // All target clients are on cooldown - abort keypress and route to onCooldown port
                            state.signal_broadcaster.emit(&node.id, "onCooldown", Some(&target_str));
                            Self::propagate_downstream(state.clone(), &node.id, "onCooldown").await;
                            return;
                        }
                    } else {
                        eligible_clients = if client_ids.is_empty() { vec![1] } else { client_ids };
                    }

                    let mut keys_vec = node.get_str_list("keys");
                    if keys_vec.is_empty() {
                        keys_vec.push(node.get_str("targetKey", "1"));
                    }

                    let delay_after = node.get_u64("delayAfter", 0);

                    state.log("action", &format!("⌨️ [Client {}] Key Press: {}", target_str, keys_vec.join("+")));

                    for client_id in &eligible_clients {
                        for k in &keys_vec {
                            Self::dispatch_cdp_key(&state, *client_id, k, 40).await;
                        }
                    }

                    if delay_after > 0 {
                        if !Self::sleep_ms(&state, delay_after).await {
                            return;
                        }
                    }

                    Self::finish(&state, &node, "onComplete", Some(&target_str)).await;
                    return;
                }

                // 2. Key Hold
                "key_hold" => {
                    let target_str = node.get_target_client();
                    let client_ids = Self::resolve_target_clients(&state, &target_str).await;
                    let key = node.get_str("targetKey", "SPACE");
                    let norm_key = key.trim().to_uppercase();
                    let mode = node.get_str("holdMode", "toggle");

                    // Determine whether to hold down or release
                    let is_down = match mode.as_str() {
                        "release" => false,
                        "hold" | "press" => true,
                        _ => {
                            // "toggle": Check current held state of primary target client
                            let lock = state.held_keys.read().await;
                            let first_client = client_ids.first().copied().unwrap_or(1);
                            !lock.contains(&(first_client, norm_key.clone()))
                        }
                    };

                    {
                        let mut lock = state.held_keys.write().await;
                        for &client_id in &client_ids {
                            if is_down {
                                lock.insert((client_id, norm_key.clone()));
                            } else {
                                lock.remove(&(client_id, norm_key.clone()));
                            }
                        }
                    }

                    let action_label = if is_down { "ENGAGED" } else { "RELEASED" };
                    state.log("action", &format!("⚓ [Client {}] Key Hold {} on '{}'", target_str, action_label, key));

                    for client_id in &client_ids {
                        Self::dispatch_cdp_key_hold(&state, *client_id, &key, is_down).await;
                    }

                    output_port = if is_down { "onEnable" } else { "onDisable" };
                    state.signal_broadcaster.emit(&node.id, output_port, Some(&target_str));
                    state.signal_broadcaster.emit(&node.id, "onComplete", Some(&target_str));

                    // Propagate to both specific state port (onEnable/onDisable) and legacy onComplete port
                    Self::propagate_downstream(state.clone(), &node.id, output_port).await;
                    Self::propagate_downstream(state.clone(), &node.id, "onComplete").await;
                    return;
                }

                // 3. Key Forwarder
                "forwarder" => {
                    let from = node.get_str("sourceClient", "1");
                    let to = node.get_target_client();
                    let key = node.get_str("targetKey", "1");
                    let client_ids = Self::resolve_target_clients(&state, &to).await;

                    // Skill Cooldown Guard
                    let cd_spec = Self::get_node_cooldown_spec(&state, &node).await;
                    let mut eligible_clients = Vec::new();

                    if let Some((cd_ms, ref skill_key, ref skill_name)) = cd_spec {
                        let clients_to_check = if client_ids.is_empty() { vec![1] } else { client_ids.clone() };
                        for &cid in &clients_to_check {
                            if let Some(remaining_secs) = state.check_skill_cooldown(cid, skill_key).await {
                                state.log("action", &format!("⏳ [Client {}] Forwarder Skill '{}' on cooldown ({:.1}s remaining) - Guarded!", cid, skill_name, remaining_secs));
                            } else {
                                eligible_clients.push(cid);
                                state.set_skill_cooldown(cid, skill_key, Duration::from_millis(cd_ms)).await;
                            }
                        }
                        if eligible_clients.is_empty() {
                            state.signal_broadcaster.emit(&node.id, "onCooldown", Some(&to));
                            Self::propagate_downstream(state.clone(), &node.id, "onCooldown").await;
                            return;
                        }
                    } else {
                        eligible_clients = if client_ids.is_empty() { vec![1] } else { client_ids };
                    }

                    state.log("action", &format!("🔁 Forward Key '{}' from Client {} -> {}", key, from, to));

                    for client_id in &eligible_clients {
                        Self::dispatch_cdp_key(&state, *client_id, &key, 40).await;
                    }

                    Self::finish(&state, &node, "onComplete", Some(&to)).await;
                    return;
                }

                // 4. Buff Sequence
                "buff_sequence" => {
                    let target_str = node.get_target_client();
                    let client_ids = Self::resolve_target_clients(&state, &target_str).await;

                    // Skill Cooldown Guard
                    let cd_spec = Self::get_node_cooldown_spec(&state, &node).await;
                    if let Some((cd_ms, ref skill_key, ref skill_name)) = cd_spec {
                        let clients_to_check = if client_ids.is_empty() { vec![1] } else { client_ids.clone() };
                        let mut any_on_cd = false;
                        for &cid in &clients_to_check {
                            if let Some(remaining_secs) = state.check_skill_cooldown(cid, skill_key).await {
                                state.log("action", &format!("⏳ [Client {}] Buff Sequence '{}' on cooldown ({:.1}s remaining) - Guarded!", cid, skill_name, remaining_secs));
                                any_on_cd = true;
                            }
                        }
                        if any_on_cd {
                            state.signal_broadcaster.emit(&node.id, "onCooldown", Some(&target_str));
                            Self::propagate_downstream(state.clone(), &node.id, "onCooldown").await;
                            return;
                        }
                        for &cid in &clients_to_check {
                            state.set_skill_cooldown(cid, skill_key, Duration::from_millis(cd_ms)).await;
                        }
                    }

                    let mut keys_vec = node.get_str_list("keys");
                    if keys_vec.is_empty() {
                        keys_vec = vec!["1".to_string(), "2".to_string()];
                    }
                    let delay_buff = node.get_u64("delayBuff", 800);
                    let delay_after = node.get_u64("delayAfter", 0);

                    state.log("action", &format!("⚡ [Client {}] Executing Buff Sequence: {} (delayBuff: {}ms)", target_str, keys_vec.join("->"), delay_buff));

                    for (idx, k) in keys_vec.iter().enumerate() {
                        if !state.is_running.load(Ordering::SeqCst) || state.hotkey_listener.is_suspended() {
                            return;
                        }
                        for client_id in &client_ids {
                            Self::dispatch_cdp_key(&state, *client_id, k, 50).await;
                        }
                        if idx + 1 < keys_vec.len() && delay_buff > 0 {
                            if !Self::sleep_ms(&state, delay_buff).await {
                                return;
                            }
                        }
                    }

                    if delay_after > 0 {
                        if !Self::sleep_ms(&state, delay_after).await {
                            return;
                        }
                    }

                    Self::finish(&state, &node, "onComplete", Some(&target_str)).await;
                    return;
                }

                // 5. Macro Group
                "macro_group" => {
                    let name = node.data.get("groupName").and_then(|v| v.as_str()).unwrap_or("Macro Group");
                    state.log("action", &format!("📦 Executing Macro Group: '{}'", name));
                    state.signal_broadcaster.emit(&node.id, "onComplete", None);
                }

                // 6. Action Branch
                "branch" => {
                    let rule = node.data.get("conditionRule").and_then(|v| v.as_str()).unwrap_or("is_running");
                    let target_action_id = node.data.get("targetActionId").and_then(|v| v.as_str()).unwrap_or("");
                    let is_true = match rule {
                        "is_running" => state.is_running.load(Ordering::SeqCst),
                        "is_stopped" => !state.is_running.load(Ordering::SeqCst),
                        "on_cooldown" => {
                            if !target_action_id.is_empty() {
                                let engine = state.engine.read().await;
                                if let Some(target_node) = engine.get_node(target_action_id) {
                                    if let Some((_, skill_key, _)) = Self::get_node_cooldown_spec(&state, target_node).await {
                                        let primary_client = Self::resolve_target_clients(&state, &target_node.get_target_client()).await.first().copied().unwrap_or(1);
                                        state.check_skill_cooldown(primary_client, &skill_key).await.is_some()
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        "is_ready" => {
                            if !target_action_id.is_empty() {
                                let engine = state.engine.read().await;
                                if let Some(target_node) = engine.get_node(target_action_id) {
                                    if let Some((_, skill_key, _)) = Self::get_node_cooldown_spec(&state, target_node).await {
                                        let primary_client = Self::resolve_target_clients(&state, &target_node.get_target_client()).await.first().copied().unwrap_or(1);
                                        state.check_skill_cooldown(primary_client, &skill_key).await.is_none()
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        _ => false,
                    };
                    output_port = if is_true { "onTrue" } else { "onFalse" };
                    state.signal_broadcaster.emit(&node.id, output_port, None);
                }

                // 7. Variable Branch
                "var_branch" => {
                    let var_name = node.data.get("varName").and_then(|v| v.as_str()).unwrap_or("");
                    let rule = node.data.get("rule").and_then(|v| v.as_str()).unwrap_or("equals");
                    let target_val = node.data.get("value").and_then(|v| v.as_str()).unwrap_or("");

                    let current_val = {
                        let engine = state.engine.read().await;
                        engine.variables_map.get(var_name).cloned()
                    };

                    let is_true = match (current_val, rule) {
                        (Some(v), "equals") => v.as_str().unwrap_or("") == target_val,
                        (Some(v), "not_equals") => v.as_str().unwrap_or("") != target_val,
                        _ => false,
                    };

                    output_port = if is_true { "onTrue" } else { "onFalse" };
                    state.signal_broadcaster.emit(&node.id, output_port, None);
                }

                // 8. Condition Group
                "condition_group" => {
                    let logic_mode = node.data.get("logicMode").and_then(|v| v.as_str()).unwrap_or("AND");
                    let is_true = logic_mode == "AND" || state.is_running.load(Ordering::SeqCst);
                    output_port = if is_true { "onTrue" } else { "onFalse" };
                    state.signal_broadcaster.emit(&node.id, output_port, None);
                }

                // 9. Loop
                "loop" => {
                    let interval = node.data.get("interval").and_then(|v| v.as_u64()).unwrap_or(1000);
                    let jitter = node.data.get("jitter").and_then(|v| v.as_u64()).unwrap_or(0);
                    let exec_immediately = node.data.get("executeImmediately").and_then(|v| v.as_bool()).unwrap_or(true);
                    let target_str = node.data.get("targetClient").and_then(|v| v.as_str()).unwrap_or("1");
                    let keys: Option<Vec<String>> = node.data.get("keys").and_then(|v| v.as_array()).map(|arr| {
                        arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
                    });

                    state.log("action", &format!("🔁 Starting Loop Node '{}' (Interval: {}ms, Jitter: {}ms)", node.id, interval, jitter));

                    let state_loop = state.clone();
                    let node_id = node.id.clone();
                    let target_str = target_str.to_string();

                    tokio::spawn(async move {
                        if exec_immediately {
                            if let Some(ref ks) = keys {
                                let client_ids = Self::resolve_target_clients(&state_loop, &target_str).await;
                                for cid in &client_ids {
                                    for k in ks {
                                        Self::dispatch_cdp_key(&state_loop, *cid, k, 40).await;
                                    }
                                }
                            }
                            state_loop.signal_broadcaster.emit(&node_id, "onEachCycle", Some(&target_str));
                            Self::propagate_downstream(state_loop.clone(), &node_id, "onEachCycle").await;
                        }

                        while state_loop.is_running.load(Ordering::SeqCst) && !state_loop.hotkey_listener.is_suspended() {
                            let mut sleep_ms = interval;
                            if jitter > 0 {
                                let extra = rand::thread_rng().gen_range(0..=jitter);
                                sleep_ms += extra;
                            }
                            if !Self::sleep_ms(&state_loop, sleep_ms).await {
                                break;
                            }

                            if let Some(ref ks) = keys {
                                let client_ids = Self::resolve_target_clients(&state_loop, &target_str).await;
                                for cid in &client_ids {
                                    for k in ks {
                                        Self::dispatch_cdp_key(&state_loop, *cid, k, 40).await;
                                    }
                                }
                            }

                            state_loop.signal_broadcaster.emit(&node_id, "onEachCycle", Some(&target_str));
                            Self::propagate_downstream(state_loop.clone(), &node_id, "onEachCycle").await;
                        }
                    });

                    return;
                }

                // 10. Loop Scheduler (Human Jitter)
                "loop_scheduler" => {
                    let min_i = node.data.get("intervalMin").and_then(|v| v.as_u64()).unwrap_or(2000);
                    let max_i = node.data.get("intervalMax").and_then(|v| v.as_u64()).unwrap_or(4000);
                    state.log("action", &format!("⏱️ Starting Loop Scheduler Node '{}' (Min: {}ms, Max: {}ms)", node.id, min_i, max_i));

                    let state_loop = state.clone();
                    let node_id = node.id.clone();

                    tokio::spawn(async move {
                        while state_loop.is_running.load(Ordering::SeqCst) && !state_loop.hotkey_listener.is_suspended() {
                            let delay = rand::thread_rng().gen_range(min_i..=max_i);
                            if !Self::sleep_ms(&state_loop, delay).await {
                                break;
                            }

                            state_loop.signal_broadcaster.emit(&node_id, "onInterval", None);
                            Self::propagate_downstream(state_loop.clone(), &node_id, "onInterval").await;
                        }
                    });

                    return;
                }

                // 11. Delay Timer
                "delay" => {
                    let ms = node.data.get("delayMs").or_else(|| node.data.get("delay")).or_else(|| node.data.get("duration")).and_then(|v| v.as_u64()).unwrap_or(500);
                    if Self::sleep_ms(&state, ms).await {
                        state.signal_broadcaster.emit(&node.id, "onComplete", None);
                    }
                }

                // 12. Step Sequencer
                "sequencer" => {
                    state.log("action", "🔢 Sequencer step advanced");
                    output_port = "step1";
                    state.signal_broadcaster.emit(&node.id, "step1", None);
                }

                // 13. Reroute Pin
                "reroute" => {
                    output_port = "out";
                    state.signal_broadcaster.emit(&node.id, "out", None);
                }

                // 14. Party Scanner (Vision)
                "party_scanner" => {
                    let target = node.data.get("targetClient").and_then(|v| v.as_str()).unwrap_or("1");
                    state.log("action", &format!("👁️ [Client {}] Party Scanner: Active HP scan complete", target));
                    output_port = "onScanned";
                    state.signal_broadcaster.emit(&node.id, "onScanned", Some(target));
                }

                // 15. Party Auto-Heal (Vision)
                "party_heal" => {
                    let target = node.data.get("targetClient").and_then(|v| v.as_str()).unwrap_or("1");
                    let threshold = node.data.get("hpThreshold").and_then(|v| v.as_f64()).unwrap_or(70.0);
                    state.log("action", &format!("🩺 [Client {}] Party Heal: Checking for members <= {}% HP", target, threshold));
                    output_port = "onHealTarget";
                    state.signal_broadcaster.emit(&node.id, "onHealTarget", Some(target));
                }

                // 16. Party Buff Cycle (Vision)
                "party_buff" => {
                    let target = node.data.get("targetClient").and_then(|v| v.as_str()).unwrap_or("1");
                    state.log("action", &format!("🛡️ [Client {}] Party Buff: Targeting next member in rotation", target));
                    output_port = "onTargetSelected";
                    state.signal_broadcaster.emit(&node.id, "onTargetSelected", Some(target));
                }

                // 17. Select Party Slot (Vision)
                "party_slot" => {
                    let slot = node.data.get("slotNumber").and_then(|v| v.as_u64()).unwrap_or(1);
                    state.log("action", &format!("👤 Target Party Slot {}", slot));
                    state.signal_broadcaster.emit(&node.id, "onComplete", None);
                }

                // 18. Capture Screen (Screenshot)
                "screenshot" => {
                    let target_str = node.data.get("targetClient")
                        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_u64().map(|n| n.to_string())))
                        .unwrap_or_else(|| "1".to_string());
                    let target_id: u32 = target_str.parse().unwrap_or(1);

                    let subfolder = node.data.get("subfolder")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("client_{}", target_id));

                    let prefix = node.data.get("prefix")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or("snap")
                        .to_string();

                    let region = node.data.get("captureRegion")
                        .and_then(|v| v.as_str())
                        .unwrap_or("full")
                        .to_string();

                    let annotate = node.data.get("annotate")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    state.log("action", &format!("📸 Capturing screen frame for Client {} (Region: {}, Folder: {})...", target_id, region, subfolder));

                    let state_snap = state.clone();
                    let node_id = node.id.clone();

                    tokio::spawn(async move {
                        // 1. Try CDP Session of the target client
                        let cdp_session = {
                            let sessions = state_snap.cdp_sessions.read().await;
                            sessions.get(&target_id).cloned()
                        };

                        let img_result = if let Some(session) = cdp_session {
                            match session.capture_screenshot("jpeg", 90).await {
                                Ok(bytes) => Ok(bytes),
                                Err(e) => Err(format!("CDP capture failed: {}", e)),
                            }
                        } else {
                            Err(format!("Client {} browser session is not running in Launcher", target_id))
                        };

                        match img_result {
                            Ok(raw_bytes) => {
                                match crate::vision::save_and_process_screenshot(
                                    &raw_bytes,
                                    &state_snap.config_store.base_dir,
                                    &subfolder,
                                    &prefix,
                                    target_id,
                                    &region,
                                    annotate,
                                ) {
                                    Ok((filepath, filename)) => {
                                        state_snap.log("action", &format!(
                                            "📸 [Client {}] Saved screenshot ({}) -> ./screenshots/{}/{}",
                                            target_id, region, subfolder, filename
                                        ));
                                        state_snap.signal_broadcaster.emit(&node_id, "onComplete", Some(&filepath.to_string_lossy()));
                                        Self::propagate_downstream(state_snap.clone(), &node_id, "onComplete").await;
                                    }
                                    Err(e) => {
                                        state_snap.log("error", &format!("❌ [Screenshot] File save error: {}", e));
                                        state_snap.signal_broadcaster.emit(&node_id, "onError", Some(&e));
                                        Self::propagate_downstream(state_snap.clone(), &node_id, "onError").await;
                                    }
                                }
                            }
                            Err(cdp_err) => {
                                // Fallback: Try desktop screen capture if available
                                #[cfg(windows)]
                                {
                                    state_snap.log("warn", &format!("⚠️ [Screenshot] {}. Attempting OS Desktop screen capture fallback...", cdp_err));
                                    match crate::vision::capture_desktop_and_save(
                                        &state_snap.config_store.base_dir,
                                        &subfolder,
                                        &prefix,
                                        target_id,
                                        &region,
                                        annotate,
                                    ) {
                                        Ok((filepath, filename)) => {
                                            state_snap.log("action", &format!(
                                                "📸 [Desktop Fallback] Saved screenshot -> ./screenshots/{}/{}",
                                                subfolder, filename
                                            ));
                                            state_snap.signal_broadcaster.emit(&node_id, "onComplete", Some(&filepath.to_string_lossy()));
                                            Self::propagate_downstream(state_snap.clone(), &node_id, "onComplete").await;
                                            return;
                                        }
                                        Err(fallback_err) => {
                                            state_snap.log("error", &format!("❌ [Screenshot] Desktop fallback also failed: {}", fallback_err));
                                        }
                                    }
                                }

                                #[cfg(not(windows))]
                                {
                                    state_snap.log("warn", &format!("⚠️ [Screenshot] {}", cdp_err));
                                }

                                state_snap.signal_broadcaster.emit(&node_id, "onError", Some(&cdp_err));
                                Self::propagate_downstream(state_snap.clone(), &node_id, "onError").await;
                            }
                        }
                    });

                    return;
                }

                // 19. Variable Def
                "variable" => {
                    output_port = "val_out";
                    state.signal_broadcaster.emit(&node.id, "val_out", None);
                }

                // 20. Get Variable
                "var_get" => {
                    output_port = "val_out";
                    state.signal_broadcaster.emit(&node.id, "val_out", None);
                }

                // 21. Set Variable
                "var_set" => {
                    let name = node.data.get("varName").and_then(|v| v.as_str()).unwrap_or("");
                    let val = node.data.get("value").cloned().unwrap_or_default();
                    {
                        let mut engine = state.engine.write().await;
                        engine.variables_map.insert(name.to_string(), val);
                    }
                    state.log("action", &format!("📝 Variable '{}' updated", name));
                    state.signal_broadcaster.emit(&node.id, "onComplete", None);
                }

                // 22. Format Text
                "format_text" => {
                    output_port = "val_out";
                    state.signal_broadcaster.emit(&node.id, "val_out", None);
                }

                // 23. Log Message (Step Log)
                "step_log" => {
                    let dynamic_msg = {
                        let engine = state.engine.read().await;
                        Self::resolve_data_string(&engine, &node.id, &["msg_in", "text_in", "val_in", "in"])
                    };
                    let msg = dynamic_msg.unwrap_or_else(|| {
                        node.data.get("message")
                            .or_else(|| node.data.get("text"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Log")
                            .to_string()
                    });
                    state.log("log", &msg);
                    state.signal_broadcaster.emit(&node.id, "onComplete", None);
                }

                // 24. Play Sound
                "sound" => {
                    let source_type = node.data.get("soundSource").and_then(|v| v.as_str()).unwrap_or("preset");
                    let sound_path = if source_type == "upload" {
                        if let Some(file_str) = node.data.get("soundFile").and_then(|v| v.as_str()) {
                            let p = std::path::Path::new(file_str);
                            if p.is_absolute() && p.exists() {
                                p.to_path_buf()
                            } else {
                                let relative_p = state.config_store.base_dir.join(file_str.trim_start_matches('/'));
                                if relative_p.exists() {
                                    relative_p
                                } else {
                                    state.config_store.base_dir.join("public/sounds").join(p.file_name().unwrap_or_default())
                                }
                            }
                        } else {
                            state.config_store.base_dir.join("public/sounds/ding.wav")
                        }
                    } else {
                        let preset_raw = node.data.get("soundPreset")
                            .or_else(|| node.data.get("soundName"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("ding");
                        let filename = if preset_raw.ends_with(".wav") || preset_raw.ends_with(".mp3") {
                            preset_raw.to_string()
                        } else {
                            format!("{}.wav", preset_raw)
                        };
                        state.config_store.base_dir.join("public/sounds").join(filename)
                    };

                    let volume_raw = node.data.get("volume")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0) as f32;
                    let volume = (if volume_raw > 1.0 { volume_raw / 100.0 } else { volume_raw }).clamp(0.0, 1.0);

                    let repeat_count = node.data.get("repeatCount")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1)
                        .clamp(1, 10) as usize;

                    if let Some(player) = &state.audio_player {
                        for i in 0..repeat_count {
                            player.play_sfx(&sound_path, volume, i == 0);
                            if repeat_count > 1 && i < repeat_count - 1 {
                                tokio::time::sleep(Duration::from_millis(450)).await;
                            }
                        }
                    }
                    state.signal_broadcaster.emit(&node.id, "onComplete", None);
                }

                // 25. Text-to-Speech (TTS)
                "tts" => {
                    let dynamic_text = {
                        let engine = state.engine.read().await;
                        Self::resolve_data_string(&engine, &node.id, &["text_in", "msg_in", "val_in", "in"])
                    };
                    let text = dynamic_text.unwrap_or_else(|| {
                        node.data.get("text")
                            .or_else(|| node.data.get("message"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    })
                    .trim()
                    .to_string();

                    if text.is_empty() {
                        state.signal_broadcaster.emit(&node.id, "onComplete", None);
                    } else {
                        let voice = node.data.get("voice")
                            .and_then(|v| v.as_str())
                            .unwrap_or("th-TH-PremwadeeNeural")
                            .to_string();
                        let volume = node.data.get("volume")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(100.0) as f32 / 100.0;
                        let wait_for_prev = node.data.get("waitForPrevious")
                            .map(|v| v.as_bool().unwrap_or_else(|| v.as_str().map(|s| s == "true").unwrap_or(false)))
                            .unwrap_or(false);

                        state.log("action", &format!("🗣️ TTS Speak: \"{}\" ({}, Queue: {})", text, voice, wait_for_prev));

                        let state_tts = state.clone();
                        let node_id = node.id.clone();

                        tokio::spawn(async move {
                            match state_tts.tts_manager.synthesize(&text, &voice).await {
                                Ok(audio_path) => {
                                    if let Some(player) = &state_tts.audio_player {
                                        player.play_tts(&audio_path, volume.clamp(0.0, 1.0), wait_for_prev);
                                    }
                                }
                                Err(e) => {
                                    state_tts.log("warn", &format!("⚠️ [TTS] Synthesis error: {}", e));
                                }
                            }
                            state_tts.signal_broadcaster.emit(&node_id, "onComplete", None);
                            Self::propagate_downstream(state_tts.clone(), &node_id, "onComplete").await;
                        });

                        return;
                    }
                }

                // 26. Emergency Stop
                "emergency_stop" => {
                    state.reset_and_stop_engine().await;
                    state.signal_broadcaster.emit(&node.id, "onComplete", None);
                    return;
                }

                // 27. Client Check
                "client_check" => {
                    let client_id = node.data.get("targetClient").and_then(|v| v.as_str()).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
                    let client = state.client_manager.get_client(client_id).await;
                    let is_active = client.map(|c| c.is_running).unwrap_or(false);
                    output_port = if is_active { "onActive" } else { "onInactive" };
                    state.signal_broadcaster.emit(&node.id, output_port, None);
                }

                // 28. Webhook Out
                "webhook_out" => {
                    let url = node.data.get("url")
                        .or_else(|| node.data.get("webhookUrl"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    if url.is_empty() {
                        state.log("warn", "⚠️ [Webhook Out] Missing target URL");
                        output_port = "onError";
                        state.signal_broadcaster.emit(&node.id, "onError", None);
                    } else {
                        let method_str = node.data.get("method")
                            .and_then(|v| v.as_str())
                            .unwrap_or("POST")
                            .to_uppercase();
                        let timeout_ms = node.data.get("timeoutMs")
                            .or_else(|| node.data.get("timeout_ms"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(5000);
                        let headers_val = node.data.get("headers").cloned();
                        let payload_val = node.data.get("payload").cloned();
                        let node_id = node.id.clone();
                        let state_wh = state.clone();

                        tokio::spawn(async move {
                            let client = match reqwest::Client::builder()
                                .timeout(Duration::from_millis(timeout_ms))
                                .build() {
                                    Ok(c) => c,
                                    Err(e) => {
                                        state_wh.log("warn", &format!("⚠️ [Webhook Out] Client build error: {}", e));
                                        state_wh.signal_broadcaster.emit(&node_id, "onError", None);
                                        Self::propagate_downstream(state_wh.clone(), &node_id, "onError").await;
                                        return;
                                    }
                                };

                            let method = match method_str.as_str() {
                                "GET" => reqwest::Method::GET,
                                "PUT" => reqwest::Method::PUT,
                                "DELETE" => reqwest::Method::DELETE,
                                _ => reqwest::Method::POST,
                            };

                            let mut req = client.request(method, &url);

                            if let Some(h) = headers_val {
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

                            if method_str == "POST" || method_str == "PUT" {
                                if let Some(p) = payload_val {
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
                                    req = req.header("Content-Type", "application/json").json(&serde_json::json!({
                                        "content": "⚡ NodeHotkey Alert: Triggered!"
                                    }));
                                }
                            }

                            match req.send().await {
                                Ok(res) => {
                                    let status = res.status();
                                    if status.is_success() {
                                        state_wh.log("action", &format!("📡 [Webhook Out] [{}] {} succeeded ({})", method_str, url, status));
                                        state_wh.signal_broadcaster.emit(&node_id, "onSuccess", None);
                                        Self::propagate_downstream(state_wh.clone(), &node_id, "onSuccess").await;
                                    } else {
                                        state_wh.log("warn", &format!("⚠️ [Webhook Out] [{}] {} returned error HTTP {}", method_str, url, status));
                                        state_wh.signal_broadcaster.emit(&node_id, "onError", None);
                                        Self::propagate_downstream(state_wh.clone(), &node_id, "onError").await;
                                    }
                                }
                                Err(e) => {
                                    state_wh.log("warn", &format!("⚠️ [Webhook Out] Request failed: {}", e));
                                    state_wh.signal_broadcaster.emit(&node_id, "onError", None);
                                    Self::propagate_downstream(state_wh.clone(), &node_id, "onError").await;
                                }
                            }
                        });

                        return;
                    }
                }

                // 29. Emit Event (Broadcast Event across active profiles)
                "emit_event" => {
                    let dynamic_event = {
                        let engine = state.engine.read().await;
                        Self::resolve_data_string(&engine, &node.id, &["event_in", "msg_in", "val_in", "name_in", "in"])
                    };
                    let event_name = dynamic_event.unwrap_or_else(|| {
                        node.data.get("eventName")
                            .or_else(|| node.data.get("event"))
                            .or_else(|| node.data.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    });

                    if !event_name.is_empty() {
                        state.log("action", &format!("📡 Broadcast Event: '{}'", event_name));
                        // Trigger event across all loaded profiles
                        Self::trigger_flow(state.clone(), "event", &event_name).await;
                    } else {
                        state.log("warn", "⚠️ [Emit Event] No eventName specified to broadcast");
                    }

                    output_port = "onComplete";
                    state.signal_broadcaster.emit(&node.id, output_port, Some(&event_name));
                }

                // 30. Trigger / Default Pass-through
                _ => {
                    state.signal_broadcaster.emit(&node.id, "onComplete", None);
                }
            }

            // Propagate execution downstream
            if !state.is_running.load(Ordering::SeqCst) || state.hotkey_listener.is_suspended() {
                return;
            }

            let downstream = {
                let engine = state.engine.read().await;
                engine.get_downstream_nodes(&node.id, Some(output_port))
            };

            for next in downstream {
                let mut next_stack = call_stack.clone();
                Self::execute_node(state.clone(), next.node, &mut next_stack).await;
            }
        })
    }
}
