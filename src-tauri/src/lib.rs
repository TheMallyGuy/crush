// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use commands::window::{kill_window, open_choice_window, open_main_window};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use window_vibrancy::*;
mod commands;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            open_main_window,
            open_choice_window,
            kill_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
