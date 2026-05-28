use std::{fs, io::Read, path::Path};

use md5;
use tauri::command;
use walkdir::WalkDir;

#[command]
pub async fn apply_mod(mod_dir: String, version_dir: String) -> Vec<String> {
    WalkDir::new(&mod_dir)
        .into_iter()
        .filter_map(|e| {
            let entry = e.ok()?;
            if !entry.file_type().is_file() {
                return None;
            }
            process_mod_entry(entry, &mod_dir, &version_dir)
        })
        .collect()
}

fn process_mod_entry(
    entry: walkdir::DirEntry,
    mod_dir: impl AsRef<Path>,
    version_dir: impl AsRef<Path>,
) -> Option<String> {
    let src = entry.path();
    let relative = src.strip_prefix(mod_dir).ok()?;
    let dest = version_dir.as_ref().join(relative);
    let rel_str = relative.to_string_lossy().into_owned();

    if is_file_up_to_date(src, &dest) {
        return Some(rel_str);
    }

    let parent = dest.parent()?;
    fs::create_dir_all(parent).ok()?;

    fs::copy(src, &dest).ok()?;
    Some(rel_str)
}

fn is_file_up_to_date(src: &Path, dest: &Path) -> bool {
    dest.exists()
        && md5_file(src)
            .zip(md5_file(dest))
            .is_some_and(|(s, d)| s == d)
}

/// Computes MD5 hash using an 8KB buffer to minimize peak RSS memory usage.
fn md5_file(path: &Path) -> Option<md5::Digest> {
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
