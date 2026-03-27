use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use dirs_next::data_local_dir;
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;

#[tauri::command]
pub fn watch_logs(app: AppHandle) -> Result<(), String> {
    std::thread::spawn(move || {
        if let Err(e) = watch_logs_internal(app) {
            eprintln!("watch error: {:?}", e);
        }
    });

    Ok(())
}

fn watch_logs_internal(app: AppHandle) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;

    let log_dir = data_local_dir()
        .expect("no local appdata")
        .join("Roblox")
        .join("logs");

    log::info!("watching: {:?}", log_dir);

    watcher.watch(&log_dir, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv_timeout(Duration::from_secs(15)) {
            Ok(res) => match res {
                Ok(event) => {
                    if let EventKind::Create(_) = event.kind {
                        for path in event.paths {
                            if let Some(ext) = path.extension() {
                                if ext == "log" {
                                    log::info!("found log! : {:?}", path);

                                    // send to frontend
                                    let _ = app.emit(
                                        "log-found",
                                        path.to_string_lossy().to_string()
                                    );

                                    return Ok(()); // stop after first log
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("watch error: {:?}", e),
            },
            Err(_) => {
                log::error!("timeouted");
                return Ok(());
            }
        }
    }
}