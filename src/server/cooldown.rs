use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Scan and assemble cooldown skill presets exactly like Node.js cooldown-manager.js
pub fn get_cooldown_presets(base_dir: &Path) -> Value {
    let mut presets: HashMap<String, Vec<Value>> = HashMap::new();
    let mut presets_by_id: HashMap<String, Value> = HashMap::new();
    let mut class_icons: HashMap<String, String> = HashMap::new();

    let skills_dir = base_dir.join("public/images/skills");
    if skills_dir.exists() {
        if let Ok(entries) = fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let class_name = entry.file_name().to_string_lossy().to_string();
                        let preset_path = entry.path().join("presets.json");
                        if preset_path.exists() {
                            if let Ok(content) = fs::read_to_string(&preset_path) {
                                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                                    let mut items_vec = Vec::new();
                                    if let Some(arr) = val.as_array() {
                                        for item in arr {
                                            let mut it = item.clone();
                                            if let Some(obj) = it.as_object_mut() {
                                                obj.insert("class".to_string(), json!(class_name));
                                            }
                                            items_vec.push(it);
                                        }
                                    } else if let Some(map) = val.as_object() {
                                        for (_k, v) in map {
                                            if let Some(sub_arr) = v.as_array() {
                                                for item in sub_arr {
                                                    let mut it = item.clone();
                                                    if let Some(obj) = it.as_object_mut() {
                                                        obj.insert("class".to_string(), json!(class_name));
                                                    }
                                                    items_vec.push(it);
                                                }
                                            }
                                        }
                                    }

                                    for it in &items_vec {
                                        if let Some(id) = it.get("id").and_then(|i| i.as_str()) {
                                            presets_by_id.insert(id.to_string(), it.clone());
                                        }
                                    }

                                    presets.insert(class_name.clone(), items_vec);
                                }
                            }
                        }

                        let icon_path = entry.path().join("class.png");
                        if icon_path.exists() {
                            class_icons.insert(
                                class_name.clone(),
                                format!("/images/skills/{}/class.png", class_name),
                            );
                        }
                    }
                }
            }
        }
    }

    let root_presets = base_dir.join("cooldown_presets.json");
    if root_presets.exists() {
        if let Ok(content) = fs::read_to_string(&root_presets) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(arr) = val.as_array() {
                    presets.insert("Custom".to_string(), arr.clone());
                } else if let Some(arr) = val.get("cooldownPresets").and_then(|v| v.as_array()) {
                    presets.insert("Custom".to_string(), arr.clone());
                }
            }
        }
    }

    json!({
        "success": true,
        "presets": presets,
        "presetsById": presets_by_id,
        "classIcons": class_icons
    })
}

/// Extract all cooldown presets as a fast lookup Map: preset_id -> (cooldown_ms, skill_name)
pub fn get_cooldown_presets_map(base_dir: &Path) -> HashMap<String, (u64, String)> {
    let mut map = HashMap::new();
    let val = get_cooldown_presets(base_dir);
    if let Some(by_id) = val.get("presetsById").and_then(|v| v.as_object()) {
        for (id, item) in by_id {
            let ms = item.get("cooldownMs").and_then(|v| v.as_u64()).unwrap_or(0);
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or(id).to_string();
            map.insert(id.clone(), (ms, name));
        }
    }
    map
}

