#[cfg(target_os = "windows")]
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::fs;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct BloxstrapConfig {
    pub CheckForUpdates: bool,
    pub BackgroundUpdatesEnabled: bool,
    pub UseDiscordRichPresence: bool,
    pub ShowServerDetails: bool,
    pub ShowAccountOnRichPresence: bool, // show account
    pub HideRPCButtons: bool,            // allow joining server

    // frostrap only
    pub EnableBetterMatchmaking: bool,
    pub ShowUsingFroststrapRPC: bool,
    pub AutoCloseCrashHandler: bool,

    pub UpdateRoblox: bool, // frost & void??? (ps : fish)

    // voidstrap only
    pub VoidRPC: bool,
    pub DisableCrash: bool,

    // funkstrap
    pub UseWindowControl: bool,
    pub MoveWindowAllowed: bool,
    pub TitleControlAllowed: bool,
    pub WindowTransparencyAllowed: bool,
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn export_boostrapconfig(
    boostrap_config_path: String,
) -> Result<BloxstrapConfig, String> {
    let base = PathBuf::from(&boostrap_config_path);

    let path = if base.join("Settings.json").exists() {
        base.join("Settings.json")
    } else if base.join("AppSettings.json").exists() {
        base.join("AppSettings.json")
    } else {
        return Err("config not found".to_string());
    };

    let data = fs::read_to_string(&path).map_err(|e: std::io::Error| e.to_string())?;
    let config: BloxstrapConfig =
        serde_json::from_str(&data).map_err(|e: serde_json::Error| e.to_string())?;

    Ok(config)
}
