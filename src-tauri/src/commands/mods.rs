use std::{fs, path::PathBuf};

use md5;
use tauri::command;
use walkdir::WalkDir;

#[command]
pub async fn apply_mod(mod_dir: String, version_dir: String) -> Vec<String> {
    let mod_dir = PathBuf::from(&mod_dir);
    let version_dir = PathBuf::from(&version_dir);
    let mut copied = Vec::new();

    for entry in WalkDir::new(&mod_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let src = entry.path();
        let relative = match src.strip_prefix(&mod_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let dest = version_dir.join(relative);

        if dest.exists() && md5_file(src) == md5_file(&dest) {
            copied.push(relative.to_string_lossy().to_string());
            continue;
        }

        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if fs::copy(src, &dest).is_ok() {
            copied.push(relative.to_string_lossy().to_string());
        }
    }

    copied
}

fn md5_file(path: &std::path::Path) -> String {
    let Ok(bytes) = fs::read(path) else {
        return String::new();
    };
    format!("{:x}", md5::compute(&bytes))
}
