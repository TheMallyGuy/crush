use super::api::fetch_universe_id;
use super::bloxstrap::on_bloxstrap_rpc;
use super::config::{config_bool, integration_enabled};
use super::notifications::{
    emit_server_info, fetch_and_store_location, send_location_notification, sleep_schedule_inner,
    update_discord_rpc,
};
use super::patterns::{
    re_bloxstrap_rpc, re_join, re_joined, re_leave, re_private_server_access_code, re_udmux,
};
use super::png::write_game_permission_png;
use super::state::WatcherState;
use super::window::{save_window_geometry, send_bloxstrap_command};
use crate::collector::{end_game_session, log_game, new_game_session, DbConn};
use crate::interactive::find_windows_by_title;
use crate::rpc::{apply_rpc, RpcState};
use crate::simple_i18n::I18n;
use crate::tray::add_menu_item;
use chrono::Utc;
use serde_json::{json, Value};
use tauri::Manager;
use tauri::AppHandle;

pub(super) async fn handle_line(
    app: &AppHandle,
    line: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
    is_vng: bool,
) -> Result<(), String> {
    if let Some(caps) = re_join().captures(line) {
        let instance_id = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let place_id: u64 = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        if state.window_started {
            if let Some(hwnd) = state.roblox_hwnd {
                send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
            }
        }

        state.reset_for_new_game(app);
        state.activity.join_initiated = true;
        state.activity.place_id = Some(place_id);
        state.activity.instance_id = Some(instance_id);
        log::info!(
            "joining place {} instance {}",
            place_id,
            state.activity.instance_id.as_deref().unwrap_or("?")
        );
        return Ok(());
    }

    if !state.activity.in_game
        && state.activity.place_id.is_none()
        && line.contains("GameJoinUtil::joinGamePostPrivateServer")
    {
        state.activity.is_private_server = true;
        if let Some(caps) = re_private_server_access_code().captures(line) {
            state.activity.access_code = Some(caps[1].to_string());
        }
    }

    if let Some(caps) = re_bloxstrap_rpc().captures(line) {
        if let Some(raw) = caps.get(1) {
            on_bloxstrap_rpc(app, raw.as_str(), state, store).await?;
        }
    }

    if let Some(caps) = re_udmux().captures(line) {
        if !state.udmux_handled {
            if let Some(ip) = caps.get(1) {
                fetch_and_store_location(ip.as_str(), state, app).await?;
                state.udmux_handled = true;
                if !state.location_notified {
                    send_location_notification(app, state, store).await?;
                }
            }
        }
        return Ok(());
    }

    if re_joined().is_match(line) {
        on_joined(app, state, store, is_vng).await?;
        return Ok(());
    }

    if state.activity.in_game && re_leave().is_match(line) {
        log::info!("left game");

        let pool = app.state::<DbConn>();

        if config_bool(store, "settings", &["robloxWarpped"]) {
            if let Some(sid) = state.activity.session_id {
                if let Err(e) = end_game_session(pool.inner(), sid).await {
                    log::warn!("collector end_game_session failed: {e}");
                }
            }
        } else {
            log::info!("aborted logging warpped")
        }

        if state.window_started {
            if let Some(hwnd) = state.roblox_hwnd {
                send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
            }
            state.window_started = false;
        }

        let current_count = state.sleep_schedule_count;
        state.reset_for_new_game(app);

        let app_c = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = sleep_schedule_inner(&app_c, false, current_count).await {
                log::warn!("sleep_schedule on leave error: {}", e);
            }
        });

        let i18n = app.state::<I18n>().inner().clone();

        if integration_enabled(store, &["discordRpc", "enable"]) {
            let _ = apply_rpc(
                &app.state::<RpcState>(),
                &i18n.t("rpc.rust.watcher.general"),
                &i18n.t("rpc.rust.watcher.idle"),
            )
            .await;
        }
    }

    Ok(())
}

async fn on_joined(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
    is_vng: bool,
) -> Result<(), String> {
    let Some(place_id) = state.activity.place_id else {
        return Ok(());
    };

    if !state.activity.join_initiated {
        log::warn!("serverId seen without a prior join, stale log?");
        return Ok(());
    }
    if state.activity.in_game || state.activity.notified {
        return Ok(());
    }

    state.activity.in_game = true;
    state.activity.notified = true;

    state.roblox_hwnd = find_windows_by_title("Roblox").into_iter().next();

    if let Some(hwnd) = state.roblox_hwnd {
        log::info!("cached Roblox HWND");

        let universe_id = match fetch_universe_id(place_id).await {
            Ok(uid) => uid,
            Err(e) => {
                log::warn!("failed to fetch universe ID for PNG: {}", e);
                place_id
            }
        };

        state.activity.universe_id = Some(universe_id);

        let store_val = |key: &str| integration_enabled(store, &["interactive", "scopes", key]);
        if let Err(e) = write_game_permission_png(
            universe_id,
            store_val("moveWindow"),
            store_val("setTitle"),
            integration_enabled(
                store,
                &["interactive", "scopes", "transparencyScopes", "enabled"],
            ),
            app,
            is_vng,
        ) {
            log::warn!("failed to write game permission PNG: {}", e);
        }

        save_window_geometry(state);
        send_bloxstrap_command(hwnd, "StartWindow", Value::Null);
        state.window_started = true;
    } else {
        log::warn!("failed to cache Roblox HWND");
    }

    log::info!("joined game {}", place_id);
    save_game_history(state, store, place_id)?;

    log::info!("saving collector info");
    let pool = app.state::<DbConn>();

    if config_bool(store, "settings", &["robloxWarpped"]) {
        if let Some(pid) = state.activity.place_id {
            if let Err(e) = log_game(pool.inner(), pid as i64).await {
                log::warn!("collector log_game failed: {e}");
            }
            match new_game_session(pool.inner(), pid as i64).await {
                Ok(sid) => state.activity.session_id = Some(sid),
                Err(e) => log::warn!("collector new_game_session failed: {e}"),
            }
        }
    } else {
        log::info!("aborting logging warpped")
    }

    if let Some(id) = state.activity.instance_id.as_deref() {
        let location = state.pending_server_location.clone().unwrap_or_default();
        emit_server_info(app, id, place_id, &location);
        add_menu_item(app, "serverinfo", "Server Infomation").ok();
    }

    if !state.location_notified {
        send_location_notification(app, state, store).await?;
    }

    if integration_enabled(store, &["discordRpc", "enable"]) {
        update_discord_rpc(app, state, place_id).await?;
    }

    Ok(())
}

fn save_game_history(
    state: &WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
    place_id: u64,
) -> Result<(), String> {
    let mut history: Vec<Value> = store
        .get("gameHistory")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    history.push(json!({
        "place_id": place_id,
        "instance_id": state.activity.instance_id.as_deref().unwrap_or_default(),
        "timestamp": Utc::now().to_rfc3339(),
    }));

    store.set("gameHistory", Value::Array(history));
    store.save().map_err(|e| e.to_string())
}
