use crate::rd::{get_download_url, best_region};

#[tauri::command]
pub async fn download_roblox() -> Result<String, String> {
    let url = get_download_url(None, None)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("download url : {}", url);

    Ok(url)
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