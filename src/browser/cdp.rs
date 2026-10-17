use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct CdpRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

impl CdpRequest {
    pub fn new(id: u64, method: &str, params: serde_json::Value) -> Self {
        Self {
            id,
            method: method.to_string(),
            params,
        }
    }

    /// Construct Input.dispatchKeyEvent for KeyDown or KeyUp
    pub fn dispatch_key(id: u64, event_type: &str, key: &str, code: &str, vk: u32) -> Self {
        Self::new(
            id,
            "Input.dispatchKeyEvent",
            json!({
                "type": event_type,
                "key": key,
                "code": code,
                "windowsVirtualKeyCode": vk,
                "nativeVirtualKeyCode": vk,
                "unmodifiedText": key,
                "text": if event_type == "keyDown" { key } else { "" }
            }),
        )
    }

    /// Construct Input.dispatchMouseEvent for MouseMove, MouseDown, MouseUp
    pub fn dispatch_mouse(id: u64, event_type: &str, x: f64, y: f64, button: &str) -> Self {
        Self::new(
            id,
            "Input.dispatchMouseEvent",
            json!({
                "type": event_type,
                "x": x,
                "y": y,
                "button": button,
                "clickCount": if event_type == "mousePressed" { 1 } else { 0 }
            }),
        )
    }

    /// Start compositor screencast for zero-flicker frames
    pub fn start_screencast(id: u64, quality: u32) -> Self {
        Self::new(
            id,
            "Page.startScreencast",
            json!({
                "format": "jpeg",
                "quality": quality,
                "everyNthFrame": 1
            }),
        )
    }
}
