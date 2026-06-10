use crate::swifttunnel_sdk::{ConnectOptions, OAuthPoll};
use crate::SdkState;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn start_browser_login(
    app: tauri::AppHandle,
    state: tauri::State<'_, SdkState>,
) -> Result<(), String> {
    let oauth_url = {
        let sdk = state.0.lock().map_err(|e| e.to_string())?;
        sdk.auth_start_oauth().map_err(|e| e.to_string())?
    };

    app.opener()
        .open_url(&oauth_url, None::<&str>)
        .map_err(|e| e.to_string())?;

    loop {
        let poll = {
            let sdk = state.0.lock().map_err(|e| e.to_string())?;
            sdk.auth_poll_oauth()
        };
        match poll {
            OAuthPoll::Complete => return Ok(()),
            OAuthPoll::Error => return Err("OAuth failed".into()),
            OAuthPoll::Waiting => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }
}

#[tauri::command]
pub async fn swift_cancel_auth(state: tauri::State<'_, SdkState>) -> Result<(), String> {
    let sdk = state.0.lock().map_err(|e| e.to_string())?;
    sdk.auth_cancel_oauth();
    Ok(())
}

#[tauri::command]
pub async fn swift_is_logged_in(state: tauri::State<'_, SdkState>) -> Result<bool, String> {
    let sdk = state.0.lock().map_err(|e| e.to_string())?;
    Ok(sdk.auth_is_logged_in())
}

#[tauri::command]
pub async fn swift_auth_logout(state: tauri::State<'_, SdkState>) -> Result<(), String> {
    let sdk = state.0.lock().map_err(|e| e.to_string())?;
    sdk.auth_sign_out();
    Ok(())
}

#[tauri::command]
pub async fn swift_fetch_servers(state: tauri::State<'_, SdkState>) -> Result<(), String> {
    let sdk = state.0.lock().map_err(|e| e.to_string())?;
    sdk.servers_fetch().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn swift_get_servers(state: tauri::State<'_, SdkState>) -> Result<String, String> {
    let sdk = state.0.lock().map_err(|e| e.to_string())?;
    sdk.servers_fetch().map_err(|e| e.to_string())?;
    sdk.servers_get_json()
        .ok_or_else(|| "No servers returned".into())
}

#[tauri::command]
pub async fn swift_servers_ping(
    region: &str,
    state: tauri::State<'_, SdkState>,
) -> Result<u32, String> {
    let sdk = state.0.lock().map_err(|e| e.to_string())?;
    sdk.servers_ping(region).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect(
    region: String,
    routing: bool,
    dynamic_assets: bool,
    country_ban: bool,
    state: tauri::State<'_, SdkState>,
) -> Result<(), String> {
    let sdk = state.0.lock().map_err(|e| e.to_string())?;

    let options = ConnectOptions {
        region,
        apps: vec!["robloxplayerbeta.exe".into()],
        asset_urls: if dynamic_assets {
            vec!["assetdelivery.roblox.com".into(), "*.rbxcdn.com".into()]
        } else {
            vec![]
        },
        enable_api_tunneling: routing || country_ban,
        enable_country_ban: country_ban,
        ..Default::default()
    };

    sdk.connect_ex(options).map_err(|e| e.to_string())?;

    Ok(())
}
