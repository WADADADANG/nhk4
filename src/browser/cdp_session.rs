use crate::browser::anti_stuck::ANTI_STUCK_INJECTION_SCRIPT;
use crate::browser::cdp::CdpRequest;
use crate::state::AppState;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Live Chrome DevTools Protocol Session over WebSocket
#[derive(Clone)]
pub struct CdpSession {
    pub client_id: u32,
    msg_id: Arc<AtomicU64>,
    writer: Arc<Mutex<Option<futures_util::stream::SplitSink<WsStream, Message>>>>,
    pending_requests: Arc<Mutex<std::collections::HashMap<u64, tokio::sync::oneshot::Sender<Value>>>>,
}

impl CdpSession {
    /// Discover WebSocket URL from Chromium HTTP debugging port and connect
    pub async fn connect(client_id: u32, port: u16, state: AppState) -> Result<Self, String> {
        let http_url = format!("http://127.0.0.1:{}/json/list", port);

        // Retry a few times while browser is initializing
        let mut ws_url = None;
        for _ in 0..10 {
            if let Ok(res) = reqwest::get(&http_url).await {
                if let Ok(tabs) = res.json::<Vec<Value>>().await {
                    for tab in tabs {
                        let tab_type = tab.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        if tab_type == "page" {
                            if let Some(ws) = tab.get("webSocketDebuggerUrl").and_then(|w| w.as_str()) {
                                ws_url = Some(ws.to_string());
                                break;
                            }
                        }
                    }
                }
            }
            if ws_url.is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }

        let ws_url = ws_url.ok_or_else(|| format!("Could not find page target on port {}", port))?;

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| format!("Failed to connect WebSocket to {}: {}", ws_url, e))?;

        let (writer, mut reader) = ws_stream.split();
        let writer_arc = Arc::new(Mutex::new(Some(writer)));
        let pending_requests = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let session = Self {
            client_id,
            msg_id: Arc::new(AtomicU64::new(1)),
            writer: writer_arc,
            pending_requests: pending_requests.clone(),
        };

        // Inject Anti-Stuck script immediately
        session.inject_anti_stuck().await?;

        // Inject In-Page Window Bounds Event Binding (CDP WebSocket Event)
        let _ = session.inject_window_bounds_binding().await;

        // Background reader task for events (Runtime.bindingCalled, Screencast frames, etc.)
        let state_clone = state.clone();
        let pending_clone = pending_requests.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = reader.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(v) = serde_json::from_str::<Value>(&text) {
                        // 1. Correlate and dispatch responses for pending call_method requests
                        if let Some(id) = v.get("id").and_then(|i| i.as_u64()) {
                            let mut lock = pending_clone.lock().await;
                            if let Some(tx) = lock.remove(&id) {
                                let _ = tx.send(v);
                                continue;
                            }
                        }

                        // 2. Window Bounds In-Page Event (__nhkSaveBounds)
                        if v.get("method").and_then(|m| m.as_str()) == Some("Runtime.bindingCalled") {
                            if let Some(params) = v.get("params") {
                                if params.get("name").and_then(|n| n.as_str()) == Some("__nhkSaveBounds") {
                                    if let Some(payload_str) = params.get("payload").and_then(|p| p.as_str()) {
                                        if let Ok(b) = serde_json::from_str::<Value>(payload_str) {
                                            let x = b.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                            let y = b.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                            let w = b.get("w").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                            let h = b.get("h").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                            if w >= 200 && h >= 200 && x > -10000 && y > -10000 {
                                                crate::browser::save_client_bounds_if_changed(&state_clone, client_id, x, y, w, h, false).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            println!("🔌 [CDP Session] Client {} disconnected.", client_id);
            state_clone.client_manager.set_running(client_id, false, None, None).await;
            {
                let mut sessions = state_clone.cdp_sessions.write().await;
                sessions.remove(&client_id);
            }
            state_clone.log("system", &format!("Client {} disconnected.", client_id));
            state_clone.notify_status_change().await;
        });

        println!("⚡ [CDP Session] Connected to Client {} via WebSocket!", client_id);

        Ok(session)
    }

    fn next_id(&self) -> u64 {
        self.msg_id.fetch_add(1, Ordering::SeqCst)
    }

    async fn send_request(&self, req: CdpRequest) -> Result<(), String> {
        let json_str = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        let mut lock = self.writer.lock().await;
        if let Some(w) = lock.as_mut() {
            w.send(Message::Text(json_str.into()))
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("WebSocket writer is closed".to_string())
        }
    }

    /// Gracefully close the browser via CDP Browser.close
    pub async fn close_browser(&self) -> Result<(), String> {
        let req = CdpRequest::new(self.next_id(), "Browser.close", serde_json::json!({}));
        self.send_request(req).await
    }

    /// Inject In-Page Anti-Stuck Sentinel
    pub async fn inject_anti_stuck(&self) -> Result<(), String> {
        let req = CdpRequest::new(
            self.next_id(),
            "Runtime.evaluate",
            serde_json::json!({
                "expression": ANTI_STUCK_INJECTION_SCRIPT,
                "includeCommandLineAPI": false,
                "silent": true
            }),
        );
        self.send_request(req).await
    }

    /// Update and lock browser tab / window title with Anti-Overwrite MutationObserver & Page.addScriptToEvaluateOnNewDocument
    pub async fn set_tab_title(&self, prefix: &str) -> Result<(), String> {
        let escaped_prefix = serde_json::to_string(prefix).unwrap_or_else(|_| format!("\"{}\"", prefix));
        let script = format!(
            r#"(function() {{
                try {{
                    window.__clientPrefix = {prefix};
                    function updateTitle() {{
                        const p = window.__clientPrefix || '';
                        if (!p) return;
                        let t = document.title || 'Flyff Universe';
                        if (!t.startsWith(p)) {{
                            if (t.includes('] ')) {{
                                const parts = t.split('] ');
                                if (parts.length > 1 && parts[0].startsWith('[')) {{
                                    t = parts.slice(1).join('] ');
                                }}
                            }}
                            document.title = p + t;
                        }}
                    }}
                    if (!window.__nhk_title_observer) {{
                        window.__nhk_title_observer = new MutationObserver(updateTitle);
                        const target = document.querySelector('title') || document.documentElement;
                        if (target) {{
                            window.__nhk_title_observer.observe(target, {{ subtree: true, characterData: true, childList: true }});
                        }}
                        setInterval(updateTitle, 1000);
                    }}
                    updateTitle();
                }} catch (e) {{}}
            }})()"#,
            prefix = escaped_prefix
        );

        // 1. Enable Page domain
        let _ = self.send_request(CdpRequest::new(self.next_id(), "Page.enable", serde_json::json!({}))).await;

        // 2. Persist across any page navigations and reloads (F5)
        let _ = self.send_request(CdpRequest::new(
            self.next_id(),
            "Page.addScriptToEvaluateOnNewDocument",
            serde_json::json!({ "source": script }),
        )).await;

        // 3. Immediately evaluate in the current active document
        let req = CdpRequest::new(
            self.next_id(),
            "Runtime.evaluate",
            serde_json::json!({
                "expression": script,
                "includeCommandLineAPI": false,
                "silent": true
            }),
        );
        self.send_request(req).await
    }

    /// Inject In-Page Window Bounds Event Binding (Pure CDP Event-Driven)
    pub async fn inject_window_bounds_binding(&self) -> Result<(), String> {
        // 1. Enable Runtime domain
        let _ = self.send_request(CdpRequest::new(self.next_id(), "Runtime.enable", serde_json::json!({}))).await;

        // 2. Add binding named "__nhkSaveBounds"
        let _ = self.send_request(CdpRequest::new(
            self.next_id(),
            "Runtime.addBinding",
            serde_json::json!({ "name": "__nhkSaveBounds" }),
        )).await;

        // 3. Enable Page domain
        let _ = self.send_request(CdpRequest::new(self.next_id(), "Page.enable", serde_json::json!({}))).await;

        let script = r#"(function() {
            try {
                if (window.__nhk_bounds_binding_installed) return;
                window.__nhk_bounds_binding_installed = true;

                let lastBounds = null;
                const reportBounds = () => {
                    try {
                        if (typeof window.__nhkSaveBounds !== 'function') return;
                        const x = window.screenX;
                        const y = window.screenY;
                        const w = window.outerWidth;
                        const h = window.outerHeight;
                        if (typeof x === 'number' && typeof y === 'number' && w >= 200 && h >= 200 && x > -10000 && y > -10000) {
                            if (!lastBounds || lastBounds.x !== x || lastBounds.y !== y || lastBounds.w !== w || lastBounds.h !== h) {
                                lastBounds = { x, y, w, h };
                                window.__nhkSaveBounds(JSON.stringify(lastBounds));
                            }
                        }
                    } catch (e) {}
                };

                window.addEventListener('resize', reportBounds, true);
                window.addEventListener('blur', reportBounds, true);
                window.addEventListener('mouseup', reportBounds, true);
                window.addEventListener('beforeunload', reportBounds, true);
                window.addEventListener('pagehide', reportBounds, true);
                setTimeout(reportBounds, 1000);
            } catch (e) {}
        })()"#;

        // 4. Persist across any page navigations and reloads (F5)
        let _ = self.send_request(CdpRequest::new(
            self.next_id(),
            "Page.addScriptToEvaluateOnNewDocument",
            serde_json::json!({ "source": script }),
        )).await;

        // 5. Immediately evaluate in the current active document
        let req = CdpRequest::new(
            self.next_id(),
            "Runtime.evaluate",
            serde_json::json!({
                "expression": script,
                "includeCommandLineAPI": false,
                "silent": true
            }),
        );
        self.send_request(req).await
    }



    /// Dispatch Key Press (KeyDown -> delay -> KeyUp)
    pub async fn send_key_press(&self, key: &str, code: &str, vk: u32, hold_ms: u64) -> Result<(), String> {
        let down_req = CdpRequest::dispatch_key(self.next_id(), "rawKeyDown", key, code, vk);
        self.send_request(down_req).await?;

        if hold_ms > 0 {
            tokio::time::sleep(Duration::from_millis(hold_ms)).await;
        }

        let up_req = CdpRequest::dispatch_key(self.next_id(), "keyUp", key, code, vk);
        self.send_request(up_req).await?;

        Ok(())
    }

    /// Dispatch Key Hold (KeyDown or KeyUp)
    pub async fn send_key_hold(&self, key: &str, code: &str, vk: u32, is_down: bool) -> Result<(), String> {
        let event_type = if is_down { "rawKeyDown" } else { "keyUp" };
        let req = CdpRequest::dispatch_key(self.next_id(), event_type, key, code, vk);
        self.send_request(req).await
    }

    /// Dispatch Mouse Click
    pub async fn send_mouse_click(&self, x: f64, y: f64, button: &str) -> Result<(), String> {
        let move_req = CdpRequest::dispatch_mouse(self.next_id(), "mouseMoved", x, y, "none");
        self.send_request(move_req).await?;

        let down_req = CdpRequest::dispatch_mouse(self.next_id(), "mousePressed", x, y, button);
        self.send_request(down_req).await?;

        tokio::time::sleep(Duration::from_millis(60)).await;

        let up_req = CdpRequest::dispatch_mouse(self.next_id(), "mouseReleased", x, y, button);
        self.send_request(up_req).await
    }

    /// Dispatch Mouse Move
    pub async fn send_mouse_move(&self, x: f64, y: f64) -> Result<(), String> {
        let req = CdpRequest::dispatch_mouse(self.next_id(), "mouseMoved", x, y, "none");
        self.send_request(req).await
    }

    /// Start GPU Compositor Zero-Flicker Screencast
    pub async fn start_screencast(&self, quality: u32) -> Result<(), String> {
        let req = CdpRequest::start_screencast(self.next_id(), quality);
        self.send_request(req).await
    }

    /// Send CDP request and await JSON response (timeout: 6s)
    pub async fn call_method(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id();
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut lock = self.pending_requests.lock().await;
            lock.insert(id, tx);
        }

        let req = CdpRequest::new(id, method, params);
        if let Err(e) = self.send_request(req).await {
            let mut lock = self.pending_requests.lock().await;
            lock.remove(&id);
            return Err(e);
        }

        match tokio::time::timeout(Duration::from_secs(6), rx).await {
            Ok(Ok(response)) => {
                if let Some(error) = response.get("error") {
                    Err(format!("CDP Error: {}", error))
                } else if let Some(result) = response.get("result") {
                    Ok(result.clone())
                } else {
                    Ok(response)
                }
            }
            Ok(Err(_)) => {
                Err("CDP connection closed while waiting for response".to_string())
            }
            Err(_) => {
                let mut lock = self.pending_requests.lock().await;
                lock.remove(&id);
                Err("CDP request timed out (6s)".to_string())
            }
        }
    }

    /// Capture zero-flicker native viewport screenshot via Page.captureScreenshot
    pub async fn capture_screenshot(&self, format: &str, quality: u32) -> Result<Vec<u8>, String> {
        let params = serde_json::json!({
            "format": format,
            "quality": quality,
            "captureBeyondViewport": false,
            "fromSurface": true
        });
        let res = self.call_method("Page.captureScreenshot", params).await?;
        let b64 = res.get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "CDP captureScreenshot response missing 'data'".to_string())?;

        use base64::prelude::*;
        BASE64_STANDARD.decode(b64.trim())
            .map_err(|e| format!("Failed to decode screenshot base64: {}", e))
    }
}
