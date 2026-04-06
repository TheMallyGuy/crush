use crate::rd::get_client;
use crate::rpc::{apply_rpc, kill_rpc, RpcState};
use dirs_next::data_local_dir;
use regex::Regex;
use serde::Deserialize;
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
use serde_json::{json, Value};

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
    system.refresh_processes(ProcessesToUpdate::All, true);

    system.processes().values().any(|p| {
        p.name()
            .to_string_lossy()
            .to_lowercase()
            .contains("robloxplayerbeta")
    })
}

fn get_latest_log() -> Option<PathBuf> {
    let dir = data_local_dir()?.join("Roblox").join("logs");

    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "log").unwrap_or(false))
        .filter(|e| {
            e.metadata()
                .and_then(|m| m.created())
                .map(|t| t.elapsed().unwrap_or_default().as_secs() < 20)
                .unwrap_or(false)
        })
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

async fn run_watcher(app: AppHandle) -> Result<(), String> {
    let regexes = WatcherRegexes::new();
    let mut state = WatcherState::default();
    let mut system = System::new();
    let mut was_running = false;

    let store = app.store("config.json").map_err(|e| e.to_string())?;

    loop {
        let running = is_roblox_running(&mut system);
        if was_running && !running {
            state.activity = Activity::default();
            let rpc_state = app.state::<RpcState>();
            let _ = kill_rpc(&rpc_state).await;
        }
        was_running = running;

        if let Some(path) = get_latest_log() {
            if state.current_file.as_ref() != Some(&path) {
                log::info!("New log file: {:?}", path);
                state.current_file = Some(path);
                state.offset = 0;
                state.activity = Activity::default();

                let should_rpc = store
                    .get("intergrations")
                    .is_some_and(|v| v.get("crushRpc").and_then(|r| r.as_bool()).unwrap_or(false));

                if should_rpc {
                    apply_rpc(
                        &app.state::<RpcState>(),
                        "Playing Roblox",
                        "Not in game",
                    )
                    .await
                    .ok();
                }
            }
        }

        if let Some(path) = state.current_file.clone() {
            if let Err(e) = process_log_file(&app, &path, &regexes, &mut state, &store).await {
                log::error!("Error processing log file: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

struct WatcherRegexes {
    join: Regex,
    joined: Regex,
    leave: Regex,
    udmux: Regex,
}

impl WatcherRegexes {
    fn new() -> Self {
        Self {
            join: Regex::new(r"! Joining game '([0-9a-f\-]+)' place (\d+) at ([0-9\.]+)").unwrap(),
            joined: Regex::new(r"serverId: ([0-9\.]+)\|").unwrap(),
            leave: Regex::new(r"Time to disconnect replication data").unwrap(),
            udmux: Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").unwrap(),
        }
    }
}

#[derive(Default)]
struct WatcherState {
    current_file: Option<PathBuf>,
    offset: u64,
    activity: Activity,
    last_rpc: Option<Instant>,
}

async fn process_log_file(
    app: &AppHandle,
    path: &PathBuf,
    regexes: &WatcherRegexes,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let Ok(mut file) = File::open(path) else {
        return Ok(());
    };

    if file.seek(SeekFrom::Start(state.offset)).is_err() {
        return Ok(());
    }

    let mut reader = BufReader::new(&mut file);
    let mut line = String::new();

    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        handle_log_line(app, &line, regexes, state, store).await?;
        line.clear();
    }

    state.offset = reader.stream_position().unwrap_or(state.offset);
    Ok(())
}

async fn handle_log_line(
    app: &AppHandle,
    line: &str,
    regexes: &WatcherRegexes,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    if let Some(caps) = regexes.join.captures(line) {
        let instance_id = caps[1].to_string();
        let place_id: u64 = caps[2].parse().unwrap_or(0);
        
        state.activity.place_id = Some(place_id);
        state.activity.instance_id = Some(instance_id);
        state.activity.in_game = false;
        log::info!("joining place {} instance {}", place_id, &state.activity.instance_id.as_deref().unwrap_or("?"));
    }

    if let Some(caps) = regexes.udmux.captures(line) {
        handle_udmux_event(app, caps.get(1).unwrap().as_str(), store).await?;
    }

    if regexes.joined.is_match(line) {
        handle_joined_event(app, state, store).await?;
    }

    if regexes.leave.is_match(line) && state.activity.in_game {
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

    let should_notify = store.get("intergrations").is_some_and(|v| {
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

    let should_rpc = store
        .get("intergrations")
        .is_some_and(|v| v.get("crushRpc").and_then(|r| r.as_bool()).unwrap_or(false));

    // @pochita maybe try add error hanlding here idk
    let mut history: Vec<Value> = store
        .get("gameHistory")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    let new_entry = json!({
        "place_id": place_id,
        "instance_id": state.activity.instance_id.clone().unwrap_or_default(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    history.push(new_entry);

    store.set("gameHistory", serde_json::Value::Array(history));

    store.save().map_err(|e| e.to_string())?;

    if should_rpc {
        let now = Instant::now();
        let debounce_ok = state
            .last_rpc
            .is_none_or(|last| now.duration_since(last).as_secs() > 2);

        if debounce_ok {
            if let Some(name) = fetch_place_name(place_id).await? {
                if let Err(e) = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", &name).await {
                    log::error!("RPC failed: {}", e);
                }
                state.last_rpc = Some(now);
            }
        }
    }

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
