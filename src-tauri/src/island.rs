use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_notification::NotificationExt;

pub const OVERLAY_WINDOW_LABEL: &str = "crushOverlay";

#[derive(Clone, Serialize, Deserialize)]
pub struct IslandPayload {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
}

pub fn emit<R: Runtime>(app: &AppHandle<R>, payload: IslandPayload) {
    #[cfg(target_os = "windows")]
    if let Err(e) = app.emit_to(OVERLAY_WINDOW_LABEL, "dynamic-island", payload) {
        log::warn!("island: failed to emit to overlay window: {}", e);
    }

    #[cfg(target_os = "macos")]
    app.notification()
        .builder()
        .title(&payload.title.to_string())
        .body(&payload.description.unwrap_or_default())
        .show()
        .unwrap();
}

pub fn show<R: Runtime>(app: &AppHandle<R>, title: impl Into<String>, description: Option<&str>) {
    emit(
        app,
        IslandPayload {
            title: title.into(),
            description: description.map(str::to_owned),
            image: None,
            duration: None,
        },
    );
}

pub fn show_with_image<R: Runtime>(
    app: &AppHandle<R>,
    title: impl Into<String>,
    description: Option<&str>,
    image: impl Into<String>,
) {
    emit(
        app,
        IslandPayload {
            title: title.into(),
            description: description.map(str::to_owned),
            image: Some(image.into()),
            duration: None,
        },
    );
}

#[tauri::command]
pub fn island_show(app: AppHandle, payload: IslandPayload) {
    emit(&app, payload);
}
