use tauri::{command, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WebviewWindow};
use window_vibrancy::{apply_blur, apply_mica, apply_acrylic};
use tauri_plugin_store::StoreExt;
use serde_json::Value;

pub fn apply_vibrancy_to_window(window: &WebviewWindow, effect: &str) {
    #[cfg(target_os = "windows")]
    {
        match effect {
            "acrylic" => {
                let _ = apply_acrylic(window, Some((20, 20, 20, 10)));
            }
            "mica" => {
                let _ = apply_mica(window, None);
            }
            "blur" => {
                let _ = apply_blur(window, Some((18, 18, 18, 125)));
            }
            _ => {
                // Default fallback logic
                if let Err(_) = apply_acrylic(window, Some((20, 20, 20, 10))) {
                    if let Err(_) = apply_mica(window, None) {
                        let _ = apply_blur(window, Some((18, 18, 18, 125)));
                    }
                }
            }
        }
    }
}

#[command]
pub async fn set_window_vibrancy(app: AppHandle, effect: String) -> Result<(), String> {
    for window in app.webview_windows().values() {
        apply_vibrancy_to_window(window, &effect);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[command]
pub async fn create_or_focus_window(
    app: AppHandle,
    label: String,
    url: String,
    title: String,
    width: f64,
    height: f64,
    min_width: Option<f64>,
    min_height: Option<f64>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let webview_url = WebviewUrl::App(url.parse().unwrap());

    let mut builder = WebviewWindowBuilder::new(&app, label, webview_url)
        .title(&title)
        .closable(true)
        .inner_size(width, height)
        .center()
        .decorations(false)
        .transparent(true);

    if let (Some(w), Some(h)) = (min_width, min_height) {
        builder = builder.min_inner_size(w, h);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    let effect = app.get_store("config.json")
        .and_then(|store| store.get("vibrancy"))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "auto".to_string());

    apply_vibrancy_to_window(&window, &effect);

    Ok(())
}

#[command]
pub async fn kill_window(app: AppHandle, window_name: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(window_name) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
