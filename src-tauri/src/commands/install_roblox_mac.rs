// this thing is macos only

use std::fs;
use std::path::{Path, PathBuf};
use tauri::command;

#[command]
pub async fn move_app_to_applications(
    source_path: String,
    dest_name: String,
) -> Result<(), String> {
    let source = PathBuf::from(&source_path);

    if !source.exists() {
        return Err(format!("Source app not found at: {}", source_path));
    }

    let applications_dir = if is_writable(Path::new("/Applications")) {
        PathBuf::from("/Applications")
    } else {
        let home =
            std::env::var("HOME").map_err(|_| "Could not resolve HOME directory".to_string())?;
        let user_apps = PathBuf::from(home).join("Applications");

        if !user_apps.exists() {
            fs::create_dir_all(&user_apps)
                .map_err(|e| format!("Failed to create {}: {}", user_apps.display(), e))?;
        }
        user_apps
    };

    let dest_path = applications_dir.join(&dest_name);

    if dest_path.exists() {
        fs::remove_dir_all(&dest_path).map_err(|e| {
            format!(
                "Failed to remove existing app at {}: {}",
                dest_path.display(),
                e
            )
        })?;
    }

    if fs::rename(&source, &dest_path).is_err() {
        copy_dir_recursive(&source, &dest_path)
            .map_err(|e| format!("Failed to copy app to {}: {}", dest_path.display(), e))?;
        fs::remove_dir_all(&source).ok();
    }

    Ok(())
}

fn is_writable(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }
    let test_file = dir.join(".rblx_write_test");
    match fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            true
        }
        Err(_) => false,
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
