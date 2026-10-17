use std::env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// Browser type to launch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserType {
    Chrome,
    Chromium,
    Edge,
    Firefox,
}

pub struct BrowserLauncher;

impl BrowserLauncher {
    /// Detect available browser executable on the current OS (Windows, Linux, macOS)
    pub fn find_browser_executable(pref: Option<BrowserType>) -> Option<(BrowserType, PathBuf)> {
        #[cfg(target_os = "windows")]
        {
            let prefixes = [
                env::var("PROGRAMFILES").unwrap_or_default(),
                env::var("PROGRAMFILES(X86)").unwrap_or_default(),
                env::var("LOCALAPPDATA").unwrap_or_default(),
            ];

            let candidates = [
                (BrowserType::Chrome, "Google\\Chrome\\Application\\chrome.exe"),
                (BrowserType::Edge, "Microsoft\\Edge\\Application\\msedge.exe"),
                (BrowserType::Firefox, "Mozilla Firefox\\firefox.exe"),
            ];

            for prefix in &prefixes {
                if prefix.is_empty() { continue; }
                for (b_type, rel_path) in &candidates {
                    if let Some(p) = pref {
                        if p != *b_type { continue; }
                    }
                    let p = Path::new(prefix).join(rel_path);
                    if p.exists() {
                        return Some((*b_type, p));
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let candidates = [
                (BrowserType::Chrome, "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                (BrowserType::Edge, "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
                (BrowserType::Firefox, "/Applications/Firefox.app/Contents/MacOS/firefox"),
            ];
            for (b_type, p_str) in &candidates {
                if let Some(p) = pref {
                    if p != *b_type { continue; }
                }
                let p = PathBuf::from(p_str);
                if p.exists() {
                    return Some((*b_type, p));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidates = [
                (BrowserType::Chrome, "google-chrome"),
                (BrowserType::Chromium, "chromium"),
                (BrowserType::Chromium, "chromium-browser"),
                (BrowserType::Firefox, "firefox"),
            ];
            for (b_type, bin_name) in &candidates {
                if let Some(p) = pref {
                    if p != *b_type { continue; }
                }
                if let Ok(output) = Command::new("which").arg(bin_name).output() {
                    if output.status.success() {
                        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path_str.is_empty() {
                            return Some((*b_type, PathBuf::from(path_str)));
                        }
                    }
                }
            }
        }

        None
    }

    /// Launch browser with Anti-Detection flags and Remote Debugging Port enabled
    pub fn launch(
        client_id: u32,
        debug_port: u16,
        user_data_dir: &Path,
        target_url: &str,
        proxy: Option<&str>,
        user_agent: Option<&str>,
        browser_pref: Option<BrowserType>,
        app_mode: bool,
        window_bounds: Option<(i32, i32, u32, u32)>,
    ) -> Result<Child, String> {
        let (_b_type, exe_path) = Self::find_browser_executable(browser_pref)
            .ok_or_else(|| "No supported browser (Chrome, Chromium, Edge, Firefox) found on this system.".to_string())?;

        let abs_user_data = if user_data_dir.is_absolute() {
            user_data_dir.to_path_buf()
        } else {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(user_data_dir)
        };

        if let Err(e) = std::fs::create_dir_all(&abs_user_data) {
            eprintln!("⚠️ [Browser Launcher] Warning: Could not create dir {:?}: {}", abs_user_data, e);
        }

        let mut cmd = Command::new(&exe_path);

        // Core remote debugging & isolated profile
        cmd.arg(format!("--remote-debugging-port={}", debug_port));
        cmd.arg(format!("--user-data-dir={}", abs_user_data.display()));

        // Anti-Detect & Performance Optimization Flags
        cmd.arg("--disable-blink-features=AutomationControlled");
        cmd.arg("--test-type"); // Suppresses "unsupported command-line flag" warning banner in Chrome
        cmd.arg("--no-first-run");
        cmd.arg("--no-default-browser-check");
        cmd.arg("--disable-infobars");
        cmd.arg("--disable-extensions");
        cmd.arg("--disable-component-update");
        cmd.arg("--disable-background-timer-throttling");
        cmd.arg("--disable-backgrounding-occluded-windows");
        cmd.arg("--disable-renderer-backgrounding");
        cmd.arg("--renderer-process-limit=1");
        cmd.arg(format!("--window-name=Client_{}", client_id));

        // Restore window position and size if recorded
        if let Some((x, y, w, h)) = window_bounds {
            if w >= 200 && h >= 200 {
                cmd.arg(format!("--window-position={},{}", x, y));
                cmd.arg(format!("--window-size={},{}", w, h));
            }
        }

        // Proxy injection
        if let Some(proxy_addr) = proxy {
            if !proxy_addr.trim().is_empty() {
                cmd.arg(format!("--proxy-server={}", proxy_addr.trim()));
            }
        }

        // Custom User-Agent
        if let Some(ua) = user_agent {
            if !ua.trim().is_empty() {
                cmd.arg(format!("--user-agent={}", ua.trim()));
            }
        }

        // Target URL & App Mode (Borderless App Window)
        if app_mode && (_b_type == BrowserType::Chrome || _b_type == BrowserType::Chromium || _b_type == BrowserType::Edge) {
            cmd.arg(format!("--app={}", target_url));
        } else {
            cmd.arg(target_url);
        }

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn browser process {}: {}", exe_path.display(), e))?;

        println!("🚀 [Browser Launcher] Launched Client {} (PID: {}) on port {}", client_id, child.id(), debug_port);

        Ok(child)
    }

    /// Terminate a process tree by PID cleanly across platforms
    pub fn kill_process_tree(pid: u32) {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .spawn();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .spawn();
        }
    }
}
