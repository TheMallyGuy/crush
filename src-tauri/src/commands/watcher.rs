use crate::interactive::{
    find_windows_by_title, get_monitor_info, get_primary_screen_size, get_virtual_screen_size,
    get_window_rect, move_window, reset_layered, set_borderless, set_layered_transparency,
    set_window_color, set_window_title, LWA_ALPHA, LWA_COLORKEY,
};
use crate::rd::get_client;
use crate::rpc::{apply_rpc, apply_rpc_full, kill_rpc, start_rpc, RpcState};
use crate::tray::{add_menu_item, remove_menu_item};
use chrono::Utc;
use dirs_next::data_local_dir;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;
use windows::Win32::Foundation::HWND;

// regex

fn re_join() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"! Joining game '([0-9a-f\-]+)' place (\d+) at ([0-9\.]+)").unwrap()
    })
}
fn re_joined() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"serverId: ([0-9\.]+)\|").unwrap())
}
fn re_leave() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"Time to disconnect replication data").unwrap())
}
fn re_udmux() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").unwrap()
    })
}
fn re_bloxstrap_rpc() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\[BloxstrapRPC\] (.+)").unwrap())
}

// states

#[derive(Default, Debug)]
struct Activity {
    place_id: Option<u64>,
    universe_id: Option<u64>,
    instance_id: Option<String>,
    in_game: bool,
    notified: bool,
    join_initiated: bool,
}

struct WatcherState {
    current_file: Option<PathBuf>,
    offset: u64,
    activity: Activity,
    last_rpc: Option<Instant>,
    udmux_handled: bool,
    pending_server_ip: Option<String>,
    pending_server_location: Option<String>,
    location_notified: bool,
    bloxstrap_rpc: Option<RichPresence>,
    roblox_hwnd: Option<HWND>,
    window_started: bool,

    // window geometry saved at StartWindow
    starting_x: i32,
    starting_y: i32,
    starting_width: i32,
    starting_height: i32,

    // last applied geometry (updated by SetWindow)
    last_x: i32,
    last_y: i32,
    last_width: i32,
    last_height: i32,

    // scale reference resolution (updated by scaleWidth/scaleHeight fields)
    last_sc_width: f64,
    last_sc_height: f64,

    // transparency state
    last_transparency: u8,
    last_window_color: u32,
    last_transparency_mode: u32,

    // misc window state
    borderless: bool,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            current_file: None,
            offset: 0,
            activity: Activity::default(),
            last_rpc: None,
            udmux_handled: false,
            pending_server_ip: None,
            pending_server_location: None,
            location_notified: false,
            bloxstrap_rpc: None,
            roblox_hwnd: None,
            window_started: false,

            starting_x: 0,
            starting_y: 0,
            starting_width: 0,
            starting_height: 0,

            last_x: 0,
            last_y: 0,
            last_width: 0,
            last_height: 0,

            last_sc_width: 1280.0,
            last_sc_height: 720.0,

            last_transparency: 255,
            last_window_color: 0,
            last_transparency_mode: LWA_COLORKEY,

            borderless: false,
        }
    }
}

impl WatcherState {
    fn reset_for_new_game(&mut self, app: &AppHandle) {
        self.activity = Activity::default();
        self.udmux_handled = false;
        self.pending_server_ip = None;
        self.pending_server_location = None;
        self.location_notified = false;
        self.bloxstrap_rpc = None;
        self.roblox_hwnd = None;
        self.window_started = false;
        self.borderless = false;

        remove_menu_item(app, "serverinfo").ok();
    }

    fn reset_fully(&mut self) {
        *self = WatcherState::default();
    }
}

// API types

#[derive(Deserialize)]
struct UniverseResponse {
    #[serde(alias = "universeId")]
    universe_id: u64,
}
#[derive(Deserialize)]
struct GameData {
    name: String,
}
#[derive(Deserialize)]
struct GamesResponse {
    data: Vec<GameData>,
}
#[derive(Deserialize)]
struct IpInfo {
    city: String,
    region: String,
}
#[derive(Deserialize)]
struct IconEntry {
    #[serde(rename = "imageUrl")]
    image_url: String,
}
#[derive(Deserialize)]
struct IconResponse {
    data: Vec<IconEntry>,
}

// bloxstrap rpc types

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RichPresence {
    details: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct BloxstrapRpcMessage {
    command: String,
    #[serde(default)]
    data: Value,
}

// emit data type

#[derive(Serialize, Deserialize, Clone)]
struct EmitServerInfomation {
    server_id: String,
    game_id: u64,
    region_info: String,
}

// entry

static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn watch_logs(app: AppHandle) -> Result<(), String> {
    if WATCHER_RUNNING.swap(true, Ordering::SeqCst) {
        log::warn!("ignoring duplicate watch_logs call");
        return Ok(());
    }

    let store = app.store("config.json").map_err(|e| e.to_string())?;

    if integration_enabled(&store, &["EnableActivityTracking"]) {
        log::info!("watching logs is disabled, returning");
        return Ok(());
    }

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build watcher runtime");

        rt.block_on(async move {
            if let Err(e) = run_watcher(app).await {
                log::error!("watcher error: {}", e);
            }
            WATCHER_RUNNING.store(false, Ordering::SeqCst);
        });
    });

    Ok(())
}

// loop

async fn run_watcher(app: AppHandle) -> Result<(), String> {
    let mut state = WatcherState::default();
    let mut system = System::new();
    let mut was_running = false;
    let store = app.store("config.json").map_err(|e| e.to_string())?;
    log::info!("watcher is now running");

    loop {
        let running = is_roblox_running(&mut system);

        if was_running && !running {
            if state.window_started {
                if let Some(hwnd) = state.roblox_hwnd {
                    send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
                }
            }
            state.reset_fully();
            kill_rpc(&app.state::<RpcState>()).await.ok();
        }

        was_running = running;

        if running {
            if let Some(path) = get_latest_log() {
                maybe_switch_log_file(&app, &mut state, path, &store).await;
            }

            if state.current_file.is_some() {
                read_new_lines(&app, &mut state, &store).await;
            }
        }

        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}

// log management

async fn maybe_switch_log_file(
    app: &AppHandle,
    state: &mut WatcherState,
    path: PathBuf,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) {
    if state.current_file.as_ref() == Some(&path) {
        return;
    }

    let initial_offset = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    log::info!(
        "Switching to log file: {:?} (skipping {} bytes)",
        path,
        initial_offset
    );

    state.reset_fully();
    state.current_file = Some(path);
    state.offset = initial_offset;

    if integration_enabled(store, &["discordRpc", "enable"]) {
        let _ = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", "Not in game").await;
    } else {
        log::info!("Discord RPC integration disabled, skipping initial RPC set");
    }
}

async fn read_new_lines(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) {
    let Some(path) = state.current_file.as_ref() else {
        return;
    };

    if let Ok(metadata) = std::fs::metadata(path) {
        let file_size = metadata.len();
        if file_size > state.offset + 1024 * 1024 {
            log::warn!(
                "Falling behind (offset: {}, size: {}), skipping old logs",
                state.offset,
                file_size
            );
            state.offset = file_size;
            return;
        }
    }

    let mut reader = match open_reader(state) {
        Ok(r) => r,
        Err(e) => {
            log::error!("open reader: {}", e);
            return;
        }
    };

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if let Err(e) = handle_line(app, &line, state, store).await {
                    log::error!("handle_line: {}", e);
                    break;
                }
            }
            Err(e) => {
                log::error!("read_line: {}", e);
                break;
            }
        }
    }

    state.offset = reader.stream_position().unwrap_or(state.offset);
}

fn open_reader(state: &mut WatcherState) -> Result<BufReader<File>, String> {
    let path = state.current_file.as_ref().ok_or("No current file")?;
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(state.offset))
        .map_err(|e| e.to_string())?;
    Ok(BufReader::new(file))
}

// line handler

async fn handle_line(
    app: &AppHandle,
    line: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {

    log::info!("LINE: {}", line);
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

    if let Some(caps) = re_bloxstrap_rpc().captures(line) {
        if let Some(raw) = caps.get(1) {
            on_bloxstrap_rpc(app, raw.as_str(), state, store).await?;
        }
    }

    if let Some(caps) = re_udmux().captures(line) {
        if !state.udmux_handled {
            if let Some(ip) = caps.get(1) {
                fetch_and_store_location(ip.as_str(), state).await?;
                state.udmux_handled = true;
                if !state.location_notified {
                    send_location_notification(app, state, store).await?;
                }
            }
        }
        return Ok(());
    }

    if re_joined().is_match(line) {
        on_joined(app, state, store).await?;
        return Ok(());
    }

    if state.activity.in_game && re_leave().is_match(line) {
        log::info!("left game");
        if state.window_started {
            if let Some(hwnd) = state.roblox_hwnd {
                send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
            }
            state.window_started = false;
        }
        state.reset_for_new_game(app);
        if integration_enabled(store, &["discordRpc", "enable"]) {
            let _ = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", "Not in game").await;
        }
    }

    Ok(())
}

// event handlers

async fn on_joined(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
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

async fn on_bloxstrap_rpc(
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
                const CLIENT_ID: &str = "1484521125550620813";

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

        "RequestWindowPermission" => {
            log::info!("BloxstrapRPC: RequestWindowPermission (handled via PNG)");
        }

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

// some helpers

fn save_window_geometry(state: &mut WatcherState) {
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

fn do_reset_window(hwnd: HWND, state: &mut WatcherState) {
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

fn get_or_find_hwnd(state: &mut WatcherState) -> Option<HWND> {
    if let Some(hwnd) = state.roblox_hwnd {
        return Some(hwnd);
    }
    let hwnd = find_windows_by_title("Roblox").into_iter().next();
    if hwnd.is_some() {
        state.roblox_hwnd = hwnd;
    }
    hwnd
}

fn get_transparency_bound(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    key: &str,
    default: u8,
) -> u8 {
    let v = store
        .get("integrations")
        .or_else(|| store.get("intergrations"));
    let Some(root) = v else { return default };
    root.get("interactive")
        .and_then(|v| v.get("scopes"))
        .and_then(|v| v.get("transparencyScopes"))
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_u64())
        .map(|v| v.clamp(0, 255) as u8)
        .unwrap_or(default)
}

fn write_game_permission_png(
    game_id: u64,
    allow_control: bool,
    allow_title: bool,
    allow_transparency: bool,
    app: &AppHandle,
) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let versions_dir = data_dir.join("Player").join("Versions");

    let version_dir = std::fs::read_dir(&versions_dir)
        .map_err(|e| format!("can't read versions dir {:?}: {}", versions_dir, e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| e.path().join("RobloxPlayerBeta.exe").exists())
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
        .ok_or_else(|| format!("no RobloxPlayerBeta.exe found under {:?}", versions_dir))?;

    let bloxstrap_dir = version_dir.join("content").join("bloxstrap");
    std::fs::create_dir_all(&bloxstrap_dir).map_err(|e| e.to_string())?;

    if let Ok(entries) = std::fs::read_dir(&bloxstrap_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str != "enabled.png" {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    let enabled_path = bloxstrap_dir.join("enabled.png");
    if !enabled_path.exists() {
        write_png_rgba(&enabled_path, 1, 1, &[255, 255, 255, 255])?;
        log::info!("wrote enabled.png to {:?}", enabled_path);
    }

    let game_png_path = bloxstrap_dir.join(format!("{}.png", game_id));
    let pixel = |on: bool| -> [u8; 4] {
        if on {
            [255, 255, 255, 255]
        } else {
            [0, 0, 0, 0]
        }
    };

    let mut pixels: Vec<u8> = Vec::with_capacity(12);
    pixels.extend_from_slice(&pixel(allow_control));
    pixels.extend_from_slice(&pixel(allow_title));
    pixels.extend_from_slice(&pixel(allow_transparency));

    write_png_rgba(&game_png_path, 3, 1, &pixels)?;
    log::info!(
        "wrote game permission PNG for {} -> {:?} (control={}, title={}, transparency={})",
        game_id,
        game_png_path,
        allow_control,
        allow_title,
        allow_transparency
    );

    Ok(())
}

fn write_png_rgba(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    fn adler32(data: &[u8]) -> u32 {
        let (mut a, mut b) = (1u32, 0u32);
        for &byte in data {
            a = (a + byte as u32) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }

    fn crc32(data: &[u8]) -> u32 {
        static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
        let table = TABLE.get_or_init(|| {
            let mut t = [0u32; 256];
            for (i, _) in t.clone().iter().enumerate() {
                let mut c = i as u32;
                for _ in 0..8 {
                    c = if c & 1 != 0 {
                        0xedb88320 ^ (c >> 1)
                    } else {
                        c >> 1
                    };
                }
                t[i] = c;
            }
            t
        });
        let mut crc = 0xffffffff_u32;
        for &byte in data {
            crc = table[((crc ^ byte as u32) & 0xff) as usize] ^ (crc >> 8);
        }
        crc ^ 0xffffffff
    }

    fn write_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
        let len = data.len() as u32;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(tag);
        out.extend_from_slice(data);
        let mut crc_input = Vec::with_capacity(4 + data.len());
        crc_input.extend_from_slice(tag);
        crc_input.extend_from_slice(data);
        out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    }

    let mut out: Vec<u8> = Vec::new();

    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_chunk(&mut out, b"IHDR", &ihdr);

    let mut raw: Vec<u8> = Vec::new();
    for row in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[row * width as usize * 4..(row + 1) * width as usize * 4]);
    }

    let mut zlib: Vec<u8> = Vec::new();
    zlib.push(0x78);
    zlib.push(0x01);
    zlib.push(0x01);

    let len16 = raw.len() as u16;
    let nlen16 = !len16;
    zlib.extend_from_slice(&len16.to_le_bytes());
    zlib.extend_from_slice(&nlen16.to_le_bytes());
    zlib.extend_from_slice(&raw);
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    write_chunk(&mut out, b"IDAT", &zlib);
    write_chunk(&mut out, b"IEND", &[]);

    std::fs::write(path, &out).map_err(|e| e.to_string())
}

// others

fn send_bloxstrap_command(_hwnd: HWND, command: &str, data: Value) {
    let payload =
        serde_json::to_string(&json!({ "command": command, "data": data })).unwrap_or_default();
    log::info!("Sending Bloxstrap command: {}", payload);
}

async fn fetch_and_store_location(ip: &str, state: &mut WatcherState) -> Result<(), String> {
    if state.activity.place_id.is_none() {
        log::info!("UDMUX fired but no place_id, skipping");
        return Ok(());
    }
    log::info!("UDMUX IP: {}", ip);

    let res = tokio::time::timeout(
        Duration::from_secs(5),
        get_client().get(format!("https://ipinfo.io/{}/json", ip)).send(),
    )
    .await;

    let response = match res {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(e.to_string()),
        Err(_) => return Err("ipinfo.io request timed out".to_string()),
    };

    let info: IpInfo = tokio::time::timeout(Duration::from_secs(5), response.json())
        .await
        .map_err(|_| "ipinfo.io json parse timed out".to_string())?
        .map_err(|e| e.to_string())?;

    state.pending_server_ip = Some(ip.to_string());
    state.pending_server_location = Some(format!("{}, {}", info.city, info.region));
    Ok(())
}

async fn send_location_notification(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    if !integration_enabled(store, &["serverLocationNotifier"]) {
        state.pending_server_ip = None;
        state.pending_server_location = None;
        return Ok(());
    }

    if let (Some(ip), Some(location)) = (
        state.pending_server_ip.take(),
        state.pending_server_location.take(),
    ) {
        state.location_notified = true;
        app.notification()
            .builder()
            .title("Connected to a server!")
            .body(format!("IP : {}\nLocation : {}", ip, location))
            .show()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn update_discord_rpc(
    app: &AppHandle,
    state: &mut WatcherState,
    place_id: u64,
) -> Result<(), String> {
    let now = Instant::now();
    if state
        .last_rpc
        .is_some_and(|t| now.duration_since(t).as_secs() <= 2)
    {
        return Ok(());
    }

    state.last_rpc = Some(now);
    let instance_id = state.activity.instance_id.clone().unwrap_or_default();
    let app_c = app.clone();

    tauri::async_runtime::spawn(async move {
        let (name, _image) = match fetch_place_info(place_id).await {
            Ok(Some(v)) => v,
            _ => ("Roblox".to_string(), String::new()),
        };

        let buttons = vec![
            (
                "Join Server".to_string(),
                format!(
                    "https://deeplink.multicrew.dev?placeId={}&jobId={}",
                    place_id, instance_id
                ),
            ),
            (
                "View Game".to_string(),
                format!("https://www.roblox.com/games/{}", place_id),
            ),
        ];

        const CLIENT_ID: &str = "1484521125550620813";
        let rpc = app_c.state::<RpcState>();

        if rpc.client.lock().await.is_none() {
            if let Err(e) = start_rpc(&rpc, CLIENT_ID).await {
                log::error!("RPC start failed: {}", e);
                return;
            }
        }

        let res = tokio::time::timeout(
            Duration::from_secs(5),
            apply_rpc_full(
                &rpc,
                Some("Crush"),
                Some("Playing Roblox"),
                Some(&name),
                None,
                None,
                Some(buttons.clone()),
            ),
        )
        .await;

        if let Ok(Err(e)) = res {
            log::warn!("RPC failed ({}), reconnecting…", e);
            *rpc.client.lock().await = None;
            *rpc.runner.lock().await = None;

            if let Err(e) = start_rpc(&rpc, CLIENT_ID).await {
                log::error!("RPC reconnect failed: {}", e);
                return;
            }

            let _ = tokio::time::timeout(
                Duration::from_secs(5),
                apply_rpc_full(
                    &rpc,
                    Some("Crush"),
                    Some("Playing Roblox"),
                    Some(&name),
                    None,
                    None,
                    Some(buttons),
                ),
            )
            .await;
        } else if let Err(_) = res {
            log::error!("RPC apply timed out");
        }
    });

    Ok(())
}

fn emit_server_info(app: &AppHandle, instance_id: &str, game_id: u64, region_info: &str) {
    let payload = EmitServerInfomation {
        server_id: instance_id.to_string(),
        game_id,
        region_info: region_info.to_string(),
    };
    app.emit("serverInfomation", payload).unwrap();
}

pub fn integration_enabled(store: &tauri_plugin_store::Store<tauri::Wry>, path: &[&str]) -> bool {
    let v = store
        .get("integrations")
        .or_else(|| store.get("intergrations"));
    let Some(mut cur) = v else { return false };
    for key in path {
        cur = cur.get(key).cloned().unwrap_or(Value::Null);
    }
    cur.as_bool().unwrap_or(false)
}

fn is_roblox_running(system: &mut System) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"(?i)robloxplayerbeta").unwrap());
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    system
        .processes()
        .values()
        .any(|p| re.is_match(p.name().to_string_lossy().as_ref()))
}

fn get_latest_log() -> Option<PathBuf> {
    let dir = data_local_dir()?.join("Roblox").join("logs");
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| {
            let e = e.ok()?;
            let path = e.path();
            if path.extension()? != "log" {
                return None;
            }
            let meta = e.metadata().ok()?;
            Some((path, meta))
        })
        .max_by_key(|(_, m)| m.modified().ok())
        .map(|(p, _)| p)
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

async fn fetch_universe_id(place_id: u64) -> Result<u64, String> {
    let res = tokio::time::timeout(
        Duration::from_secs(5),
        get_client()
            .get(format!(
                "https://apis.roblox.com/universes/v1/places/{}/universe",
                place_id
            ))
            .send(),
    )
    .await;

    let response = match res {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(e.to_string()),
        Err(_) => return Err("apis.roblox.com request timed out".to_string()),
    };

    let universe: UniverseResponse = tokio::time::timeout(Duration::from_secs(5), response.json())
        .await
        .map_err(|_| "apis.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?;

    Ok(universe.universe_id)
}

async fn fetch_place_info(place_id: u64) -> Result<Option<(String, String)>, String> {
    let client = get_client();

    let universe_id = fetch_universe_id(place_id).await?;

    let (games_res, icon_res) = tokio::join!(
        tokio::time::timeout(
            Duration::from_secs(5),
            client
                .get(format!(
                    "https://games.roblox.com/v1/games?universeIds={}",
                    universe_id
                ))
                .send()
        ),
        tokio::time::timeout(
            Duration::from_secs(5),
            client
                .get(format!(
                    "https://thumbnails.roblox.com/v1/games/icons?universeIds={}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false",
                    universe_id
                ))
                .send()
        ),
    );

    let games_response = games_res
        .map_err(|_| "games.roblox.com request timed out".to_string())?
        .map_err(|e| e.to_string())?;

    let icon_response = icon_res
        .map_err(|_| "thumbnails.roblox.com request timed out".to_string())?
        .map_err(|e| e.to_string())?;

    let (games_data, icon_data) = tokio::join!(
        tokio::time::timeout(Duration::from_secs(5), games_response.json::<GamesResponse>()),
        tokio::time::timeout(Duration::from_secs(5), icon_response.json::<IconResponse>()),
    );

    let name = games_data
        .map_err(|_| "games.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?
        .data
        .into_iter()
        .next()
        .map(|g| g.name)
        .unwrap_or_else(|| "Unknown Game".to_string());

    let image_url = icon_data
        .map_err(|_| "thumbnails.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?
        .data
        .into_iter()
        .next()
        .map(|i| i.image_url)
        .unwrap_or_default();

    Ok(Some((name, image_url)))
}
