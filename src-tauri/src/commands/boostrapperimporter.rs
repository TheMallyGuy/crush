use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct BloxstrapConfig {
    CheckForUpdates: bool,
    BackgroundUpdatesEnabled: bool,
    UseDiscordRichPresence: bool,
    ShowServerDetails: bool,
}

#[tauri::command]
pub async fn export_boostrapconfig(boostrap_config_path: String) -> Result<(), String> {
    let path = PathBuf::from(boostrap_config_path).join("Settings.json");

    let data = fs::read_to_string(&path)
        .map_err(|e| e.to_string())?;

    let config: BloxstrapConfig = serde_json::from_str(&data)
        .map_err(|e| e.to_string())?;

    // print (debug)
    println!("{:?}", config.BackgroundUpdatesEnabled);

    Ok(())
}