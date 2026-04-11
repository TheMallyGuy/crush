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
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub fn watch_logs(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_watcher(app).await {
            log::error!("watcher error: {}", e);
        }
    });
    Ok(())
}

#[derive(Default, Debug)]
struct Activity {
    place_id: Option<u64>,
    instance_id: Option<String>,
    in_game: bool,
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

    system.refresh_processes(ProcessesToUpdate::All, true);

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
            let _ = kill_rpc(&app.state::<RpcState>()).await;
        }
        was_running = running;

        if let Some(path) = get_latest_log() {
            update_watcher_file(&app, &mut state, path, &store).await;
        }

        if let Some(path) = state.current_file.clone() {
            if let Err(e) = process_log_file(&app, &path, &mut state, &store).await {
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
    state.current_file = Some(path);
    state.offset = 0;
    state.activity = Activity::default();

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
    activity: Activity,
    last_rpc: Option<Instant>,
    udmux_handled: bool,
}

async fn process_log_file(
    app: &AppHandle,
    path: &PathBuf,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;

    file.seek(SeekFrom::Start(state.offset))
        .map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(&mut file);
    let mut line = String::new();

    while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        handle_log_line(app, &line, state, store).await?;
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
        log::info!(
            "joining place {} instance {}",
            place_id,
            state.activity.instance_id.as_deref().unwrap_or("?")
        );
        return Ok(());
    }

    if let Some(caps) = WatcherRegexes::udmux().captures(line) {
        if !state.udmux_handled {
            if let Some(ip) = caps.get(1) {
                handle_udmux_event(app, ip.as_str(), store).await?;
                state.udmux_handled = true;
            }
        }
        return Ok(());
    }

    if WatcherRegexes::joined().is_match(line) {
        handle_joined_event(app, state, store).await?;
        return Ok(());
    }

    if WatcherRegexes::leave().is_match(line) && state.activity.in_game {
        log::info!("left game");
        state.activity = Activity::default();
        let _ = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", "Not in game").await;
    }

    Ok(())
}

async fn handle_udmux_event(
    app: &AppHandle,
    ip: &str,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    log::info!("UDMUX IP: {}", ip);

    let res = get_client()
        .get(format!("https://ipinfo.io/{}/json", ip))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let info: IpInfo = res.json().await.map_err(|e| e.to_string())?;

    let integrations = get_integrations(store);

    let should_notify = integrations.as_ref().is_some_and(|v| {
        v.get("serverLocationNotifier")
            .and_then(|n| n.as_bool())
            .unwrap_or(false)
    });

    if should_notify {
        app.notification()
            .builder()
            .title("Connected to a server!")
            .body(format!(
                "IP : {} \nLocation : {}, {}",
                ip, info.city, info.region
            ))
            .show()
            .map_err(|e| e.to_string())?;
    }

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

    if state.activity.in_game {
        return Ok(());
    }

    state.activity.in_game = true;
    log::info!("joined game {}", place_id);

    save_game_history(state, store, place_id)?;

    let integrations = get_integrations(store);
    let should_rpc = integrations.as_ref().is_some_and(|v| {
        v.get("crushRpc")
            .and_then(|r| r.as_bool())
            .unwrap_or(false)
    });

    if !should_rpc {
        return Ok(());
    }

    update_rpc_if_needed(app, state, place_id).await
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
