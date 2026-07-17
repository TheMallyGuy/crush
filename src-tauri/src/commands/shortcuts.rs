use dirs::desktop_dir;
use image;
use lnks::{Icon, Shortcut};
use tauri::Manager;

#[tauri::command]
pub async fn new_shorcut(
    app: tauri::AppHandle,
    name: String,
    game_id: u64,
    image_path: String,
) -> Result<(), String> {
    let resource_path = app
        .path()
        .resolve("open_game.bat", tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;

    let crush = resource_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    let crush_str = crush.to_string_lossy().replace("\\\\?\\", "");
    let final_crush_path = std::path::PathBuf::from(crush_str);

    let ico_path = image_path.replace(".png", ".ico");
    let img = image::open(&image_path).map_err(|e| e.to_string())?;
    let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    resized.save(&ico_path).map_err(|e| e.to_string())?;

    let mut shortcut = Shortcut::new(final_crush_path);
    shortcut.arguments = Some(format!(" {}", game_id));
    shortcut.icon = Some(Icon {
        path: ico_path.into(),
        index: 0,
    });

    let desktop = desktop_dir().ok_or_else(|| "Unable to locate desktop directory".to_string())?;
    let out = desktop.join(format!("{name}.lnk"));

    shortcut.save(out).map_err(|e| e.to_string())?;
    Ok(())
}
