// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use commands::archive::extract_zip;
use commands::discord_rpc::set_rpc;
use commands::launch_roblox::launch;
use commands::mods::apply_mod;
use commands::rename::rename;
use commands::roblox_deployment::{
    get_best_region, get_download_deployment_urls, get_latest_version_player,
};
use commands::watcher::watch_logs;
use commands::window::{create_or_focus_window, kill_window};
use filthy_rich::DiscordIPC;
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
            tauri_plugin_log::Builder::default()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .build(),
        )
        .plugin(tauri_plugin_fs_pro::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
}

fn setup_deep_links(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.deep_link().register_all()?;

    let app_handle = app.handle().clone();
    app.deep_link().on_open_url(move |event| {
        let urls = event.urls();
        let Some(url) = urls.first() else { return };

        app_handle.emit("deep-link-received", url.to_string()).ok();

        if let Some(win) = app_handle.get_webview_window("crushBoostrapChoiceWindow") {
            let _ = win.show();
            let _ = win.set_focus();
        }
        log::info!("{}", url);
    });

    if let Ok(Some(urls)) = app.deep_link().get_current() {
        if let Some(url) = urls.first() {
            app.emit("deep-link-received", url.to_string()).ok();
        }
    }

    Ok(())
}

fn spawn_discord_rpc(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<RpcState>();

        let mut client = DiscordIPC::new("1484521125550620813")
            .on_ready(|data| println!("Connected to user: {}", data.user.username));

        if let Err(e) = client.run(true).await {
            eprintln!("RPC error: {:?}", e);
            return;
        }

        let mut lock = state.client.lock().await;
        *lock = Some(client);
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
                if let Some(window) = app.get_webview_window("crushBoostrapChoiceWindow") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    builder = register_plugins(builder);

    builder
        .manage(RpcState::new())
        .setup(|app| {
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
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            kill_window,
            get_download_deployment_urls,
            get_best_region,
            create_or_focus_window,
            extract_zip,
            launch,
            get_latest_version_player,
            rename,
            apply_mod,
            watch_logs,
            set_rpc
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
