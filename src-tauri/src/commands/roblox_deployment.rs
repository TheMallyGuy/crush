use crate::rd::{best_region, get_download_urls, latest_version};
// @pochita hey! can you add error handling to this? when the download is failed/paused its its do something in the frontend and then prompt user to reinstall (or just do it anyway best) that would be nice thanks

#[tauri::command]
pub async fn get_download_deployment_urls(region: Option<&str>) -> Result<Vec<String>, String> {
    let urls = get_download_urls(None, region)
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
    latest_version()
        .await
        .map(|v| v.client_version_upload)
        .map_err(|e| e.to_string())
}
