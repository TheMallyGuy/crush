use super::config::integration_enabled;
use super::line::handle_line;
use super::state::WatcherState;
use crate::rpc::{apply_rpc, RpcState};
use crate::simple_i18n::I18n;
#[cfg(target_os = "macos")]
use dirs::home_dir;
#[cfg(target_os = "windows")]
use dirs_next::data_local_dir;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub(super) fn get_latest_log(vng: bool) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let dir = data_local_dir()?
        .join(if vng { "RobloxPCVNG" } else { "Roblox" })
        .join("logs");

    #[cfg(target_os = "macos")]
    let dir = home_dir()?
        .join("Roblox")
        .join("logs");

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

pub(super) async fn maybe_switch_log_file(
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

    let i18n = app.state::<I18n>().inner().clone();

    if integration_enabled(store, &["discordRpc", "enable"]) {
        let _ = apply_rpc(
            &app.state::<RpcState>(),
            &i18n.t("rpc.rust.watcher.general"),
            &i18n.t("rpc.rust.watcher.idle"),
        )
        .await;
    } else {
        log::info!("Discord RPC integration disabled, skipping initial RPC set");
    }
}

pub(super) async fn read_new_lines(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
    is_vng: bool,
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
                if let Err(e) = handle_line(app, &line, state, store, is_vng).await {
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
