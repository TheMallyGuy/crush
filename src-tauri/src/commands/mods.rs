use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::PathBuf,
};

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
        let Ok(relative) = src.strip_prefix(&mod_dir) else {
            continue;
        };

        let dest = version_dir.join(relative);

        // Skip copying if the file already exists and has the same hash
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

/// Computes MD5 hash of a file using streaming I/O to maintain a low memory footprint.
fn md5_file(path: &std::path::Path) -> String {
    let Ok(file) = File::open(path) else {
        return String::new();
    };

    let mut reader = BufReader::new(file);
    let mut context = md5::Context::new();
    let mut buffer = [0u8; 8192];

    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        context.consume(&buffer[..n]);
    }

    format!("{:x}", context.finalize())
}
