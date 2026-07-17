use super::config::{get_transparency_bound, integration_enabled};
use super::state::WatcherState;
use super::types::{BloxstrapRpcMessage, RichPresence};
use super::window::{do_reset_window, get_or_find_hwnd, save_window_geometry};
#[cfg(target_os = "windows")]
use crate::interactive::{
    get_monitor_info, get_primary_screen_size, get_virtual_screen_size, move_window, reset_layered,
    set_borderless, set_layered_transparency, set_window_color, set_window_title, LWA_ALPHA,
    LWA_COLORKEY,
};
use crate::rpc::{apply_rpc_full, start_rpc, RpcState};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

pub(super) async fn on_bloxstrap_rpc(
    app: &AppHandle,
    raw: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    log::info!("BloxstrapRPC raw: {}", raw);

    let msg: BloxstrapRpcMessage = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("BloxstrapRPC: failed to parse: {} raw: {}", e, raw);
            return Ok(());
        }
    };

    log::info!("BloxstrapRPC command: {}", msg.command);

    match msg.command.as_str() {
        "SetRichPresence" => {
            if !integration_enabled(store, &["discordRpc", "enable"]) {
                return Ok(());
            }

            let rpc: RichPresence = match serde_json::from_value(msg.data) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("BloxstrapRPC: SetRichPresence parse failed: {}", e);
                    return Ok(());
                }
            };

            log::info!("BloxstrapRPC SetRichPresence: {:?}", rpc);
            state.bloxstrap_rpc = Some(rpc.clone());

            let app_c = app.clone();
            tauri::async_runtime::spawn(async move {
                let rpc_state = app_c.state::<RpcState>();
                const CLIENT_ID: &str = "363445589247131668";

                if rpc_state.client.lock().await.is_none() {
                    if let Err(e) = start_rpc(&rpc_state, CLIENT_ID).await {
                        log::error!("RPC start failed: {}", e);
                        return;
                    }
                }

                let res = tokio::time::timeout(
                    Duration::from_secs(5),
                    apply_rpc_full(
                        &rpc_state,
                        rpc.details.as_deref(),
                        rpc.state.as_deref(),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                )
                .await;

                match res {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => log::error!("BloxstrapRPC apply failed: {}", e),
                    Err(_) => log::error!("BloxstrapRPC apply timed out"),
                }
            });
        }

        #[cfg(target_os = "windows")]
        "RequestWindowPermission" => {
            log::info!("BloxstrapRPC: RequestWindowPermission (handled via PNG)");
        }
        #[cfg(target_os = "windows")]
        "StartWindow" => {
            if state.window_started {
                return Ok(());
            }
            if get_or_find_hwnd(state).is_some() {
                save_window_geometry(state);
                state.window_started = true;
                log::info!("BloxstrapRPC: StartWindow – geometry saved");
            } else {
                log::warn!("BloxstrapRPC: StartWindow – no HWND");
            }
        }
        #[cfg(target_os = "windows")]
        "StopWindow" => {
            if !state.window_started {
                return Ok(());
            }
            if let Some(hwnd) = state.roblox_hwnd {
                do_reset_window(hwnd, state);
            }
            state.window_started = false;
            log::info!("BloxstrapRPC: StopWindow");
        }
        #[cfg(target_os = "windows")]
        "ResetWindow" => {
            if !state.window_started {
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else {
                return Ok(());
            };
            do_reset_window(hwnd, state);
            log::info!("BloxstrapRPC: ResetWindow");
        }
        #[cfg(target_os = "windows")]
        "SetWindow" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                return Ok(());
            }
            if !integration_enabled(store, &["interactive", "scopes", "moveWindow"]) {
                log::info!("BloxstrapRPC: SetWindow – moveWindow scope disabled");
                return Ok(());
            }
            if !state.window_started {
                log::warn!("BloxstrapRPC: SetWindow before StartWindow, ignoring");
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else {
                log::warn!("BloxstrapRPC: SetWindow – no HWND");
                return Ok(());
            };

            if msg
                .data
                .get("reset")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                do_reset_window(hwnd, state);
                return Ok(());
            }

            let (mon_x, mon_y, mon_w, mon_h) = get_monitor_info(hwnd);
            let screen_w = mon_w as f64;
            let screen_h = mon_h as f64;

            if let Some(v) = msg.data.get("scaleWidth").and_then(|v| v.as_f64()) {
                state.last_sc_width = v;
            }
            if let Some(v) = msg.data.get("scaleHeight").and_then(|v| v.as_f64()) {
                state.last_sc_height = v;
            }

            let scale_x = screen_w / state.last_sc_width;
            let scale_y = screen_h / state.last_sc_height;

            if let Some(v) = msg.data.get("width").and_then(|v| v.as_f64()) {
                state.last_width = (v * scale_x).round() as i32;
            }
            if let Some(v) = msg.data.get("height").and_then(|v| v.as_f64()) {
                state.last_height = (v * scale_y).round() as i32;
            }

            let (primary_w, primary_h) = get_primary_screen_size();
            let (virtual_w, virtual_h) = get_virtual_screen_size();
            let width_mult = primary_w as f64 / virtual_w as f64;
            let height_mult = primary_h as f64 / virtual_h as f64;

            if let Some(v) = msg.data.get("x").and_then(|v| v.as_f64()) {
                let fake_width_fix =
                    (state.last_width as f64 - state.last_width as f64 * width_mult) / 2.0;
                state.last_x = (v * scale_x + fake_width_fix).round() as i32;
            }
            if let Some(v) = msg.data.get("y").and_then(|v| v.as_f64()) {
                let fake_height_fix =
                    (state.last_height as f64 - state.last_height as f64 * height_mult) / 2.0;
                state.last_y = (v * scale_y + fake_height_fix).round() as i32;
            }

            let final_x = state.last_x + mon_x;
            let final_y = state.last_y + mon_y;
            let final_w = (state.last_width as f64 * width_mult).round() as i32;
            let final_h = (state.last_height as f64 * height_mult).round() as i32;

            move_window(hwnd, final_x, final_y, final_w, final_h);
            log::info!(
                "SetWindow → screen({screen_w}x{screen_h}) mon({mon_x},{mon_y}) \
                 → move({final_x},{final_y},{final_w},{final_h})"
            );
        }
        #[cfg(target_os = "windows")]
        "SetWindowTitle" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                return Ok(());
            }
            if !integration_enabled(store, &["interactive", "scopes", "setTitle"]) {
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else {
                return Ok(());
            };
            let title = msg.data.as_str().unwrap_or("Roblox");
            set_window_title(hwnd, title);
        }
        #[cfg(target_os = "windows")]
        "SetWindowTransparency" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                return Ok(());
            }
            if !integration_enabled(
                store,
                &["interactive", "scopes", "transparencyScopes", "enabled"],
            ) {
                return Ok(());
            }
            if !state.window_started {
                log::warn!("BloxstrapRPC: SetWindowTransparency before StartWindow, ignoring");
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else {
                return Ok(());
            };

            if let Some(t) = msg.data.get("transparency").and_then(|v| v.as_f64()) {
                state.last_transparency = (t.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            if let Some(c) = msg.data.get("color").and_then(|v| v.as_str()) {
                state.last_window_color = u32::from_str_radix(c, 16).unwrap_or(0);
            }
            if let Some(use_alpha) = msg.data.get("useAlpha").and_then(|v| v.as_bool()) {
                state.last_transparency_mode = if use_alpha { LWA_ALPHA } else { LWA_COLORKEY };
            }

            let min = get_transparency_bound(store, "minTransparency", 0);
            let max = get_transparency_bound(store, "maxTransparency", 255);
            let clamped = state.last_transparency.clamp(min, max);

            if clamped == 255 {
                reset_layered(hwnd);
            } else {
                set_layered_transparency(
                    hwnd,
                    state.last_window_color,
                    clamped,
                    state.last_transparency_mode,
                );
            }
        }
        #[cfg(target_os = "windows")]
        "SetWindowBorderless" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                return Ok(());
            }
            if !integration_enabled(store, &["interactive", "scopes", "moveWindow"]) {
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else {
                return Ok(());
            };
            let enabled = msg
                .data
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            set_borderless(hwnd, enabled);
            state.borderless = enabled;
            log::info!("BloxstrapRPC: SetWindowBorderless = {}", enabled);
        }
        #[cfg(target_os = "windows")]
        "SetWindowColor" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else {
                return Ok(());
            };

            let reset = msg
                .data
                .get("reset")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let (caption, border) = if reset {
                (Some(0x1F1F1Fu32), Some(0x1F1F1Fu32))
            } else {
                let caption = msg
                    .data
                    .get("caption")
                    .and_then(|v| v.as_str())
                    .and_then(|s| u32::from_str_radix(s, 16).ok());
                let border = msg
                    .data
                    .get("border")
                    .and_then(|v| v.as_str())
                    .and_then(|s| u32::from_str_radix(s, 16).ok());
                (caption, border)
            };

            set_window_color(hwnd, caption, border);
            log::info!(
                "BloxstrapRPC: SetWindowColor caption={:?} border={:?}",
                caption,
                border
            );
        }
        #[cfg(target_os = "windows")]
        "SendNotification" => {
            let title = msg
                .data
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("[[MISSING TITLE]]");
            let caption = msg
                .data
                .get("caption")
                .and_then(|v| v.as_str())
                .unwrap_or("[[MISSING CAPTION]]");
            let _duration = msg
                .data
                .get("duration")
                .and_then(|v| v.as_u64())
                .unwrap_or(5);

            app.notification()
                .builder()
                .title(title)
                .body(caption)
                .show()
                .map_err(|e| e.to_string())?;

            log::info!("BloxstrapRPC: SendNotification '{}' '{}'", title, caption);
        }

        other => {
            log::warn!("BloxstrapRPC: unknown command '{}'", other);
        }
    }

    Ok(())
}
