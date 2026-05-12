use commands::archive::{extract_files_from_zip, extract_zip};
use commands::boostrapper_importer::export_boostrapconfig;
use commands::crush::crush;
use commands::discord_rpc::set_rpc;
use commands::fs::copy_file;
use commands::launch_roblox::launch;
use commands::mods::apply_mod;
use commands::rename::rename;
use commands::roblox_deployment::{
    get_best_region, get_download_deployment_urls, get_latest_version_player,
    get_latest_version_studio,
};
use commands::watcher::watch_logs;
use commands::window::{
    apply_vibrancy_to_window, create_or_focus_window, kill_window, set_window_vibrancy,
};
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_store::StoreExt;
mod commands;
use crate::rpc::kill_rpc;
use rpc::RpcState;
use simple_i18n::I18n;

pub mod interactive;
pub mod rd;
pub mod rpc;
pub mod tray;
pub mod simple_i18n;

use crate::tray::setup_tray;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn register_plugins<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let is_deep_link = args
                .iter()
                .any(|a| a.starts_with("roblox-player:") || a.starts_with("roblox:"));

            if is_deep_link {
                return;
            }

            app.dialog()
                .message("The app is already running! Look for it in your system tray.")
                .kind(MessageDialogKind::Info)
                .title("Already Running")
                .blocking_show();
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_fs_pro::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
}

fn handle_received_url(app_handle: &tauri::AppHandle, url: String) {
    app_handle.emit("deep-link-received", url.clone()).ok();

    if let Some(win) = app_handle.get_webview_window("crushBoostrapChoiceWindow") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    log::info!("{}", url);
}

fn setup_deep_links(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.deep_link().register_all()?;

    let app_handle = app.handle().clone();
    app.deep_link().on_open_url(move |event| {
        if let Some(url) = event.urls().first() {
            handle_received_url(&app_handle, url.to_string());
        }
    });

    if let Ok(Some(urls)) = app.deep_link().get_current() { // store bef emit
        if let Some(url) = urls.first() {
            let url_str = url.to_string();
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                handle_received_url(&app_handle, url_str);
            });
        }
    }

    Ok(())
}

fn spawn_discord_rpc(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state: tauri::State<'_, RpcState> = app_handle.state::<RpcState>();

        if let Err(e) = crate::rpc::start_rpc(&state, "363445589247131668").await {
            log::error!("RPC error: {}", e);
        };
    });
}

fn print_debug_info() {
    log::info!("Debug Info:");
    log::info!("OS: {}", tauri_plugin_os::platform());
    log::info!("Git hash: {}", env!("VERGEN_RUSTC_COMMIT_HASH"));
    log::info!("Build date: {}", env!("VERGEN_BUILD_DATE"));
    log::info!("Build timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_clipboard_manager::init());

    builder = register_plugins(builder);

    builder
        .manage(RpcState::new())
        .setup(|app| {
            print_debug_info();

            let platform = tauri_plugin_os::platform();

            let locale = app
                .get_store("config.json")
                .and_then(|store| store.get("language"))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "en-US".to_string());

            let path = app.path().resolve("resources/locales", tauri::path::BaseDirectory::Resource)?;

            let i18n = I18n::new(path, &locale).unwrap();

            app.manage(i18n);

            if platform != "windows" {
                app.dialog()
                    .message(format!(
                        "This app can't work on {}. However, we will have plans for {}.",
                        platform, platform
                    ))
                    .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                    .title("Error")
                    .blocking_show();
                std::process::exit(1);
            }

            let Some(window) = app.get_webview_window("crushBoostrapChoiceWindow") else {
                return Err("Failed to find main bootstrap choice window".into());
            };

            let effect = app
                .get_store("config.json")
                .and_then(|store| store.get("vibrancy"))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "auto".to_string());

            apply_vibrancy_to_window(&window, &effect);

            setup_deep_links(app)?;
            spawn_discord_rpc(app.handle().clone());
            setup_tray(app)?; // <-- now lives in tray.rs, manages TrayState

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "CrushBoostrap" || window.label() == "crushBoostrap" {
                    return;
                }

                api.prevent_close();
                let _ = window.hide();

                let app_handle = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_handle.state::<RpcState>();
                    let _ = kill_rpc(&state).await;
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            kill_window,
            get_download_deployment_urls,
            get_best_region,
            create_or_focus_window,
            extract_zip,
            extract_files_from_zip,
            launch,
            get_latest_version_player,
            rename,
            apply_mod,
            crush,
            watch_logs,
            set_rpc,
            copy_file,
            export_boostrapconfig,
            set_window_vibrancy,
            get_latest_version_studio
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
