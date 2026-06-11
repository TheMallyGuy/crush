use super::config::integration_enabled;
use super::log_file::{get_latest_log, maybe_switch_log_file, read_new_lines};
use super::notifications::sleep_schedule_inner;
use super::process::{find_hwnd_by_pid, get_roblox_pid, is_roblox_running};
use super::state::WatcherState;
use super::window::send_bloxstrap_command;
use crate::larp_focuser::start_larping;
use crate::rpc::{kill_rpc, RpcState};
use crate::SdkState;
use serde_json::Value;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::System;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;
use tokio_util::sync::CancellationToken;
use windows::Win32::Foundation::HWND;

static WATCHER_CANCEL: Mutex<Option<CancellationToken>> = Mutex::new(None);

#[tauri::command]
pub fn watch_logs(app: AppHandle, is_vng: Option<bool>) -> Result<(), String> {
    if let Ok(mut guard) = WATCHER_CANCEL.lock() {
        if let Some(token) = guard.take() {
            token.cancel();
        }
    }

    let store = app.store("config.json").map_err(|e| e.to_string())?;
    if !integration_enabled(&store, &["activityWatching"]) {
        log::info!("watching logs is disabled, returning");
        return Ok(());
    }

    let token = CancellationToken::new();

    if let Ok(mut guard) = WATCHER_CANCEL.lock() {
        *guard = Some(token.clone());
    }

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build watcher runtime");

        rt.block_on(async move {
            tokio::select! {
                result = run_watcher(app, is_vng.unwrap_or(false)) => {
                    if let Err(e) = result {
                        log::error!("watcher error: {}", e);
                    }
                }
                _ = token.cancelled() => {
                    log::info!("watcher cancelled by new watch_logs call");
                }
            }
        });
    });

    Ok(())
}

async fn run_watcher(app: AppHandle, is_vng: bool) -> Result<(), String> {
    let mut state = WatcherState::default();
    let mut system = System::new();
    let mut was_running = false;
    let store = app.store("config.json").map_err(|e| e.to_string())?;
    let mut last_sleep_check = Instant::now() - Duration::from_secs(61);
    // throttle expensive operations so they don't run every tick
    let mut last_process_check = Instant::now() - Duration::from_millis(501);
    let mut cached_running = false;
    let mut last_log_dir_check = Instant::now() - Duration::from_secs(3);
    log::info!("watcher is now running");

    loop {
        // full process enumeration is expensive — only do it every 500ms
        if last_process_check.elapsed() >= Duration::from_millis(500) {
            last_process_check = Instant::now();
            cached_running = is_roblox_running(&mut system);
        }
        let running = cached_running;

        if running && state.roblox_hwnd.is_none() {
            let roblox_pid = get_roblox_pid(&mut system);

            if let Some(pid) = roblox_pid {
                state.roblox_hwnd = find_hwnd_by_pid(pid);
            }
        }
        if let Some(hwnd) = state.roblox_hwnd {
            if integration_enabled(&store, &["optimizer"]) && !state.larp_started {
                state.larp_started = true;
                let app_c = app.clone();
                let hwnd_val = hwnd.0 as usize;
                std::thread::spawn(move || {
                    let hwnd = HWND(hwnd_val as *mut _);
                    start_larping(hwnd, app_c);
                });
            }
        }

        if was_running && !running {
            if state.window_started {
                if let Some(hwnd) = state.roblox_hwnd {
                    send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
                }
            }
            if integration_enabled(&store, &["swifttunnel", "disconnectWhenRobloxClosed"]) {
                if let Ok(sdk) = app.state::<SdkState>().0.lock() {
                    if let Err(e) = sdk.disconnect() {
                        log::warn!("failed to disconnect swifttunnel on roblox close: {}", e);
                    } else {
                        log::info!("swifttunnel disconnected because roblox closed");
                    }
                }
            }
            state.reset_fully();
            kill_rpc(&app.state::<RpcState>()).await.ok();
        }

        was_running = running;

        if running {
            // directory listing is cheap but still unnecessary at 10/s — 2s is plenty
            if last_log_dir_check.elapsed() >= Duration::from_secs(2) {
                last_log_dir_check = Instant::now();
                if let Some(path) = get_latest_log(is_vng) {
                    maybe_switch_log_file(&app, &mut state, path, &store).await;
                }
            }

            if state.current_file.is_some() {
                read_new_lines(&app, &mut state, &store, is_vng).await;
            }

            if last_sleep_check.elapsed() >= Duration::from_secs(60) {
                log::info!("sleep check firing, roblox running={}", running);
                last_sleep_check = Instant::now();

                let app_c = app.clone();
                let in_game = state.activity.in_game;
                let count = state.sleep_schedule_count;

                let handle = tauri::async_runtime::spawn(async move {
                    match sleep_schedule_inner(&app_c, in_game, count).await {
                        Ok(new_count) => new_count,
                        Err(e) => {
                            log::warn!("sleep_schedule error: {}", e);
                            count
                        }
                    }
                });

                match handle.await {
                    Ok(new_count) => state.sleep_schedule_count = new_count,
                    Err(e) => log::warn!("sleep_schedule task panicked: {}", e),
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
