use tokio::process::Command;

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