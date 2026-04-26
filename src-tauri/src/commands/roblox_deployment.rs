use crate::rd::{best_region, get_download_urls, latest_version_player, latest_version_studio};

#[tauri::command]
pub async fn get_download_deployment_urls(
    player : bool,
    region: Option<&str>,
    version: Option<&str>,
) -> Result<Vec<String>, String> {
    let urls = get_download_urls(player, version, region)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("download urls: {:?}", urls);

    Ok(urls)
}

#[tauri::command]
pub async fn get_best_region() -> String {
    let url = best_region()
        .await
        .unwrap_or("https://setup.rbxcdn.com")
        .to_string();

    log::info!("best download url : {}", url);

    url
}

#[tauri::command]
pub async fn get_latest_version_player() -> Result<String, String> {
    latest_version_player()
        .await
        .map(|v| v.client_version_upload)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_latest_version_studio() -> Result<String, String> {
    latest_version_studio()
        .await
        .map(|v| v.client_version_upload)
        .map_err(|e| e.to_string())
}