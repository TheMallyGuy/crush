use crate::rd::get_client;
use crate::rpc::{apply_rpc, kill_rpc, RpcState};
use dirs_next::data_local_dir;
use regex::Regex;
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    sync::OnceLock,
    time::{Duration, Instant},
};
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

static RE_JOIN: OnceLock<Regex> = OnceLock::new();
static RE_JOINED: OnceLock<Regex> = OnceLock::new();
static RE_LEAVE: OnceLock<Regex> = OnceLock::new();
static RE_UDMUX: OnceLock<Regex> = OnceLock::new();
static RE_ROBLOX: OnceLock<Regex> = OnceLock::new();

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

    let re = RE_ROBLOX.get_or_init(|| Regex::new(r"(?i)robloxplayerbeta").unwrap());

    system
        .processes()
        .values()
        .any(|p| re.is_match(&p.name().to_string_lossy()))
}

fn get_latest_log() -> Option<PathBuf> {
    let dir = data_local_dir()?.join("Roblox").join("logs");

    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.ends_with(".log"))
                .unwrap_or(false)
        })
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
    let re_join = RE_JOIN.get_or_init(|| {
        Regex::new(r"! Joining game '([0-9a-f\-]+)' place (\d+)")
            .expect("Failed to compile RE_JOIN")
    });
    let re_joined = RE_JOINED.get_or_init(|| {
        Regex::new(r"serverId: ([0-9\.]+)\|").expect("Failed to compile RE_JOINED")
    });
    let re_leave = RE_LEAVE.get_or_init(|| {
        Regex::new(r"Time to disconnect replication data").expect("Failed to compile RE_LEAVE")
    });
    let re_udmux = RE_UDMUX.get_or_init(|| {
        Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").expect("Failed to compile RE_UDMUX")
    });

    let mut current_file: Option<PathBuf> = None;
    let mut offset = 0;

    let mut activity = Activity::default();
    let mut last_rpc = Instant::now();

    let mut system = System::new();
    let mut was_running = false;

    let store = app.store("config.json").map_err(|e| e.to_string())?;

    loop {
        let running = is_roblox_running(&mut system);

        if was_running && !running {
            activity = Activity::default();

            let state = app.state::<RpcState>();
            let _ = kill_rpc(&state.client).await;
        }

        was_running = running;

        if let Some(path) = get_latest_log() {
            if current_file.as_ref() != Some(&path) {
                log::info!("New log file: {:?}", path);
                current_file = Some(path);
                offset = 0;
                activity = Activity::default();
            }
        }

        if let Some(ref path) = current_file {
            if let Ok(mut file) = File::open(path) {
                if file.seek(SeekFrom::Start(offset)).is_ok() {
                    let mut reader = BufReader::new(&mut file);
                    let mut line = String::new();

                    while reader.read_line(&mut line).unwrap_or(0) > 0 {
                        if let Some(caps) = re_join.captures(&line) {
                            let place_id: u64 = caps[2].parse().unwrap_or(0);

                            activity.place_id = Some(place_id);
                            activity.in_game = false;

                            log::info!("joining place {}", place_id);
                        }

                        if let Some(caps) = re_udmux.captures(&line) {
                            let ip = caps.get(1).unwrap().as_str().to_string();
                            log::info!("UDMUX IP: {}", ip);

                            let res = get_client()
                                .get(format!("https://ipinfo.io/{}/json", ip))
                                .send()
                                .await
                                .map_err(|e| e.to_string())?;
                            let infoip: IpInfo = res.json().await.map_err(|e| e.to_string())?;

                            let should_notify =
                                if let Some(integrations) = store.get("intergrations") {
                                    integrations
                                        .get("serverLocationNotifier")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false)
                                } else {
                                    false
                                };

                            if should_notify {
                                app.notification()
                                    .builder()
                                    .title("Connected to a server!")
                                    .body(format!(
                                        "IP : {} \nLocation : {}, {}",
                                        ip, infoip.city, infoip.region
                                    ))
                                    .show()
                                    .map_err(|e| e.to_string())?;
                            }
                        }

                        if re_joined.is_match(&line) {
                            if let Some(place_id) = activity.place_id {
                                if !activity.in_game {
                                    activity.in_game = true;

                                    log::info!("joined game {}", place_id);

                                    let should_rpc =
                                        if let Some(integrations) = store.get("intergrations") {
                                            integrations
                                                .get("crushRpc")
                                                .and_then(|v| v.as_bool())
                                                .unwrap_or(false)
                                        } else {
                                            false
                                        };

                                    if should_rpc {
                                        // debounce RPC
                                        if last_rpc.elapsed().as_secs() > 2 {
                                            if let Some(name) = fetch_place_name(place_id).await? {
                                                let state = app.state::<RpcState>();

                                                if let Err(e) = apply_rpc(
                                                    &state.client,
                                                    "Playing Roblox",
                                                    &name,
                                                )
                                                .await
                                                {
                                                    log::error!("RPC failed: {}", e);
                                                }

                                                last_rpc = Instant::now();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if re_leave.is_match(&line) && activity.in_game {
                            log::info!("left game");

                            activity = Activity::default();

                            let state = app.state::<RpcState>();
                            let _ = apply_rpc(&state.client, "Idle", "Not in game").await;
                        }

                        line.clear();
                    }

                    offset = reader.stream_position().unwrap_or(offset);
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
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
