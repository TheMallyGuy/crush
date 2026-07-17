use super::state::WatcherState;
#[cfg(target_os = "windows")]
use crate::interactive::{
    find_windows_by_title, get_window_rect, move_window, reset_layered, set_borderless,
    set_window_title, LWA_COLORKEY,
};
use serde_json::{json, Value};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

#[cfg(target_os = "windows")]
pub(super) fn save_window_geometry(state: &mut WatcherState) {
    let Some(hwnd) = state.roblox_hwnd else {
        return;
    };
    if let Some((x, y, w, h)) = get_window_rect(hwnd) {
        state.starting_x = x;
        state.starting_y = y;
        state.starting_width = w;
        state.starting_height = h;
        state.last_x = x;
        state.last_y = y;
        state.last_width = w;
        state.last_height = h;
    }
    state.last_sc_width = 1280.0;
    state.last_sc_height = 720.0;
}

#[cfg(target_os = "windows")]
pub(super) fn do_reset_window(hwnd: HWND, state: &mut WatcherState) {
    state.last_x = state.starting_x;
    state.last_y = state.starting_y;
    state.last_width = state.starting_width;
    state.last_height = state.starting_height;
    state.last_transparency = 255;
    state.last_window_color = 0x000000;
    state.last_transparency_mode = LWA_COLORKEY;

    move_window(
        hwnd,
        state.starting_x,
        state.starting_y,
        state.starting_width,
        state.starting_height,
    );
    reset_layered(hwnd);
    set_borderless(hwnd, false);
    set_window_title(hwnd, "Roblox");
    state.borderless = false;
}

#[cfg(target_os = "windows")]
pub(super) fn get_or_find_hwnd(state: &mut WatcherState) -> Option<HWND> {
    if let Some(hwnd) = state.roblox_hwnd {
        return Some(hwnd);
    }
    let hwnd = find_windows_by_title("Roblox").into_iter().next();
    if hwnd.is_some() {
        state.roblox_hwnd = hwnd;
    }
    hwnd
}

pub(super) fn send_bloxstrap_command(_hwnd: HWND, command: &str, data: Value) {
    let payload =
        serde_json::to_string(&json!({ "command": command, "data": data })).unwrap_or_default();
    log::info!("Sending Bloxstrap command: {}", payload);
}
