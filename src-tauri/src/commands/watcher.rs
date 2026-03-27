use dirs_next::data_local_dir;
use reqwest;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    time::Duration,
};
use tauri::AppHandle;
use tauri::Manager;
use crate::rpc::{RpcState, apply_rpc};
use serde::Deserialize;

#[tauri::command]
pub fn watch_logs(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_watcher(app).await {
            log::error!("watcher error: {}", e);
        }
    });
    Ok(())
}

fn get_latest_log() -> Option<PathBuf> {
    let log_dir = data_local_dir()?.join("Roblox").join("logs");

    std::fs::read_dir(log_dir)
        .ok()?
        .flatten()
        .filter(|e| {
            e.path().extension().map(|x| x == "log").unwrap_or(false)
        })
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}
#[derive(Deserialize)]
struct UniverseResponse {
    universeId: u64,
}

#[derive(Deserialize)]
struct GameData {
    name: String,
}

#[derive(Deserialize)]
struct GamesResponse {
    data: Vec<GameData>,
}

async fn run_watcher(app: AppHandle) -> Result<(), reqwest::Error> {
    let re = Regex::new(r"Joining game '.*?' place (\d+)").unwrap();
    let mut current_path: Option<PathBuf> = None;
    let mut offset: u64 = 0;

    loop {
        // check if there's a newer log file (new Roblox session started)
        if let Some(latest) = get_latest_log() {
            if current_path.as_ref() != Some(&latest) {
                log::info!("Watching new log file: {:?}", latest);
                current_path = Some(latest);
                offset = 0; // reset offset for new file
            }
        }

        if let Some(ref path) = current_path {
            if let Ok(mut file) = File::open(path) {
                if file.seek(SeekFrom::Start(offset)).is_ok() {
                    let mut reader = BufReader::new(&mut file);
                    let mut line = String::new();

                    loop {
                        line.clear();
                        match reader.read_line(&mut line) {
                            Ok(0) => break, // no new content yet
                            Ok(_) => {
                                if let Some(caps) = re.captures(&line) {
                                    let place_id = caps[1].to_string();
                                    log::info!("new join: {}", place_id);

                                    let res = reqwest::get(format!(
                                        "https://apis.roblox.com/universes/v1/places/{}/universe",
                                        place_id
                                    ))
                                    .await?;

                                    let universe: UniverseResponse = res.json().await?;
                                    let universe_id = universe.universeId;

                                    let res2 = reqwest::get(format!(
                                        "https://games.roblox.com/v1/games?universeIds={}",
                                        universe_id
                                    ))
                                    .await?;

                                    let games: GamesResponse = res2.json().await?;

                                    if let Some(game) = games.data.first() {
                                        let place_name = game.name.clone();
                                        log::info!("Game name: {}", place_name);

                                        let state = app.state::<RpcState>();
                                        if let Err(e) = apply_rpc(&state.client, "Playing roblox", &place_name).await {
                                            log::error!("failed to set RPC: {}", e);
                                        }
                                    }

                                }
                            }
                            Err(_) => break,
                        }
                    }

                    // save true byte position
                    if let Ok(pos) = reader.stream_position() {
                        offset = pos;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}