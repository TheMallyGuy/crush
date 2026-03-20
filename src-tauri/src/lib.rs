// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use commands::roblox_deployment::{get_best_region, get_download_deployment_urls};
use commands::window::{create_or_focus_window, kill_window};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use commands::archive::extract_zip;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
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

            let mut client  = DiscordIpcClient::new("1484521125550620813");
            client.connect()?;
            client.set_activity(activity::Activity::new()
                .state("Playing")
                .details("Crush")
            )?;


            Ok(())
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
            extract_zip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
