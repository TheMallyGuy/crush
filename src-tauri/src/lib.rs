// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use tauri_plugin_dialog::{DialogExt};
use tauri::{Manager};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command] // THIS HAVE TO BE ASYNC otherwise its will freezes
async fn open_main_window(app: tauri::AppHandle) -> Result<(), String> {
    let label = "CrushMainWindow";

    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let url = tauri::WebviewUrl::App(
        "mainWin/firstStarted".parse().unwrap()
    );

    tauri::WebviewWindowBuilder::new(&app, label, url)
        .title("Crush")
        .closable(true)
        .inner_size(800.0, 600.0)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, open_main_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}