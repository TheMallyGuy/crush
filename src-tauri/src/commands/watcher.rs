use crate::rd::get_client;
use crate::rpc::{RpcState, apply_rpc, apply_rpc_full, kill_rpc, start_rpc};
use chrono::Utc;
use dirs_next::data_local_dir;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use crate::interactive::{set_transparency, find_windows_by_title, move_window,maximize_window,minimize_window,focus_window,get_window_rect,set_borderless,set_window_title,restore_window};
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
fn re_interactive() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\[InteractiveAPI\] (.+)").unwrap())
}


// states

#[derive(Default, Debug)]
struct Activity {
    place_id: Option<u64>,
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
    interactive: Option<bool>,
    roblox_hwnd: Option<HWND>,
}

impl WatcherState {
    fn reset_for_new_game(&mut self) {
        self.activity = Activity::default();
        self.udmux_handled = false;
        self.pending_server_ip = None;
        self.pending_server_location = None;
        self.location_notified = false;
        self.bloxstrap_rpc = None;
        self.interactive = None;
        self.roblox_hwnd = None;
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
    // small_image: Option<RichPresenceImage>,
    // large_image: Option<RichPresenceImage>,
}
 
#[derive(Deserialize)]
struct BloxstrapRpcMessage {
    data: RichPresence,
}

// interactive

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InteractiveMessage {
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

// line checker or sum idk 

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

        state.reset_for_new_game();
        state.activity.join_initiated = true;
        state.activity.place_id = Some(place_id);
        state.activity.instance_id = Some(instance_id);
        log::info!("joining place {} instance {}", place_id, state.activity.instance_id.as_deref().unwrap_or("?"));
        return Ok(());
    }

    // interactive api
    if let Some(caps) = re_interactive().captures(line) {
        if let Some(raw) = caps.get(1) {
            on_interactive(app, raw.as_str(), state, store).await?;
        }
    }

    // UDMUX (server IP)
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

    // fully joined server
    if re_joined().is_match(line) {
        on_joined(app, state, store).await?;
        return Ok(());
    }

    // bloxstrapRPC detected
    if let Some(caps) = re_bloxstrap_rpc().captures(line) {
        if let Some(raw) = caps.get(1) {
            on_bloxstrap_rpc(app, raw.as_str(), state, store).await?;
        }
    }

    // left
    if state.activity.in_game && re_leave().is_match(line) {
        log::info!("left game");
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

    if state.roblox_hwnd.is_some() {
        log::info!("cached Roblox HWND");
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

async fn on_bloxstrap_rpc(
    app: &AppHandle,
    raw: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    if !integration_enabled(store, &["discordRpc", "enable"]) {
        return Ok(());
    }
    
    log::info!("BloxstrapRPC raw: {}", raw);

    let msg: BloxstrapRpcMessage = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("BloxstrapRPC: failed to parse: {} raw debug : {}", e, raw);
            return Ok(());
        }
    };

    let rpc = msg.data;
    log::info!("BloxstrapRPC: {:?}", rpc);
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
        // large.as_deref(),
        None,
        None,
        // rpc.large_image.as_ref().and_then(|i| i.hover_text.as_deref()),
        // small.as_deref(),
        None,
        None,
    )
    .await
    .map_err(|e| format!("BloxstrapRPC apply failed: {}", e))
}


async fn on_interactive(
    app: &AppHandle,
    raw: &str,
    __state__: &mut WatcherState,
    _store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    if !integration_enabled(_store, &["interactive", "enable"]) {
        log::info!("InteractiveAPI: disabled, ignoring");
        return Ok(());
    }

    log::info!("InteractiveAPI raw: {}", raw);
    let msg: InteractiveMessage = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("InteractiveAPI parse failed: {} raw: {}", e, raw);
            return Ok(());
        }
    };
    log::info!("InteractiveAPI command: {}", msg.command);
    let Some(hwnd) = get_or_find_hwnd(__state__) else {
        log::warn!("InteractiveAPI: no Roblox window found");
        return Ok(());
    };

    match msg.command.as_str() {
        "info" => {
            app.notification()
                .builder()
                .title("This game uses InteractiveAPI!")
                .body("You can turn off InteractiveAPI anytime in the integrations tab.")
                .show()
                .map_err(|e| e.to_string())?;
        }
        "moveWindow" => {
            if !integration_enabled(_store, &["interactive", "scopes", "moveWindow"]) {
                log::info!("InteractiveAPI: moveWindow scope disabled");
                return Ok(());
            }
            let x = msg.data.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = msg.data.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let w = msg.data.get("width").and_then(|v| v.as_i64()).unwrap_or(800) as i32;
            let h = msg.data.get("height").and_then(|v| v.as_i64()).unwrap_or(600) as i32;

            move_window(hwnd, x, y, w, h);
        }

        "minimize" => {
            if !integration_enabled(_store, &["interactive", "scopes", "minimize"]) {
                log::info!("InteractiveAPI: minimize scope disabled");
                return Ok(());
            }
            minimize_window(hwnd);
        }

        "maximize" => {
            if !integration_enabled(_store, &["interactive", "scopes", "maximize"]) {
                log::info!("InteractiveAPI: maximize scope disabled");
                return Ok(());
            }
            maximize_window(hwnd);
        }

        "restore" => {
            if !integration_enabled(_store, &["interactive", "scopes", "restore"]) {
                log::info!("InteractiveAPI: restore scope disabled");
                return Ok(());
            }
            restore_window(hwnd);
        }

        "focus" => {
            if !integration_enabled(_store, &["interactive", "scopes", "focus"]) {
                log::info!("InteractiveAPI: focus scope disabled");
                return Ok(());
            }
            focus_window(hwnd);
        }

        "setTitle" => {
            if !integration_enabled(_store, &["interactive", "scopes", "setTitle"]) {
                log::info!("InteractiveAPI: setTitle scope disabled");
                return Ok(());
            }
            if let Some(title) = msg.data.get("title").and_then(|v| v.as_str()) {
                set_window_title(hwnd, title);
            }
        }

        "setBorderless" => {
            if !integration_enabled(_store, &["interactive", "scopes", "setBorderless"]) {
                log::info!("InteractiveAPI: setBorderless scope disabled");
                return Ok(());
            }
            let enabled = msg.data
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

                set_borderless(hwnd, enabled);
        }

        "setTransparency" => {
            if !integration_enabled(_store, &["interactive", "scopes", "transparencyScopes", "enabled"]) {
                log::info!("InteractiveAPI: transparency scope disabled");
                return Ok(());
            }
            let alpha = msg.data
                .get("value")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .unwrap_or(255);

            let min = get_transparency_bound(_store, "minTransparency", 0);
            let max = get_transparency_bound(_store, "maxTransparency", 255);
            let alpha = alpha.clamp(min, max);

            set_transparency(hwnd, alpha);
        }

        other => {
            log::warn!("InteractiveAPI: unknown command '{}'", other);
        }
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

    let show_account = integration_enabled(store, &["discordRpc", "displayAccount"]);
    let let_join     = integration_enabled(store, &["discordRpc", "letJoin"]);
    let _ = (show_account, let_join); // reserved for future use

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