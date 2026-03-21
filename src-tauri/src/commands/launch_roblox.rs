use std::process::Command;

#[tauri::command]
pub async fn launch(path: String) -> Result<(), String> {
    Command::new(path)
        .spawn()
        .map_err(|e| format!("Failed to launch app: {}", e))?;
    Ok(())
}
