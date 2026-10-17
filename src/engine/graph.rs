use crate::model::{Connection, Profile, WorkflowNode};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DownstreamTarget {
    pub node: WorkflowNode,
    pub connection: Connection,
}

/// Pure Node Workflow Execution Graph Engine in Rust
#[derive(Debug, Clone, Default)]
pub struct NodeExecutionEngine {
    pub profiles: Vec<Profile>,
    pub nodes_map: HashMap<String, WorkflowNode>,
    pub action_id_to_node_map: HashMap<String, WorkflowNode>,
    pub connections_from_map: HashMap<String, Vec<Connection>>,
    pub connections_to_map: HashMap<String, Vec<Connection>>,
    pub variables_map: HashMap<String, serde_json::Value>,
}

impl NodeExecutionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_profiles(&mut self, profiles: Vec<Profile>) {
        self.profiles = profiles;
        self.nodes_map.clear();
        self.action_id_to_node_map.clear();
        self.connections_from_map.clear();
        self.connections_to_map.clear();

        for profile in &self.profiles {
            for var in &profile.variables {
                let initial_val = var.current_value.clone().unwrap_or_else(|| var.default_value.clone());
                self.variables_map.insert(var.name.clone(), initial_val);
            }

            for node in &profile.nodes {
                self.nodes_map.insert(node.id.clone(), node.clone());

                // Action ID index (e.g. data.actionId or stripped node_xxx)
                let action_id = if let Some(act_id) = node.data.get("actionId").and_then(|v| v.as_str()) {
                    act_id.to_string()
                } else if node.id.starts_with("node_") {
                    node.id.strip_prefix("node_").unwrap_or(&node.id).to_string()
                } else {
                    node.id.clone()
                };
                self.action_id_to_node_map.insert(action_id, node.clone());
            }

            for conn in &profile.connections {
                self.connections_from_map
                    .entry(conn.from_node_id.clone())
                    .or_default()
                    .push(conn.clone());

                self.connections_to_map
                    .entry(conn.to_node_id.clone())
                    .or_default()
                    .push(conn.clone());
            }
        }
    }

    /// Reset all variable values back to their default values as declared in profiles
    pub fn reset_variables_to_defaults(&mut self) {
        self.variables_map.clear();
        for profile in &self.profiles {
            for var in &profile.variables {
                let initial_val = var.default_value.clone();
                self.variables_map.insert(var.name.clone(), initial_val);
            }
        }
    }

    pub fn get_node(&self, id_or_action_id: &str) -> Option<&WorkflowNode> {
        self.nodes_map
            .get(id_or_action_id)
            .or_else(|| self.action_id_to_node_map.get(id_or_action_id))
    }

    /// Strict Port Matcher (Reproducing NodeHotkey v3.2.0 compatibility)
    pub fn match_port(conn_port: &str, req_port: Option<&str>) -> bool {
        let req_port = match req_port {
            None => return true, // Wildcard matches any port
            Some(r) if r.is_empty() => return true,
            Some(r) => r,
        };

        if conn_port == req_port {
            return true;
        }

        let norm_conn = conn_port.to_lowercase().replace(['-', '_'], "");
        let norm_req = req_port.to_lowercase().replace(['-', '_'], "");

        if norm_conn == norm_req {
            return true;
        }

        // Generic Flow Continuation group
        let generic_completion = ["next", "execout", "oncomplete", "onsuccess", "onfired", "onselected", "out"];
        let generic_inputs = ["in", "execin", "triggerin", "input"];

        let is_conn_generic = generic_completion.contains(&norm_conn.as_str());
        let is_req_generic = generic_completion.contains(&norm_req.as_str());
        if is_conn_generic && is_req_generic {
            return true;
        }

        let is_conn_input = generic_inputs.contains(&norm_conn.as_str());
        let is_req_input = generic_inputs.contains(&norm_req.as_str());
        if (is_conn_generic && is_req_input) || (is_conn_input && is_req_generic) || (is_conn_input && is_req_input) {
            return true;
        }

        // Interval / Cycle aliases
        let interval_ports = ["oneachcycle", "oninterval", "oncycle"];
        if interval_ports.contains(&norm_conn.as_str()) && interval_ports.contains(&norm_req.as_str()) {
            return true;
        }

        // Party Heal aliases
        let heal_target_ports = ["onhealtarget", "ontargetselected", "onmemberlowhp"];
        if heal_target_ports.contains(&norm_conn.as_str()) && heal_target_ports.contains(&norm_req.as_str()) {
            return true;
        }

        let all_healthy_ports = ["onnotarget", "onallhealthy"];
        if all_healthy_ports.contains(&norm_conn.as_str()) && all_healthy_ports.contains(&norm_req.as_str()) {
            return true;
        }

        // Client Check / Active status aliases
        let active_ports = ["onactive", "ontrue", "onopen", "isactive", "isopen"];
        if active_ports.contains(&norm_conn.as_str()) && active_ports.contains(&norm_req.as_str()) {
            return true;
        }

        let inactive_ports = ["oninactive", "onfalse", "onclosed", "isinactive", "isclosed"];
        if inactive_ports.contains(&norm_conn.as_str()) && inactive_ports.contains(&norm_req.as_str()) {
            return true;
        }

        false
    }

    /// Get downstream connected nodes from a specific output port
    pub fn get_downstream_nodes(&self, from_id_or_action_id: &str, port_name: Option<&str>) -> Vec<DownstreamTarget> {
        let node = match self.get_node(from_id_or_action_id) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let outgoing = match self.connections_from_map.get(&node.id) {
            Some(conns) => conns,
            None => return Vec::new(),
        };

        let mut targets = Vec::new();
        for conn in outgoing {
            if Self::match_port(&conn.from_port, port_name) {
                if let Some(target_node) = self.nodes_map.get(&conn.to_node_id) {
                    targets.push(DownstreamTarget {
                        node: target_node.clone(),
                        connection: conn.clone(),
                    });
                }
            }
        }

        targets
    }

    /// Find trigger nodes matching input trigger type and value
    pub fn get_matching_trigger_nodes(&self, trigger_type: &str, trigger_value: &str) -> Vec<WorkflowNode> {
        let val_norm = normalize_trigger_key(trigger_value);
        let trig_type = trigger_type.to_lowercase().trim().to_string();

        let mut matched = Vec::new();
        for node in self.nodes_map.values() {
            if node.node_type == "trigger" {
                let n_type = node.data.get("triggerType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("keyboard")
                    .to_lowercase();
                let n_val = node.data.get("triggerValue")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let is_enabled = node.data.get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                if is_enabled && n_type == trig_type && (normalize_trigger_key(n_val) == val_norm || n_val.eq_ignore_ascii_case(trigger_value)) {
                    matched.push(node.clone());
                }
            }
        }

        matched
    }

    /// Cycle Detector Guard
    pub fn check_and_push_callstack(stack: &mut HashSet<String>, node_id: &str) -> bool {
        if stack.contains(node_id) {
            return false; // Cycle detected!
        }
        stack.insert(node_id.to_string());
        true
    }
}

pub fn normalize_trigger_key(k: &str) -> String {
    let s = k.trim().to_uppercase().replace([' ', '_', '-'], "");

    // 1. Numpad / Keypad (Strictly distinct from top-row digits)
    if s == "NUMPAD0" || s == "KP0" {
        return "NUMPAD0".to_string();
    }
    if s == "NUMPAD1" || s == "KP1" {
        return "NUMPAD1".to_string();
    }
    if s == "NUMPAD2" || s == "KP2" {
        return "NUMPAD2".to_string();
    }
    if s == "NUMPAD3" || s == "KP3" {
        return "NUMPAD3".to_string();
    }
    if s == "NUMPAD4" || s == "KP4" {
        return "NUMPAD4".to_string();
    }
    if s == "NUMPAD5" || s == "KP5" {
        return "NUMPAD5".to_string();
    }
    if s == "NUMPAD6" || s == "KP6" {
        return "NUMPAD6".to_string();
    }
    if s == "NUMPAD7" || s == "KP7" {
        return "NUMPAD7".to_string();
    }
    if s == "NUMPAD8" || s == "KP8" {
        return "NUMPAD8".to_string();
    }
    if s == "NUMPAD9" || s == "KP9" {
        return "NUMPAD9".to_string();
    }

    // Numpad Operations
    if s == "NUMPADMINUS" || s == "KPMINUS" || s == "NUMPADSUBTRACT" {
        return "NUMPAD_MINUS".to_string();
    }
    if s == "NUMPADPLUS" || s == "KPPLUS" || s == "NUMPADADD" {
        return "NUMPAD_PLUS".to_string();
    }
    if s == "NUMPADENTER" || s == "KPENTER" || s == "KPRETURN" {
        return "NUMPAD_ENTER".to_string();
    }
    if s == "NUMPADMULTIPLY" || s == "KPMULTIPLY" {
        return "NUMPAD_MULTIPLY".to_string();
    }
    if s == "NUMPADDIVIDE" || s == "KPDIVIDE" {
        return "NUMPAD_DIVIDE".to_string();
    }
    if s == "NUMPADDOT" || s == "KPDOT" || s == "NUMPADDECIMAL" || s == "KPDECIMAL" || s == "KPDELETE" {
        return "NUMPAD_DOT".to_string();
    }

    // 2. Main Keyboard Top Row Digits
    if s == "0" || s == "NUM0" || s == "DIGIT0" {
        return "0".to_string();
    }
    if s == "1" || s == "NUM1" || s == "DIGIT1" {
        return "1".to_string();
    }
    if s == "2" || s == "NUM2" || s == "DIGIT2" {
        return "2".to_string();
    }
    if s == "3" || s == "NUM3" || s == "DIGIT3" {
        return "3".to_string();
    }
    if s == "4" || s == "NUM4" || s == "DIGIT4" {
        return "4".to_string();
    }
    if s == "5" || s == "NUM5" || s == "DIGIT5" {
        return "5".to_string();
    }
    if s == "6" || s == "NUM6" || s == "DIGIT6" {
        return "6".to_string();
    }
    if s == "7" || s == "NUM7" || s == "DIGIT7" {
        return "7".to_string();
    }
    if s == "8" || s == "NUM8" || s == "DIGIT8" {
        return "8".to_string();
    }
    if s == "9" || s == "NUM9" || s == "DIGIT9" {
        return "9".to_string();
    }

    // 3. Main Keyboard Special Keys
    if s == "MINUS" {
        return "-".to_string();
    }
    if s == "PLUS" || s == "EQUALS" || s == "EQUAL" {
        return "+".to_string();
    }
    if s == "SPACE" || s == "SPACEBAR" {
        return "SPACE".to_string();
    }
    if s == "RETURN" || s == "ENTER" {
        return "ENTER".to_string();
    }
    if s == "ESCAPE" || s == "ESC" {
        return "ESC".to_string();
    }

    // Handle rdev KeyA - KeyZ -> "A" - "Z"
    if s.starts_with("KEY") && s.len() == 4 {
        let ch = s.chars().nth(3).unwrap();
        if ch.is_ascii_alphabetic() {
            return ch.to_string();
        }
    }
    s
}
