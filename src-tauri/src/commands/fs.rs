#[tauri::command]
pub async fn copy_file(from: String, to: String) -> Result<(), String> {
    use std::path::Path;

    if let Some(parent) = Path::new(&to).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    std::fs::copy(&from, &to)
        .map(|_| ())
        .map_err(|e| format!("Failed to copy file: {}", e))
}

#[tauri::command]
pub fn rename(name: String, new_name: String) -> Result<(), String> {
    std::fs::rename(&name, &new_name).map_err(|e| format!("Rename failed: {}", e))
}
