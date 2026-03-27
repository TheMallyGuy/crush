// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use commands::archive::extract_zip;
use commands::launch_roblox::launch;
use commands::rename::rename;
use commands::roblox_deployment::{
    get_best_region, get_download_deployment_urls, get_latest_version_player,
};
use commands::mods::apply_mod;
use commands::window::{create_or_focus_window, kill_window};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri::{
  menu::{Menu, MenuItem},
  tray::{TrayIconBuilder, MouseButtonState, MouseButton, TrayIconEvent},
};
use commands::watcher::watch_logs;
use window_vibrancy::*;
mod commands;

pub mod rd;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .target(tauri_plugin_log::Target::new(
                tauri_plugin_log::TargetKind::Stdout
            ))
            .build()
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
        .setup(|app| {
            let platform: &str = tauri_plugin_os::platform();

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

            let mut client = DiscordIpcClient::new("1484521125550620813");
            client.connect()?;
            client.set_activity(activity::Activity::new().state("Playing").details("Crush"))?;

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("crushBoostrapChoiceWindow") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;


            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    window.hide().unwrap();
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
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
            watch_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
