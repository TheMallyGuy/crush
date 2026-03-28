use std::{fs, io::Read, path::PathBuf};

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

        if dest.exists() {
            let src_md5 = md5_file(src);
            let dest_md5 = md5_file(&dest);

            if src_md5.is_some() && src_md5 == dest_md5 {
                copied.push(relative.to_string_lossy().to_string());
                continue;
            }
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

/// Computes MD5 hash using an 8KB buffer to minimize peak RSS memory usage.
fn md5_file(path: &std::path::Path) -> Option<md5::Digest> {
    let mut file = fs::File::open(path).ok()?;
    let mut context = md5::Context::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        context.consume(&buffer[..n]);
    }

    Some(context.finalize())
}
