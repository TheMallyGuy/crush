use crate::rd::get_client;
use crate::rpc::{RpcState, apply_rpc, apply_rpc_full, kill_rpc, start_rpc};
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
        Regex::new(r"(?i)robloxplayerbeta").expect("Failed to compile ROBLOX regex")
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

            if state.current_file.is_some() {
                if let Err(e) = process_log_file(&app, &mut state, &store).await {
                    log::error!("Error processing log file: {}", e);
                }
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
    let initial_offset = std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);

    log::info!("Skipping {} bytes of existing log content", initial_offset);

    state.current_file = Some(path);
    state.reader = None;
    state.offset = initial_offset;
    state.activity = Activity::default();
    state.udmux_handled = false;
    state.pending_server_ip = None;
    state.pending_server_location = None;
    state.location_notified = false;

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
    location_notified: bool,
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
    if let Some(caps) = WatcherRegexes::join().captures(line) {
        let instance_id = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let place_id: u64 = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        state.activity.join_initiated = true;
        state.activity.place_id = Some(place_id);
        state.activity.instance_id = Some(instance_id);
        state.activity.in_game = false;
        state.udmux_handled = false;
        state.pending_server_ip = None;
        state.pending_server_location = None;
        state.location_notified = false;

        log::info!(
            "joining place {} instance {}",
            place_id,
            state.activity.instance_id.as_deref().unwrap_or("?")
        );
        return Ok(());
    }

    if let Some(caps) = WatcherRegexes::udmux().captures(line) {
        let Some(ip) = caps.get(1) else { return Ok(()) };
        if state.udmux_handled {
            return Ok(());
        }

        handle_udmux_event(ip.as_str(), state).await?;
        state.udmux_handled = true;

        if state.activity.in_game && !state.location_notified {
            try_send_location_notification(app, state, store).await?;
        }

        return Ok(());
    }


    if WatcherRegexes::joined().is_match(line) {
        log::info!("joined regex matched: {}", line.trim());
        handle_joined_event(app, state, store).await?;
        return Ok(());
    }

    if state.activity.in_game && WatcherRegexes::leave().is_match(line) {
        log::info!("left game");
        state.activity = Activity::default();
        state.pending_server_ip = None;
        state.pending_server_location = None;
        state.location_notified = false;
        let _ = apply_rpc(&app.state::<RpcState>(), "Playing Roblox", "Not in game").await;
    }

    Ok(())
}

async fn try_send_location_notification(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let integrations = get_integrations(store);
    let should_notify = integrations.as_ref().is_some_and(|v| {
        v.get("serverLocationNotifier")
            .and_then(|n| n.as_bool())
            .unwrap_or(false)
    });

    if !should_notify {
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

    try_send_location_notification(app, state, store).await?;

    let integrations = get_integrations(store);

    let should_rpc = integrations.as_ref().is_some_and(|v| {
        v.get("discordRpc")
            .and_then(|r| r.get("enable"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false)
    });

    if !should_rpc {
        return Ok(());
    }

    let should_display_account = integrations.as_ref().is_some_and(|v| {
        v.get("discordRpc")
            .and_then(|r| r.get("displayAccount"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false)
    });

    let should_let_join = integrations.as_ref().is_some_and(|v| {
        v.get("discordRpc")
            .and_then(|r| r.get("letJoin"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false)
    });

    update_rpc_if_needed(app, state, place_id, should_display_account, should_let_join).await
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
    __state__: &mut WatcherState,
    place_id: u64,
    _should_display_account: bool,
    _should_let_join: bool,
) -> Result<(), String> {
    let now = Instant::now();
    let debounce_ok = __state__
        .last_rpc
        .is_none_or(|last| now.duration_since(last).as_secs() > 2);
    if !debounce_ok {
        return Ok(());
    }

    let Some((name, _image_url)) = fetch_place_info(place_id).await? else {
        return Ok(());
    };

    let instance_id = __state__.activity.instance_id.as_deref().unwrap_or("");
    let deeplink = format!(
        "https://deeplink.multicrew.dev?placeId={}&jobId={}",
        place_id, instance_id
    );
    let buttons: Vec<(String, String)> = vec![
        ("Join Server".to_string(), deeplink),
        (
            "View Game".to_string(),
            format!("https://www.roblox.com/games/{}", place_id),
        ),
    ];

    let rpc_state = app.state::<RpcState>();
    const CLIENT_ID: &str = "1484521125550620813";

    if rpc_state.client.lock().await.is_none() {
        log::info!("RPC not initialized, starting...");
        if let Err(e) = start_rpc(&rpc_state, CLIENT_ID).await {
            log::error!("RPC start failed: {}", e);
            return Ok(());
        }
    }

    if let Err(e) = apply_rpc_full(
        &rpc_state,
        Some("Crush"),
        Some("Playing Roblox"),
        Some(&name),
        None,
        None,
        Some(buttons.clone()),
    )
    .await
    {
        log::warn!("RPC failed ({}), reconnecting...", e);

        *rpc_state.client.lock().await = None;
        *rpc_state.runner.lock().await = None;

        if let Err(e) = start_rpc(&rpc_state, CLIENT_ID).await {
            log::error!("RPC reconnect failed: {}", e);
            return Ok(());
        }

        if let Err(e) = apply_rpc_full(
            &rpc_state,
            Some("Crush"),
            Some("Playing Roblox"),
            Some(&name),
            None,
            None,
            Some(buttons),
        )
        .await
        {
            log::error!("RPC retry failed: {}", e);
        }
    }

    __state__.last_rpc = Some(now);
    Ok(())
}

async fn fetch_place_info(place_id: u64) -> Result<Option<(String, String)>, String> {
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
    let uid = universe.universe_id;

    let (games_res, icon_res) = tokio::join!(
        client
            .get(format!(
                "https://games.roblox.com/v1/games?universeIds={}",
                uid
            ))
            .send(),
        client
            .get(format!(
                "https://thumbnails.roblox.com/v1/games/icons?universeIds={}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false",
                uid
            ))
            .send(),
    );

    let name = games_res
        .map_err(|e| e.to_string())?
        .json::<GamesResponse>()
        .await
        .map_err(|e| e.to_string())?
        .data
        .into_iter()
        .next()
        .map(|g| g.name)
        .unwrap_or_else(|| "Unknown Game".to_string());

    #[derive(Deserialize)]
    struct IconEntry { #[serde(rename = "imageUrl")] image_url: String }
    #[derive(Deserialize)]
    struct IconResponse { data: Vec<IconEntry> }

    let image_url = icon_res
        .map_err(|e| e.to_string())?
        .json::<IconResponse>()
        .await
        .map_err(|e| e.to_string())?
        .data
        .into_iter()
        .next()
        .map(|i| i.image_url)
        .unwrap_or_default();

    Ok(Some((name, image_url)))
}
