#[derive(serde::Serialize)]
pub struct BuildInfo {
    hash: String,
    build_date: String,
    version: String,
}

#[tauri::command]
pub async fn crush() -> Result<BuildInfo, String> {
    Ok(BuildInfo {
        hash: env!("VERGEN_RUSTC_COMMIT_HASH").to_string(),
        build_date: env!("VERGEN_BUILD_DATE").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command] // this is required for swifttunnel
pub async fn relaunch_with_admins_perms(
    app: tauri::AppHandle,
    deeplink: Option<String>,
) -> Result<(), String> {
    relaunch_elevated(deeplink)?;

    app.exit(0);

    Ok(())
}

#[cfg(target_os = "windows")]
fn relaunch_elevated(deeplink: Option<String>) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{w, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;

    let file: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let params: Option<Vec<u16>> = deeplink.filter(|s| !s.is_empty()).map(|link| {
        std::ffi::OsString::from(format!("--deeplink \"{}\"", link))
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    });

    let result = unsafe {
        ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(file.as_ptr()),
            match &params {
                Some(p) => PCWSTR(p.as_ptr()),
                None => PCWSTR::null(),
            },
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if (result.0 as isize) <= 32 {
        return Err(format!(
            "failed to relaunch elevated (ShellExecuteW returned {})",
            result.0 as isize
        ));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn relaunch_elevated(_deeplink: Option<String>) -> Result<(), String> {
    Err("elevation is only supported on Windows".to_string())
}

#[tauri::command]
pub fn check_elevated() -> bool {
    is_elevated()
}

#[cfg(target_os = "windows")]
fn is_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token_handle = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        );

        result.is_ok() && elevation.TokenIsElevated != 0
    }
}
