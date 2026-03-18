use tauri::command;
use tauri::Manager;

#[command] // THIS HAVE TO BE ASYNC otherwise its will freezes
pub async fn open_main_window(app: tauri::AppHandle, url: &str) -> Result<(), String> {
    let label = "CrushMainWindow";

    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let url = tauri::WebviewUrl::App(url.parse().unwrap());

    tauri::WebviewWindowBuilder::new(&app, label, url)
        .title("Crush")
        .closable(true)
        .inner_size(1000.0, 600.0)
        .center()
        .decorations(false)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command] // THIS HAVE TO BE ASYNC otherwise its will freezes
pub async fn open_choice_window(app: tauri::AppHandle) -> Result<(), String> {
    let label = "crushBoostrapChoiceWindow"; // trying to be diffrent here | EDIT : dont

    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let url = tauri::WebviewUrl::App("mainWin/choiceWin".parse().unwrap());

    tauri::WebviewWindowBuilder::new(&app, label, url)
        .title("crushBoostrapChoiceWindow")
        .closable(true)
        .inner_size(500.0, 250.0)
        .min_inner_size(500.0, 250.0)
        .center()
        .decorations(false)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn kill_window(app: tauri::AppHandle, window_name: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(window_name) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
