use dirs::desktop_dir;
use image;
use lnks::{Icon, Shortcut};
use std::env;

#[tauri::command]
pub async fn new_shorcut(name: String, game_id: u64, image_path: String) -> Result<(), String> {
    let crush = env::current_exe().unwrap();

    let ico_path = image_path.replace(".png", ".ico");
    let img = image::open(&image_path).map_err(|e| e.to_string())?;
    let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    resized.save(&ico_path).map_err(|e| e.to_string())?;

    let mut shortcut = Shortcut::new(crush);
    shortcut.arguments = Some(format!("-l {game_id}"));
    shortcut.icon = Some(Icon {
        path: ico_path.into(),
        index: 0,
    });

    let desktop = desktop_dir().ok_or_else(|| "Unable to locate desktop directory".to_string())?;
    let out = desktop.join(format!("{name}.lnk"));

    shortcut.save(out).map_err(|e| e.to_string())?;
    Ok(())
}
