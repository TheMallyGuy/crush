use std::{fs, io::Read, path::PathBuf};

use md5;
use tauri::command;
use walkdir::WalkDir;

#[command]
pub async fn apply_mod(mod_dir: String, version_dir: String) -> Vec<String> {
    let mod_dir = PathBuf::from(&mod_dir);
    let version_dir = PathBuf::from(&version_dir);

    WalkDir::new(&mod_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let src = entry.path();
            let relative = src.strip_prefix(&mod_dir).ok()?;
            let dest = version_dir.join(relative);
            let rel_str = relative.to_string_lossy().to_string();

            if is_file_up_to_date(src, &dest) {
                return Some(rel_str);
            }

            let parent = dest.parent()?;
            fs::create_dir_all(parent).ok()?;

            fs::copy(src, &dest).ok().map(|_| rel_str)
        })
        .collect()
}

fn is_file_up_to_date(src: &std::path::Path, dest: &std::path::Path) -> bool {
    dest.exists()
        && md5_file(src)
            .zip(md5_file(dest))
            .is_some_and(|(s, d)| s == d)
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
