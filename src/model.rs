use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canvas Viewport State (Zoom & Pan)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanvasSettings {
    #[serde(default = "default_zoom")]
    pub zoom: f64,
    #[serde(default)]
    pub pan: CanvasPan,
}

fn default_zoom() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanvasPan {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

/// Node Position on Canvas
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct NodePosition {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

/// Blueprint Node Definition in JSON
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub position: NodePosition,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl WorkflowNode {
    pub fn new(
        id: impl Into<String>,
        node_type: impl Into<String>,
        title: Option<String>,
        x: f64,
        y: f64,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            node_type: node_type.into(),
            title,
            position: NodePosition { x, y },
            data,
            extra: HashMap::new(),
        }
    }

    /// Extract string property from node data with fallback default
    pub fn get_str(&self, key: &str, default: &str) -> String {
        self.data
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    /// Extract u64 numeric property from node data with fallback default
    pub fn get_u64(&self, key: &str, default: u64) -> u64 {
        self.data
            .get(key)
            .and_then(|v| v.as_u64())
            .unwrap_or(default)
    }

    /// Extract i64 numeric property from node data with fallback default
    pub fn get_i64(&self, key: &str, default: i64) -> i64 {
        self.data
            .get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }

    /// Extract f64 float property from node data with fallback default
    pub fn get_f64(&self, key: &str, default: f64) -> f64 {
        self.data
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or(default)
    }

    /// Extract boolean property from node data with fallback default
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.data
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Extract array of strings from node data (e.g. keys: ["1", "2"])
    pub fn get_str_list(&self, key: &str) -> Vec<String> {
        self.data
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract target client string (defaults to "")
    pub fn get_target_client(&self) -> String {
        self.get_str("targetClient", "")
    }
}

/// Connection Wire (Exec or Data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,
    #[serde(rename = "fromPort")]
    pub from_port: String,
    #[serde(rename = "toNodeId")]
    pub to_node_id: String,
    #[serde(rename = "toPort")]
    pub to_port: String,
}

/// Variable stored in the profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String, // "string" | "number" | "boolean"
    #[serde(default)]
    pub default_value: serde_json::Value,
    #[serde(default)]
    pub current_value: Option<serde_json::Value>,
}

/// Full Profile (Compatible with configs/profiles/*.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default = "default_version")]
    pub version: String,
    pub name: String,
    #[serde(default)]
    pub canvas: CanvasSettings,
    #[serde(default)]
    pub nodes: Vec<WorkflowNode>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    #[serde(default)]
    pub variables: Vec<ProfileVariable>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            version: default_version(),
            name: "Default".to_string(),
            canvas: CanvasSettings::default(),
            nodes: Vec::new(),
            connections: Vec::new(),
            variables: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

impl Profile {
    /// Ensure all nodes have valid position values
    pub fn normalize(&mut self) {
        for node in &mut self.nodes {
            if node.position.x == 0.0 && node.position.y == 0.0 {
                if let (Some(x_val), Some(y_val)) = (node.extra.get("x"), node.extra.get("y")) {
                    if let (Some(x), Some(y)) = (x_val.as_f64(), y_val.as_f64()) {
                        node.position = NodePosition { x, y };
                    }
                }
            }
        }
    }

    /// Remove stray screen, browser, and global settings from profile extra fields
    pub fn sanitize_profile_keys(&mut self) {
        let global_keys = [
            "clientAliases",
            "clientBrowsers",
            "clientUserAgents",
            "clientProxies",
            "clientSlots",
            "targetUrlKeyword",
            "enableOverlay",
            "suspendHotkey",
            "ghostMouseJitter",
            "clientMuteAudio",
            "clientFpsLimit",
            "clientRamLimit",
            "clientScale1x",
            "clientWindowBounds",
            "gpuAcceleration",
            "enablePipOverlay",
            "webPort",
            "disabledClients",
        ];
        for k in &global_keys {
            self.extra.remove(*k);
        }
    }
}

fn default_version() -> String {
    "4.0.0".to_string()
}

/// Ghost Mouse Jitter settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GhostMouseSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(rename = "intervalMin", default = "default_jitter_min")]
    pub interval_min: u64,
    #[serde(rename = "intervalMax", default = "default_jitter_max")]
    pub interval_max: u64,
    #[serde(rename = "maxOffset", default = "default_max_offset")]
    pub max_offset: i32,
}

fn default_jitter_min() -> u64 { 8000 }
fn default_jitter_max() -> u64 { 25000 }
fn default_max_offset() -> i32 { 12 }

/// Creator Identity for Community Hub
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatorIdentity {
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "authorSecret")]
    pub author_secret: String,
    #[serde(rename = "authorName")]
    pub author_name: String,
}

/// Global Settings from configs/global.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    #[serde(rename = "targetUrlKeyword", default = "default_target_url")]
    pub target_url_keyword: String,
    #[serde(rename = "enableOverlay", default = "default_true")]
    pub enable_overlay: bool,
    #[serde(rename = "suspendHotkey", default = "default_suspend_hotkey")]
    pub suspend_hotkey: String,
    #[serde(rename = "ghostMouseJitter", default)]
    pub ghost_mouse_jitter: GhostMouseSettings,
    #[serde(rename = "clientSlots", default = "default_client_slots")]
    pub client_slots: Vec<u32>,
    #[serde(rename = "clientAliases", default)]
    pub client_aliases: HashMap<String, String>,
    #[serde(rename = "clientUserAgents", default)]
    pub client_user_agents: HashMap<String, String>,
    #[serde(rename = "clientProxies", default)]
    pub client_proxies: HashMap<String, String>,
    #[serde(rename = "clientBrowsers", default)]
    pub client_browsers: HashMap<String, String>,
    #[serde(rename = "clientWindowBounds", default)]
    pub client_window_bounds: HashMap<String, serde_json::Value>,
    #[serde(rename = "gpuAcceleration", default = "default_true")]
    pub gpu_acceleration: bool,
    #[serde(rename = "enablePipOverlay", default)]
    pub enable_pip_overlay: bool,
    #[serde(rename = "webPort", default = "default_web_port")]
    pub web_port: u16,
}

fn default_target_url() -> String { "universe.flyff.com/play".to_string() }
fn default_true() -> bool { true }
fn default_suspend_hotkey() -> String { "END".to_string() }
fn default_client_slots() -> Vec<u32> { vec![1] }
fn default_web_port() -> u16 { 3088 }

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            target_url_keyword: default_target_url(),
            enable_overlay: true,
            suspend_hotkey: default_suspend_hotkey(),
            ghost_mouse_jitter: GhostMouseSettings::default(),
            client_slots: default_client_slots(),
            client_aliases: HashMap::new(),
            client_user_agents: HashMap::new(),
            client_proxies: HashMap::new(),
            client_browsers: HashMap::new(),
            client_window_bounds: HashMap::new(),
            gpu_acceleration: true,
            enable_pip_overlay: false,
            web_port: default_web_port(),
        }
    }
}

/// Global Config root in configs/global.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(rename = "activeProfile", default = "default_active_profile")]
    pub active_profile: String,
    #[serde(rename = "activeProfiles", default = "default_active_profiles")]
    pub active_profiles: Vec<String>,
    #[serde(rename = "disabledClients", default)]
    pub disabled_clients: Vec<u32>,
    #[serde(rename = "globalSettings", default)]
    pub global_settings: GlobalSettings,
    #[serde(rename = "creatorIdentity", default)]
    pub creator_identity: Option<CreatorIdentity>,
}

fn default_active_profile() -> String { "Default".to_string() }
fn default_active_profiles() -> Vec<String> { vec!["Default".to_string()] }

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            active_profile: default_active_profile(),
            active_profiles: default_active_profiles(),
            disabled_clients: Vec::new(),
            global_settings: GlobalSettings::default(),
            creator_identity: None,
        }
    }
}
