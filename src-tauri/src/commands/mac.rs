use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[tauri::command]
pub fn move_app_to_applications(source_path: String, dest_name: String) -> Result<(), String> {
    let source = PathBuf::from(&source_path);

    if !source.exists() {
        return Err(format!("Source app bundle not found: {}", source.display()));
    }

    let applications_dir = Path::new("/Applications");
    let destination = applications_dir.join(&dest_name);

    let staging_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let staging = applications_dir.join(format!(".{}.installing-{}", dest_name, staging_suffix));

    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| {
            format!(
                "Cannot clean up stale staging dir '{}': {}",
                staging.display(),
                e
            )
        })?;
    }

    if let Err(e) = copy_dir_recursive(&source, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(format!(
            "Failed to copy app bundle to staging location: {}",
            e
        ));
    }

    if destination.exists() {
        if let Err(e) = fs::remove_dir_all(&destination) {
            return Err(format!(
                "Copied new app to staging but failed to remove existing install at '{}': {}. The new copy is safely at '{}'.",
                destination.display(),
                e,
                staging.display()
            ));
        }
    }

    fs::rename(&staging, &destination).map_err(|e| {
        format!(
            "Copied new app but failed to move it into place at '{}': {}. It's still available at '{}'.",
            destination.display(),
            e,
            staging.display()
        )
    })?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Cannot create dir '{}': {}", dst.display(), e))?;

    for entry in
        fs::read_dir(src).map_err(|e| format!("Cannot read dir '{}': {}", src.display(), e))?
    {
        let entry =
            entry.map_err(|e| format!("Cannot read dir entry in '{}': {}", src.display(), e))?;
        let file_type = entry.file_type().map_err(|e| {
            format!(
                "Cannot get file type for '{}': {}",
                entry.path().display(),
                e
            )
        })?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_symlink() {
            copy_symlink(&src_path, &dst_path)?;
        } else if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "Cannot copy '{}' -> '{}': {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn copy_symlink(src: &Path, dst: &Path) -> Result<(), String> {
    let target = fs::read_link(src)
        .map_err(|e| format!("Cannot read symlink '{}': {}", src.display(), e))?;
    std::os::unix::fs::symlink(&target, dst).map_err(|e| {
        format!(
            "Cannot create symlink '{}' -> '{}': {}",
            dst.display(),
            target.display(),
            e
        )
    })
}

#[cfg(not(unix))]
fn copy_symlink(src: &Path, dst: &Path) -> Result<(), String> {
    fs::copy(src, dst)
        .map(|_| ())
        .map_err(|e| format!("Cannot copy '{}': {}", src.display(), e))
}


fn client_settings_path(app_bundle: &str) -> Result<PathBuf, String> {
    if app_bundle.is_empty()
        || app_bundle.contains('/')
        || app_bundle.contains('\\')
        || app_bundle.contains("..")
    {
        return Err(format!("Invalid app bundle name: '{}'", app_bundle));
    }
 
    Ok(Path::new("/Applications")
        .join(app_bundle)
        .join("Contents/MacOS/ClientSettings/ClientAppSettings.json"))
}
 

#[tauri::command]
pub fn write_mac_client_settings(app_bundle: String, content: String) -> Result<(), String> {
    let path = client_settings_path(&app_bundle)?;
 
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create dir '{}': {}", parent.display(), e))?;
    }
 
    fs::write(&path, content).map_err(|e| format!("Cannot write '{}': {}", path.display(), e))
}
 

#[tauri::command]
pub fn remove_mac_client_settings(app_bundle: String) -> Result<(), String> {
    let path = client_settings_path(&app_bundle)?;
 
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Cannot remove '{}': {}", path.display(), e))?;
    }
 
    Ok(())
}