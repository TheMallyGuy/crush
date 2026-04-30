use crate::rd::get_client;
use crate::rpc::{RpcState, apply_rpc, apply_rpc_full, kill_rpc, start_rpc};
use chrono::Utc;
use dirs_next::data_local_dir;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use std::path::Path;
use crate::interactive::{set_transparency, find_windows_by_title, move_window, get_window_rect, set_window_title};
use std::sync::{OnceLock, atomic::{AtomicBool, Ordering}};
use std::thread::{self, sleep};
use std::{fs::File, io::{BufRead, BufReader, Seek, SeekFrom}, path::PathBuf, time::{Duration, Instant}};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

// regex

fn re_join() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"! Joining game '([0-9a-f\-]+)' place (\d+) at ([0-9\.]+)").unwrap())
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
    R.get_or_init(|| Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").unwrap())
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

#[derive(Default)]
struct WatcherState {
    current_file: Option<PathBuf>,
    reader: Option<BufReader<File>>,
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
}

impl WatcherState {
    fn reset_for_new_game(&mut self) {
        self.activity = Activity::default();
        self.udmux_handled = false;
        self.pending_server_ip = None;
        self.pending_server_location = None;
        self.location_notified = false;
        self.bloxstrap_rpc = None;
        self.roblox_hwnd = None;
        self.window_started = false;
    }

    fn reset_fully(&mut self) {
        *self = WatcherState::default();
    }
}

// API types

#[derive(Deserialize)] struct UniverseResponse { #[serde(alias = "universeId")] universe_id: u64 }
#[derive(Deserialize)] struct GameData { name: String }
#[derive(Deserialize)] struct GamesResponse { data: Vec<GameData> }
#[derive(Deserialize)] struct IpInfo { city: String, region: String }
#[derive(Deserialize)] struct IconEntry { #[serde(rename = "imageUrl")] image_url: String }
#[derive(Deserialize)] struct IconResponse { data: Vec<IconEntry> }

// bloxstrap rpc types

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RichPresence {
    details:     Option<String>,
    state:       Option<String>,
}

#[derive(Deserialize)]
struct BloxstrapRpcMessage {
    command: String,
    #[serde(default)]
    data: Value,
}

// entry

static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn watch_logs(app: AppHandle) -> Result<(), String> {
    if WATCHER_RUNNING.swap(true, Ordering::SeqCst) {
        log::warn!("ignoring duplicate watch_logs call");
        return Ok(());
    }
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_watcher(app).await {
            log::error!("watcher error: {}", e);
            WATCHER_RUNNING.store(false, Ordering::SeqCst);
        }
    });
    Ok(())
}

// loop

async fn run_watcher(app: AppHandle) -> Result<(), String> {
    let mut state = WatcherState::default();
    let mut system = System::new();
    let mut was_running = false;
    let store = app.store("config.json").map_err(|e| e.to_string())?;

    loop {
        let running = is_roblox_running(&mut system);

        if was_running && !running {
            // send StopWindow before fully resetting if we had a window session
            if state.window_started {
                if let Some(hwnd) = state.roblox_hwnd {
                    send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
                }
            }
            state.reset_fully();
            let _ = kill_rpc(&app.state::<RpcState>()).await;
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
    log::info!("New log file: {:?} (skipping {} bytes)", path, initial_offset);

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
    let Some(path) = state.current_file.as_ref() else { return; };

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
        Err(e) => { log::error!("open reader: {}", e); return; }
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
            Err(e) => { log::error!("read_line: {}", e); break; }
        }
    }

    state.offset = reader.stream_position().unwrap_or(state.offset);
    state.reader = Some(reader);
}

fn open_reader(state: &mut WatcherState) -> Result<BufReader<File>, String> {
    let path = state.current_file.as_ref().ok_or("No current file")?;
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(state.offset)).map_err(|e| e.to_string())?;
    Ok(BufReader::new(file))
}

// line handler

async fn handle_line(
    app: &AppHandle,
    line: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    // new join
    if let Some(caps) = re_join().captures(line) {
        let instance_id = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let place_id: u64 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);

        // stop any previous window session cleanly
        if state.window_started {
            if let Some(hwnd) = state.roblox_hwnd {
                send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
            }
        }

        state.reset_for_new_game();
        state.activity.join_initiated = true;
        state.activity.place_id = Some(place_id);
        state.activity.instance_id = Some(instance_id);
        log::info!("joining place {} instance {}", place_id, state.activity.instance_id.as_deref().unwrap_or("?"));
        return Ok(());
    }

    // BloxstrapRPC (now handles both RPC and window control)
    if let Some(caps) = re_bloxstrap_rpc().captures(line) {
        if let Some(raw) = caps.get(1) {
            on_bloxstrap_rpc(app, raw.as_str(), state, store).await?;
        }
    }

    // UDMUX
    if let Some(caps) = re_udmux().captures(line) {
        if !state.udmux_handled {
            if let Some(ip) = caps.get(1) {
                fetch_and_store_location(ip.as_str(), state).await?;
                state.udmux_handled = true;
                if state.activity.in_game && !state.location_notified {
                    send_location_notification(app, state, store).await?;
                }
            }
        }
        return Ok(());
    }

    // fully joined
    if re_joined().is_match(line) {
        on_joined(app, state, store).await?;
        return Ok(());
    }

    // left
    if state.activity.in_game && re_leave().is_match(line) {
        log::info!("left game");
        if state.window_started {
            if let Some(hwnd) = state.roblox_hwnd {
                send_bloxstrap_command(hwnd, "StopWindow", Value::Null);
            }
            state.window_started = false;
        }
        state.reset_for_new_game();
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
    let Some(place_id) = state.activity.place_id else { return Ok(()); };

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
                place_id // fallback to place_id if fetch fails
            }
        };

        state.activity.universe_id = Some(universe_id);

        let store_val = |key: &str| integration_enabled(store, &["interactive", "scopes", key]);
        if let Err(e) = write_game_permission_png(
            universe_id,
            store_val("moveWindow"),
            store_val("setTitle"),
            integration_enabled(store, &["interactive", "scopes", "transparencyScopes", "enabled"]),
            app,
        ) {
            log::warn!("failed to write game permission PNG: {}", e);
        }

        send_bloxstrap_command(hwnd, "StartWindow", Value::Null);
        state.window_started = true;
    } else {
        log::warn!("failed to cache Roblox HWND");
    }

    log::info!("joined game {}", place_id);
    save_game_history(state, store, place_id)?;
    send_location_notification(app, state, store).await?;

    if integration_enabled(store, &["discordRpc", "enable"]) {
        update_discord_rpc(app, state, place_id, store).await?;
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
        // rich presence
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

            let rpc_state = app.state::<RpcState>();
            const CLIENT_ID: &str = "1484521125550620813";

            if rpc_state.client.lock().await.is_none() {
                if let Err(e) = start_rpc(&rpc_state, CLIENT_ID).await {
                    log::error!("RPC start failed: {}", e);
                    return Ok(());
                }
            }

            apply_rpc_full(
                &rpc_state,
                rpc.details.as_deref(),
                rpc.state.as_deref(),
                None, None, None, None,
            )
            .await
            .map_err(|e| format!("BloxstrapRPC apply failed: {}", e))?;
        }

        "RequestWindowPermission" => {
            log::info!("BloxstrapRPC: RequestWindowPermission received (handled via PNG)");
        }

        "SetWindow" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                log::info!("BloxstrapRPC: SetWindow ignored, interactive disabled");
                return Ok(());
            }
            if !state.window_started {
                log::warn!("BloxstrapRPC: SetWindow received before StartWindow, ignoring");
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else {
                log::warn!("BloxstrapRPC: SetWindow — no HWND");
                return Ok(());
            };

            let x      = msg.data.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y      = msg.data.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let w      = msg.data.get("width").and_then(|v| v.as_i64()).unwrap_or(800) as i32;
            let h      = msg.data.get("height").and_then(|v| v.as_i64()).unwrap_or(600) as i32;


            let scale_w = msg.data.get("scaleWidth").and_then(|v| v.as_f64()).unwrap_or(1920.0);
            let scale_h = msg.data.get("scaleHeight").and_then(|v| v.as_f64()).unwrap_or(1080.0);

            // TODO: replace with actual screen resolution query if available
            let screen_w = 1920.0_f64;
            let screen_h = 1080.0_f64;

            let sx = (x as f64 * screen_w / scale_w).round() as i32;
            let sy = (y as f64 * screen_h / scale_h).round() as i32;
            let sw = (w as f64 * screen_w / scale_w).round() as i32;
            let sh = (h as f64 * screen_h / scale_h).round() as i32;

            move_window(hwnd, sx, sy, sw, sh);
        }

        "SetWindowTitle" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else { return Ok(()); };
            let title = msg.data.as_str().unwrap_or("Roblox");
            set_window_title(hwnd, title);
        }

        "SetWindowTransparency" => {
            if !integration_enabled(store, &["interactive", "enable"]) {
                return Ok(());
            }
            if !state.window_started {
                log::warn!("BloxstrapRPC: SetWindowTransparency received before StartWindow, ignoring");
                return Ok(());
            }
            let Some(hwnd) = get_or_find_hwnd(state) else { return Ok(()); };

            let t = msg.data.get("transparency")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);

            let alpha = (t * 255.0).round() as u8;

            let min = get_transparency_bound(store, "minTransparency", 0);
            let max = get_transparency_bound(store, "maxTransparency", 255);
            set_transparency(hwnd, alpha.clamp(min, max));
        }

        "StartWindow" => {
            if let Some(hwnd) = get_or_find_hwnd(state) {
                state.window_started = true;
                log::info!("BloxstrapRPC: StartWindow acknowledged");
            }
        }

        "StopWindow" => {
            state.window_started = false;
            log::info!("BloxstrapRPC: StopWindow acknowledged");
        }

        "ResetWindow" => {
            if !state.window_started {
                return Ok(());
            }

            log::info!("BloxstrapRPC: ResetWindow acknowledged");
        }

        other => {
            log::warn!("BloxstrapRPC: unknown command '{}'", other);
        }
    }

    Ok(())
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
        if on { [255, 255, 255, 255] } else { [0, 0, 0, 0] }
    };

    let mut pixels: Vec<u8> = Vec::with_capacity(12);
    pixels.extend_from_slice(&pixel(allow_control));
    pixels.extend_from_slice(&pixel(allow_title));
    pixels.extend_from_slice(&pixel(allow_transparency));

    write_png_rgba(&game_png_path, 3, 1, &pixels)?;
    log::info!(
        "wrote game permission PNG for {} -> {:?} (control={}, title={}, transparency={})",
        game_id, game_png_path, allow_control, allow_title, allow_transparency
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
            for i in 0..256 {
                let mut c = i as u32;
                for _ in 0..8 {
                    c = if c & 1 != 0 { 0xedb88320 ^ (c >> 1) } else { c >> 1 };
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

    // PNG signature
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // IHDR
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);  // bit depth
    ihdr.push(6);  // colour type: RGBA
    ihdr.push(0);  // compression
    ihdr.push(0);  // filter
    ihdr.push(0);  // interlace
    write_chunk(&mut out, b"IHDR", &ihdr);

    let mut raw: Vec<u8> = Vec::new();
    for row in 0..height as usize {
        raw.push(0); // filter type None
        raw.extend_from_slice(&rgba[row * width as usize * 4..(row + 1) * width as usize * 4]);
    }

    let mut zlib: Vec<u8> = Vec::new();
    zlib.push(0x78); // CMF
    zlib.push(0x01); // FLG (no dict, check bits)
    zlib.push(0x01); // BFINAL=1, BTYPE=00

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


fn send_bloxstrap_command(_hwnd: HWND, command: &str, data: Value) {
    let payload = serde_json::to_string(&json!({ "command": command, "data": data }))
        .unwrap_or_default();
    println!("[BloxstrapRPC] {}", payload);
}

async fn fetch_and_store_location(ip: &str, state: &mut WatcherState) -> Result<(), String> {
    if state.activity.place_id.is_none() {
        log::info!("UDMUX fired but no place_id, skipping");
        return Ok(());
    }
    log::info!("UDMUX IP: {}", ip);

    let info: IpInfo = get_client()
        .get(format!("https://ipinfo.io/{}/json", ip))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

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

    if let (Some(ip), Some(location)) = (state.pending_server_ip.take(), state.pending_server_location.take()) {
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
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let now = Instant::now();
    if state.last_rpc.is_some_and(|t| now.duration_since(t).as_secs() <= 2) {
        return Ok(());
    }

    let Some((name, _)) = fetch_place_info(place_id).await? else { return Ok(()); };

    let instance_id = state.activity.instance_id.as_deref().unwrap_or("");
    let buttons = vec![
        ("Join Server".to_string(), format!("https://deeplink.multicrew.dev?placeId={}&jobId={}", place_id, instance_id)),
        ("View Game".to_string(), format!("https://www.roblox.com/games/{}", place_id)),
    ];

    const CLIENT_ID: &str = "1484521125550620813";
    let rpc = app.state::<RpcState>();

    if rpc.client.lock().await.is_none() {
        if let Err(e) = start_rpc(&rpc, CLIENT_ID).await {
            log::error!("RPC start failed: {}", e);
            return Ok(());
        }
    }

    if let Err(e) = apply_rpc_full(&rpc, Some("Crush"), Some("Playing Roblox"), Some(&name), None, None, Some(buttons.clone())).await {
        log::warn!("RPC failed ({}), reconnecting…", e);
        *rpc.client.lock().await = None;
        *rpc.runner.lock().await = None;

        if let Err(e) = start_rpc(&rpc, CLIENT_ID).await {
            log::error!("RPC reconnect failed: {}", e);
            return Ok(());
        }
        if let Err(e) = apply_rpc_full(&rpc, Some("Crush"), Some("Playing Roblox"), Some(&name), None, None, Some(buttons)).await {
            log::error!("RPC retry failed: {}", e);
        }
    }

    state.last_rpc = Some(now);
    Ok(())
}

// helpers

fn get_transparency_bound(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    key: &str,
    default: u8,
) -> u8 {
    let v = store.get("integrations").or_else(|| store.get("intergrations"));
    let Some(root) = v else { return default };
    root.get("interactive")
        .and_then(|v| v.get("scopes"))
        .and_then(|v| v.get("transparencyScopes"))
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_u64())
        .map(|v| v.clamp(0, 255) as u8)
        .unwrap_or(default)
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

fn integration_enabled(store: &tauri_plugin_store::Store<tauri::Wry>, path: &[&str]) -> bool {
    let v = store.get("integrations").or_else(|| store.get("intergrations"));
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
    system.processes().values().any(|p| re.is_match(p.name().to_string_lossy().as_ref()))
}

fn get_latest_log() -> Option<PathBuf> {
    let dir = data_local_dir()?.join("Roblox").join("logs");
    std::fs::read_dir(dir).ok()?
        .filter_map(|e| {
            let e = e.ok()?;
            let path = e.path();
            if path.extension()? != "log" { return None; }
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
    let mut history: Vec<Value> = store.get("gameHistory")
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
    let universe: UniverseResponse = get_client()
        .get(format!("https://apis.roblox.com/universes/v1/places/{}/universe", place_id))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    Ok(universe.universe_id)
}

async fn fetch_place_info(place_id: u64) -> Result<Option<(String, String)>, String> {
    let client = get_client();

    let universe: UniverseResponse = client
        .get(format!("https://apis.roblox.com/universes/v1/places/{}/universe", place_id))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let uid = universe.universe_id;

    let (games_res, icon_res) = tokio::join!(
        client.get(format!("https://games.roblox.com/v1/games?universeIds={}", uid)).send(),
        client.get(format!("https://thumbnails.roblox.com/v1/games/icons?universeIds={}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false", uid)).send(),
    );

    let name = games_res.map_err(|e| e.to_string())?
        .json::<GamesResponse>().await.map_err(|e| e.to_string())?
        .data.into_iter().next().map(|g| g.name)
        .unwrap_or_else(|| "Unknown Game".to_string());

    let image_url = icon_res.map_err(|e| e.to_string())?
        .json::<IconResponse>().await.map_err(|e| e.to_string())?
        .data.into_iter().next().map(|i| i.image_url)
        .unwrap_or_default();

    Ok(Some((name, image_url)))
}