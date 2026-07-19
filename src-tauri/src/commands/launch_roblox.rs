use tokio::process::Command;

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn launch(path: String, arguments: Option<Vec<String>>) -> Result<(), String> {
    let mut cmd = Command::new(&path);
    if let Some(args) = arguments {
        cmd.args(args);
    }
    cmd.spawn()
        .map_err(|e| format!("Failed to launch app: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn launch_mac_app(app_path: String, arguments: Vec<String>) -> Result<(), String> {
    let mut cmd = std::process::Command::new("open");
    cmd.arg("-a").arg(&app_path);
    if !arguments.is_empty() {
        cmd.arg("--args");
        cmd.args(&arguments);
    }
    cmd.spawn()
        .map_err(|e| format!("Failed to launch {}: {}", app_path, e))?;
    Ok(())
}
