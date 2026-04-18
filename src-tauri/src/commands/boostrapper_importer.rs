use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct BloxstrapConfig {
    pub CheckForUpdates: bool,
    pub BackgroundUpdatesEnabled: bool,
    pub UseDiscordRichPresence: bool,
    pub ShowServerDetails: bool,
}

#[tauri::command]
pub async fn export_boostrapconfig(
    boostrap_config_path: String,
) -> Result<BloxstrapConfig, String> {
    let path = PathBuf::from(boostrap_config_path).join("Settings.json");
    let data = fs::read_to_string(&path).map_err(|e: std::io::Error| e.to_string())?;
    let config: BloxstrapConfig =
        serde_json::from_str(&data).map_err(|e: serde_json::Error| e.to_string())?;

    Ok(config)
}
