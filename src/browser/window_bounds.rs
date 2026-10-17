use crate::state::AppState;

#[cfg(windows)]
pub fn get_client_window_bounds_from_pid(pid: u32) -> Option<(i32, i32, u32, u32)> {
    use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
    use winapi::shared::windef::{HWND, RECT};
    use winapi::um::winuser::{
        EnumWindows, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    };

    struct SearchData {
        target_pid: u32,
        found_rect: Option<(i32, i32, u32, u32)>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam as *mut SearchData);
        let mut proc_id = 0u32;
        GetWindowThreadProcessId(hwnd, &mut proc_id);

        if proc_id == data.target_pid && IsWindowVisible(hwnd) != 0 && IsIconic(hwnd) == 0 {
            let mut rect: RECT = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect) != 0 {
                let w = (rect.right - rect.left) as u32;
                let h = (rect.bottom - rect.top) as u32;
                if w >= 200 && h >= 200 && rect.left > -10000 && rect.top > -10000 {
                    data.found_rect = Some((rect.left, rect.top, w, h));
                    return 0; // Found main game window, stop enumeration
                }
            }
        }
        TRUE
    }

    let mut data = SearchData {
        target_pid: pid,
        found_rect: None,
    };

    unsafe {
        EnumWindows(Some(enum_proc), &mut data as *mut _ as LPARAM);
    }

    data.found_rect
}

#[cfg(not(windows))]
pub fn get_client_window_bounds_from_pid(_pid: u32) -> Option<(i32, i32, u32, u32)> {
    None
}

/// Save client window bounds to global config only if they differ from current config
pub async fn save_client_bounds_if_changed(
    state: &AppState,
    client_id: u32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    is_closing: bool,
) {
    let mut global = state.config_store.get_global_config().await;
    let changed = if let Some(existing) = global
        .global_settings
        .client_window_bounds
        .get(&client_id.to_string())
    {
        let ex_x = existing.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let ex_y = existing.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let ex_w = existing
            .get("w")
            .or_else(|| existing.get("width"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let ex_h = existing
            .get("h")
            .or_else(|| existing.get("height"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        ex_x != x || ex_y != y || ex_w != w || ex_h != h
    } else {
        true
    };

    if changed {
        global.global_settings.client_window_bounds.insert(
            client_id.to_string(),
            serde_json::json!({
                "x": x,
                "y": y,
                "w": w,
                "h": h
            }),
        );
        let _ = state.config_store.save_global_config(&global).await;
        let action_type = if is_closing { "Final window" } else { "Window" };
        let msg = format!(
            "📐 [Client {}] {} bounds saved -> Position: ({}, {}), Size: ({}x{})",
            client_id, action_type, x, y, w, h
        );
        println!("{}", msg);
        state.log("system", &msg);
    }
}

