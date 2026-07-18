use sysinfo::System;

#[tauri::command]
pub async fn kill_roblox_if_open() -> Result<(), String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    #[cfg(target_os = "windows")]
    let name = "RobloxPlayerBeta.exe";

    #[cfg(target_os = "macos")]
    let name = "Roblox";

    for process in sys.processes_by_name(name.as_ref()) {
        process.kill();
    }

    Ok(())
}
