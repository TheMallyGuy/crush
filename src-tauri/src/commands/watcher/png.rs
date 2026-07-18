#[cfg(target_os = "windows")]
use std::path::Path;
#[cfg(target_os = "windows")]
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
pub(super) fn write_game_permission_png(
    game_id: u64,
    allow_control: bool,
    allow_title: bool,
    allow_transparency: bool,
    app: &AppHandle,
    vng: bool,
) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let player_folder = if vng { "PlayerVNG" } else { "Player" };
    let versions_dir = data_dir.join(player_folder).join("Versions");

    let version_dir = std::fs::read_dir(&versions_dir)
        .map_err(|e| format!("can't read versions dir {:?}: {}", versions_dir, e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| e.path().join("RobloxPlayerBeta.exe").exists())
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
        .ok_or_else(|| format!("no RobloxPlayerBeta.exe found under {:?}", versions_dir))?;

    let bloxstrap_dir = version_dir.join("content").join("bloxstrap");
    std::fs::create_dir_all(&bloxstrap_dir).map_err(|e| e.to_string())?;

    if let Ok(entries) = std::fs::read_dir(&bloxstrap_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str != "enabled.png" {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    let enabled_path = bloxstrap_dir.join("enabled.png");
    if !enabled_path.exists() {
        write_png_rgba(&enabled_path, 1, 1, &[255, 255, 255, 255])?;
        log::info!("wrote enabled.png to {:?}", enabled_path);
    }

    let game_png_path = bloxstrap_dir.join(format!("{}.png", game_id));
    let pixel = |on: bool| -> [u8; 4] {
        if on {
            [255, 255, 255, 255]
        } else {
            [0, 0, 0, 0]
        }
    };

    let mut pixels: Vec<u8> = Vec::with_capacity(12);
    pixels.extend_from_slice(&pixel(allow_control));
    pixels.extend_from_slice(&pixel(allow_title));
    pixels.extend_from_slice(&pixel(allow_transparency));

    write_png_rgba(&game_png_path, 3, 1, &pixels)?;
    log::info!(
        "wrote game permission PNG for {} -> {:?} (control={}, title={}, transparency={})",
        game_id,
        game_png_path,
        allow_control,
        allow_title,
        allow_transparency
    );

    Ok(())
}

#[cfg(target_os = "windows")]
fn write_png_rgba(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    fn adler32(data: &[u8]) -> u32 {
        let (mut a, mut b) = (1u32, 0u32);
        for &byte in data {
            a = (a + byte as u32) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }

    fn crc32(data: &[u8]) -> u32 {
        static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
        let table = TABLE.get_or_init(|| {
            let mut t = [0u32; 256];
            for (i, _) in t.clone().iter().enumerate() {
                let mut c = i as u32;
                for _ in 0..8 {
                    c = if c & 1 != 0 {
                        0xedb88320 ^ (c >> 1)
                    } else {
                        c >> 1
                    };
                }
                t[i] = c;
            }
            t
        });
        let mut crc = 0xffffffff_u32;
        for &byte in data {
            crc = table[((crc ^ byte as u32) & 0xff) as usize] ^ (crc >> 8);
        }
        crc ^ 0xffffffff
    }

    fn write_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
        let len = data.len() as u32;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(tag);
        out.extend_from_slice(data);
        let mut crc_input = Vec::with_capacity(4 + data.len());
        crc_input.extend_from_slice(tag);
        crc_input.extend_from_slice(data);
        out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    }

    let mut out: Vec<u8> = Vec::new();

    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_chunk(&mut out, b"IHDR", &ihdr);

    let mut raw: Vec<u8> = Vec::new();
    for row in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[row * width as usize * 4..(row + 1) * width as usize * 4]);
    }

    let mut zlib: Vec<u8> = Vec::new();
    zlib.push(0x78);
    zlib.push(0x01);
    zlib.push(0x01);

    let len16 = raw.len() as u16;
    let nlen16 = !len16;
    zlib.extend_from_slice(&len16.to_le_bytes());
    zlib.extend_from_slice(&nlen16.to_le_bytes());
    zlib.extend_from_slice(&raw);
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    write_chunk(&mut out, b"IDAT", &zlib);
    write_chunk(&mut out, b"IEND", &[]);

    std::fs::write(path, &out).map_err(|e| e.to_string())
}
