use std::fs;
use std::path::PathBuf;
extern crate dirs;

#[tauri::command]
pub async fn get_gbs(vng: Option<bool>) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let mut roblox_path: PathBuf =
        dirs::data_local_dir().ok_or("Could not find local data directory")?;

    #[cfg(target_os = "macos")]
    let mut roblox_path: PathBuf = home().ok_or("Could not find local data directory")?;

    roblox_path.push(if vng.unwrap_or(false) {
        "RobloxPCVNG"
    } else {
        "Roblox"
    });
    roblox_path.push("GlobalBasicSettings_13.xml");

    fs::read_to_string(&roblox_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_gbs(content: String, vng: Option<bool>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut roblox_path: PathBuf =
        dirs::data_local_dir().ok_or("Could not find local data directory")?;

    #[cfg(target_os = "macos")]
    let mut roblox_path: PathBuf = home().ok_or("Could not find local data directory")?;

    roblox_path.push(if vng.unwrap_or(false) {
        "RobloxPCVNG"
    } else {
        "Roblox"
    });
    roblox_path.push("GlobalBasicSettings_13.xml");

    // log::info!("contents : {}", content);

    fs::write(roblox_path, content).map_err(|e| e.to_string())
}
