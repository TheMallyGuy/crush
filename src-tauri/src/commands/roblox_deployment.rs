use crate::rd::{best_region, get_download_urls};

#[tauri::command]
pub async fn get_download_deployment_urls() -> Result<Vec<String>, String> {
    let urls = get_download_urls(None, None)
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
