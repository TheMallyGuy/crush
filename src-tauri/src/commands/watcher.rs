use crate::rd::get_client;
use crate::rpc::{apply_rpc, kill_rpc, RpcState};
use dirs_next::data_local_dir;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

use std::sync::atomic::{AtomicBool, Ordering};

static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn watch_logs(app: AppHandle) -> Result<(), String> {
    if WATCHER_RUNNING.swap(true, Ordering::SeqCst) {
        log::warn!("ignoring duplicate watch logs");
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

#[derive(Default, Debug)]
struct Activity {
    place_id: Option<u64>,
    instance_id: Option<String>,
    in_game: bool,
    notified: bool,
    join_initiated: bool,
}

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
struct IpInfo {
    #[serde(rename = "ip")]
    _ip: String,
    city: String,
    region: String,
}

#[derive(Deserialize)]
struct GamesResponse {
    data: Vec<GameData>,
}

fn is_roblox_running(system: &mut System) -> bool {
    static ROBLOX_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = ROBLOX_REGEX.get_or_init(|| {
        Regex::new(r"(?i)robloxplayerbeta").expect("Failed to compile ROBLOX_REGEX")
    });

    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing(),
    );

    system
        .processes()
        .values()
        .any(|p| regex.is_match(p.name().to_string_lossy().as_ref()))
}

fn get_latest_log() -> Option<PathBuf> {
    let dir = data_local_dir()?.join("Roblox").join("logs");

    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|res| {
            let entry = res.ok()?;
            let path = entry.path();

            if path.extension().is_none_or(|ext| ext != "log") {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            Some((path, metadata))
        })
        .max_by_key(|(_, metadata)| metadata.modified().ok())
        .map(|(path, _)| path)
}

fn get_integrations(store: &tauri_plugin_store::Store<tauri::Wry>) -> Option<Value> {
    store.get("integrations").or_else(|| store.get("intergrations"))
}

async fn run_watcher(app: AppHandle) -> Result<(), String> {
    let mut state = WatcherState::default();
    let mut system = System::new();
    let mut was_running = false;

    let store = app.store("config.json").map_err(|e| e.to_string())?;

    loop {
        let running = is_roblox_running(&mut system);
        if was_running && !running {
            state.activity = Activity::default();
            state.pending_server_ip = None;
            state.pending_server_location = None;
            state.current_file = None;
            state.reader = None;
            let _ = kill_rpc(&app.state::<RpcState>()).await;
        }
        was_running = running;
        if running {

            if let Some(path) = get_latest_log() {
                update_watcher_file(&app, &mut state, path, &store).await;
            }

        let Some(_) = state.current_file else {
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        };

        if let Err(e) = process_log_file(&app, &mut state, &store).await {
            log::error!("Error processing log file: {}", e);
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn update_watcher_file(
    app: &AppHandle,
    state: &mut WatcherState,
    path: PathBuf,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) {
    if state.current_file.as_ref() == Some(&path) {
        return;
    }

    log::info!("New log file: {:?}", path);
    state.current_file = Some(path);
    state.reader = None;
    state.offset = 0;
    state.activity = Activity::default();
    state.udmux_handled = false;
    state.pending_server_ip = None;
    state.pending_server_location = None;

    let integrations = get_integrations(store);
    let should_rpc = integrations.as_ref().is_some_and(|v| {
        v.get("discordRpc")
            .and_then(|r| r.get("enable"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false)
    });

    if should_rpc {
        let _ = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", "Not in game").await;
    }
}

struct WatcherRegexes;

impl WatcherRegexes {
    fn join() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            Regex::new(r"! Joining game '([0-9a-f\-]+)' place (\d+) at ([0-9\.]+)").expect("Failed to compile JOIN regex")
        })
    }
    fn joined() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| Regex::new(r"serverId: ([0-9\.]+)\|").expect("Failed to compile JOINED regex"))
    }
    fn leave() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| Regex::new(r"Time to disconnect replication data").expect("Failed to compile LEAVE regex"))
    }
    fn udmux() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").expect("Failed to compile UDMUX regex")
        })
    }
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
}

async fn process_log_file(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let mut reader = match get_reader(state) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut line = String::new();
    let mut process_result = Ok(());

    loop {
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if let Err(e) = handle_log_line(app, &line, state, store).await {
                    log::error!("Error handling log line: {}", e);
                    process_result = Err(e);
                    break;
                }
                line.clear();
            }
            Err(e) => {
                let err_msg = e.to_string();
                log::error!("Error reading log line: {}", err_msg);
                process_result = Err(err_msg);
                break;
            }
        }
    }

    state.offset = reader.stream_position().unwrap_or(state.offset);
    state.reader = Some(reader);

    process_result
}

fn get_reader(state: &mut WatcherState) -> Result<BufReader<File>, String> {
    if let Some(r) = state.reader.take() {
        return Ok(r);
    }

    let path = state.current_file.as_ref().ok_or("No current file")?;
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(state.offset))
        .map_err(|e| e.to_string())?;
    Ok(BufReader::new(file))
}

async fn handle_log_line(
    app: &AppHandle,
    line: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    if handle_join_game(line, state) {
        return Ok(());
    }

    if handle_udmux_address(line, state).await? {
        return Ok(());
    }

    if handle_game_joined(app, line, state, store).await? {
        return Ok(());
    }

    handle_game_leave(app, line, state).await?;

    Ok(())
}

fn handle_join_game(line: &str, state: &mut WatcherState) -> bool {
    let Some(caps) = WatcherRegexes::join().captures(line) else {
        return false;
    };

    let instance_id = caps
        .get(1)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let place_id: u64 = caps
        .get(2)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    state.activity.place_id = Some(place_id);
    state.activity.instance_id = Some(instance_id);
    state.activity.in_game = false;
    state.udmux_handled = false;
    state.pending_server_ip = None;
    state.pending_server_location = None;

    log::info!(
        "joining place {} instance {}",
        place_id,
        state.activity.instance_id.as_deref().unwrap_or("?")
    );
    true
}

async fn handle_udmux_address(line: &str, state: &mut WatcherState) -> Result<bool, String> {
    let Some(caps) = WatcherRegexes::udmux().captures(line) else {
        return Ok(false);
    };

    let Some(ip) = caps.get(1) else {
        return Ok(true);
    };
    if state.udmux_handled {
        return Ok(true);
    }

    handle_udmux_event(ip.as_str(), state).await?;
    state.udmux_handled = true;
    Ok(true)
}

async fn handle_game_joined(
    app: &AppHandle,
    line: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<bool, String> {
    if !WatcherRegexes::joined().is_match(line) {
        return Ok(false);
    }

    log::info!("joined regex matched: {}", line.trim());
    handle_joined_event(app, state, store).await?;
    Ok(true)
}

async fn handle_game_leave(
    app: &AppHandle,
    line: &str,
    state: &mut WatcherState,
) -> Result<(), String> {
    if !state.activity.in_game || !WatcherRegexes::leave().is_match(line) {
        return Ok(());
    }

    log::info!("left game");
    state.activity = Activity::default();
    state.pending_server_ip = None;
    state.pending_server_location = None;
    let _ = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", "Not in game").await;
    Ok(())
}

async fn handle_udmux_event(ip: &str, state: &mut WatcherState) -> Result<(), String> {
    let Some(_) = state.activity.place_id else {
        log::info!("UDMUX fired but no place_id — skipping");
        return Ok(());
    };

    log::info!("UDMUX IP: {}", ip);

    let res = get_client()
        .get(format!("https://ipinfo.io/{}/json", ip))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let info: IpInfo = res.json().await.map_err(|e| e.to_string())?;

    state.pending_server_ip = Some(ip.to_string());
    state.pending_server_location = Some(format!("{}, {}", info.city, info.region));

    Ok(())
}

async fn handle_joined_event(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let Some(place_id) = state.activity.place_id else {
        return Ok(());
    };

    if !state.activity.join_initiated {
        log::warn!("serverId line seen but no join was initiated what skipping (stale log?)");
        return Ok(());
    }

    if state.activity.in_game || state.activity.notified {
        return Ok(());
    }

    state.activity.in_game = true;
    state.activity.notified = true;
    log::info!("joined game {}", place_id);

    save_game_history(state, store, place_id)?;

    let integrations = get_integrations(store);

    handle_joined_notifications(app, state, integrations.as_ref())?;

    let should_rpc = integrations.as_ref().is_some_and(|v| {
        v.get("discordRpc")
            .and_then(|r| r.get("enable"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false)
    });

    if !should_rpc {
        return Ok(());
    }

    update_rpc_if_needed(app, state, place_id).await
}

fn handle_joined_notifications(
    app: &AppHandle,
    state: &mut WatcherState,
    integrations: Option<&Value>,
) -> Result<(), String> {
    let should_notify = integrations.is_some_and(|v| {
        v.get("serverLocationNotifier")
            .and_then(|n| n.as_bool())
            .unwrap_or(false)
    });

    if !should_notify {
        state.pending_server_ip = None;
        state.pending_server_location = None;
        return Ok(());
    }

    let (Some(ip), Some(location)) = (
        state.pending_server_ip.take(),
        state.pending_server_location.take(),
    ) else {
        return Ok(());
    };

    app.notification()
        .builder()
        .title("Connected to a server!")
        .body(format!("IP : {}\nLocation : {}", ip, location))
        .show()
        .map_err(|e| e.to_string())
}

fn save_game_history(
    state: &WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
    place_id: u64,
) -> Result<(), String> {
    let mut history: Vec<Value> = match store.get("gameHistory") {
        Some(v) if v.is_array() => v.as_array().cloned().unwrap_or_default(),
        Some(_) => {
            log::warn!("gameHistory is not an array, resetting");
            Vec::new()
        }
        None => Vec::new(),
    };

    history.push(json!({
        "place_id": place_id,
        "instance_id": state.activity.instance_id.as_deref().unwrap_or_default(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    store.set("gameHistory", Value::Array(history));
    store.save().map_err(|e| e.to_string())
}

async fn update_rpc_if_needed(
    app: &AppHandle,
    state: &mut WatcherState,
    place_id: u64,
) -> Result<(), String> {
    let now = Instant::now();
    let debounce_ok = state
        .last_rpc
        .is_none_or(|last| now.duration_since(last).as_secs() > 2);

    if !debounce_ok {
        return Ok(());
    }

    let Some(name) = fetch_place_name(place_id).await? else {
        return Ok(());
    };

    if let Err(e) = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", &name).await {
        log::error!("RPC failed: {}", e);
    }
    state.last_rpc = Some(now);

    Ok(())
}

async fn fetch_place_name(place_id: u64) -> Result<Option<String>, String> {
    let client = get_client();
    let res = client
        .get(format!(
            "https://apis.roblox.com/universes/v1/places/{}/universe",
            place_id
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let universe: UniverseResponse = res.json().await.map_err(|e| e.to_string())?;

    let res2 = client
        .get(format!(
            "https://games.roblox.com/v1/games?universeIds={}",
            universe.universe_id
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let games: GamesResponse = res2.json().await.map_err(|e| e.to_string())?;

    Ok(games.data.first().map(|g| g.name.clone()))
}