// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use commands::archive::{extract_files_from_zip, extract_zip};
use commands::crush::crush;
use commands::discord_rpc::set_rpc;
use commands::fs::copy_file;
use commands::launch_roblox::launch;
use commands::mods::apply_mod;
use commands::rename::rename;
use commands::roblox_deployment::{
    get_best_region, get_download_deployment_urls, get_latest_version_player,
};
use commands::watcher::watch_logs;
use commands::window::{create_or_focus_window, kill_window};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use window_vibrancy::*;
mod commands;
use rpc::RpcState;

pub mod rd;
pub mod rpc;

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

    if let Ok(Some(urls)) = app.deep_link().get_current() {
        if let Some(url) = urls.first() {
            handle_received_url(app.handle(), url.to_string());
        }
    }

    Ok(())
}

fn spawn_discord_rpc(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<RpcState>();

        if let Err(e) = crate::rpc::start_rpc(&state, "1484521125550620813").await {
            log::error!("RPC error: {}", e);
        };
    });
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let window = app
                    .get_webview_window("CrushMainWindow")
                    .or_else(|| app.get_webview_window("crushBoostrapChoiceWindow"));

                if let Some(window) = window {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
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
    let mut builder = tauri::Builder::default();

    builder = register_plugins(builder);

    builder
        .manage(RpcState::new())
        .setup(|app| {
            print_debug_info();

            let platform = tauri_plugin_os::platform();

            if platform != "windows" {
                app.dialog()
                    .message(format!("This app can't work on {}", platform))
                    .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                    .title("Error")
                    .blocking_show();
                std::process::exit(1);
            }

            let window = app.get_webview_window("crushBoostrapChoiceWindow").unwrap();
            let _ = apply_blur(&window, Some((18, 18, 18, 125)));

            setup_deep_links(app)?;
            spawn_discord_rpc(app.handle().clone());
            setup_tray(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            let tauri::WindowEvent::CloseRequested { api, .. } = event else {
                return;
            };

            // Secondary windows like "CrushBoostrap" should close and be destroyed normally.
            // The main Choice window (the tray entry point) and the config window should stay in the tray.
            if window.label() == "CrushBoostrap" || window.label() == "crushBoostrap" {
                return;
            }

            let _ = window.hide();
            api.prevent_close();
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
            copy_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
