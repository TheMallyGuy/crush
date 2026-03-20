use std::fs::{self, File};
use std::io::copy;
use std::path::Path;
use zip::ZipArchive;

#[tauri::command]
pub fn extract_zip(zip_path: String, dest: String) -> Result<(), String> {
    let file = File::open(&zip_path)
        .map_err(|e| format!("Cannot open zip '{}': {}", zip_path, e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Cannot read zip archive: {}", e))?;

    let dest_path = Path::new(&dest);

    fs::create_dir_all(dest_path)
        .map_err(|e| format!("Cannot create dest dir '{}': {}", dest_path.display(), e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Cannot read entry {}: {}", i, e))?;

        let entry_name = match entry.enclosed_name() {
            Some(name) => name.to_owned(),
            None => continue,
        };

        let outpath = dest_path.join(&entry_name);

        if entry.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Cannot create dir '{}': {}", outpath.display(), e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create parent '{}': {}", parent.display(), e))?;
            }

            let mut outfile = File::create(&outpath)
                .map_err(|e| format!("Cannot create file '{}': {}", outpath.display(), e))?;

            copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Cannot write '{}': {}", outpath.display(), e))?;
        }
    }

    Ok(())
}