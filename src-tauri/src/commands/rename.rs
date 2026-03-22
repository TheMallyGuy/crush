#[tauri::command]
pub fn rename(name: String, new_name: String) -> Result<(), String> {
    std::fs::rename(&name, &new_name).map_err(|e| format!("Rename failed: {}", e))
}
