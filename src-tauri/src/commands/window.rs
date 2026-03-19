use tauri::{command, Manager, AppHandle, WebviewUrl, WebviewWindowBuilder};

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
        .decorations(false);

    if let (Some(w), Some(h)) = (min_width, min_height) {
        builder = builder.min_inner_size(w, h);
    }

    builder.build().map_err(|e| e.to_string())?;

    Ok(())
}


#[command]
pub async fn kill_window(app: AppHandle, window_name: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(window_name) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}