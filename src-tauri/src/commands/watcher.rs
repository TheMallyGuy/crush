// @pochita for the love of god, please dont modify this file
use crate::rd::get_client;
use crate::rpc::{apply_rpc, kill_rpc, RpcState};
use dirs_next::data_local_dir;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
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
    let regex = ROBLOX_REGEX.get_or_init(|| Regex::new(r"(?i)robloxplayerbeta").unwrap());

    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::Always),
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
            state.reader = None;
            state.current_file = None;
            let _ = kill_rpc(&app.state::<RpcState>()).await;
        }

        let roblox_just_started = running && !was_running;
        was_running = running;

        if roblox_just_started || (running && state.current_file.is_none()) {
            if let Some(path) = get_latest_log() {
                update_watcher_file(&app, &mut state, path, &store).await;
            }
        }

        if state.current_file.is_some() {
            if let Err(e) = process_log_file(&app, &mut state, &store).await {
                log::error!("Error processing log file: {}", e);
            }
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

    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to open log file {:?}: {}", path, e);
            return;
        }
    };

    state.current_file = Some(path);
    state.offset = 0;
    state.reader = Some(BufReader::new(file));
    state.activity = Activity::default();
    state.udmux_handled = false;
    state.pending_server_ip = None;
    state.pending_server_location = None;

    let integrations = get_integrations(store);
    let should_rpc = integrations.as_ref().is_some_and(|v| {
        v.get("crushRpc")
            .and_then(|r| r.as_bool())
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
            Regex::new(r"! Joining game '([0-9a-f\-]+)' place (\d+) at ([0-9\.]+)").unwrap()
        })
    }
    fn joined() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| Regex::new(r"serverId: ([0-9\.]+)\|").unwrap())
    }
    fn leave() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| Regex::new(r"Time to disconnect replication data").unwrap())
    }
    fn udmux() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").unwrap()
        })
    }
}

#[derive(Default)]
struct WatcherState {
    current_file: Option<PathBuf>,
    offset: u64,
    reader: Option<BufReader<File>>,
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
    let mut reader = state.reader.take();
    let res = if let Some(ref mut r) = reader {
        process_lines(app, r, state, store).await
    } else {
        Ok(())
    };
    state.reader = reader;
    res
}

async fn process_lines(
    app: &AppHandle,
    reader: &mut BufReader<File>,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let mut line = String::new();
    while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        if let Err(e) = handle_log_line(app, &line, state, store).await {
            state.offset = reader.stream_position().map_err(|e| e.to_string())?;
            return Err(e);
        }
        line.clear();
    }
    state.offset = reader.stream_position().map_err(|e| e.to_string())?;
    Ok(())
}

async fn handle_log_line(
    app: &AppHandle,
    line: &str,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    if let Some(caps) = WatcherRegexes::join().captures(line) {
        process_join_line(caps, state);
        return Ok(());
    }

    if let Some(caps) = WatcherRegexes::udmux().captures(line) {
        return process_udmux_line(caps, state).await;
    }

    if WatcherRegexes::joined().is_match(line) {
        return handle_joined_event(app, state, store).await;
    }

    if WatcherRegexes::leave().is_match(line) {
        return handle_leave_event(app, state).await;
    }

    Ok(())
}

fn process_join_line(caps: regex::Captures<'_>, state: &mut WatcherState) {
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
}

async fn process_udmux_line(caps: regex::Captures<'_>, state: &mut WatcherState) -> Result<(), String> {
    if state.udmux_handled {
        return Ok(());
    }

    let Some(ip) = caps.get(1).map(|m| m.as_str()) else {
        return Ok(());
    };

    handle_udmux_event(ip, state).await?;
    state.udmux_handled = true;
    Ok(())
}

async fn handle_leave_event(app: &AppHandle, state: &mut WatcherState) -> Result<(), String> {
    if !state.activity.in_game {
        return Ok(());
    }

    log::info!("left game");
    state.activity = Activity::default();
    state.pending_server_ip = None;
    state.pending_server_location = None;
    let _ = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", "Not in game").await;
    Ok(())
}

async fn handle_udmux_event(
    ip: &str,
    state: &mut WatcherState,
) -> Result<(), String> {
    if state.activity.place_id.is_none() {
        log::info!("UDMUX fired but no place_id — skipping");
        return Ok(());
    }

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

    if state.activity.in_game || state.activity.notified {
        return Ok(());
    }

    state.activity.in_game = true;
    state.activity.notified = true;
    log::info!("joined game {}", place_id);

    save_game_history(state, store, place_id)?;

    let integrations = get_integrations(store);
    let Some(integrations) = integrations.as_ref() else {
        state.pending_server_ip = None;
        state.pending_server_location = None;
        return Ok(());
    };

    handle_server_notification(app, state, integrations)?;

    let should_rpc = integrations
        .get("crushRpc")
        .and_then(|r| r.as_bool())
        .unwrap_or(false);

    if should_rpc {
        update_rpc_if_needed(app, state, place_id).await?;
    }

    Ok(())
}

fn handle_server_notification(
    app: &AppHandle,
    state: &mut WatcherState,
    integrations: &Value,
) -> Result<(), String> {
    let should_notify = integrations
        .get("serverLocationNotifier")
        .and_then(|n| n.as_bool())
        .unwrap_or(false);

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
        .map_err(|e| e.to_string())?;

    Ok(())
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
    if state.last_rpc.is_some_and(|l| now.duration_since(l).as_secs() <= 2) {
        return Ok(());
    }

    let Some(name) = fetch_place_name(place_id).await? else {
        return Ok(());
    };

    apply_rpc(&app.state::<RpcState>(), "Playing Roblox", &name)
        .await
        .map_err(|e| e.to_string())?;

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