use regex::Regex;
use std::sync::OnceLock;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
};
use windows_result::BOOL;

pub(super) fn is_roblox_running(system: &mut System) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"(?i)robloxplayerbeta").unwrap());
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    system
        .processes()
        .values()
        .any(|p| re.is_match(p.name().to_string_lossy().as_ref()))
}

pub(super) fn get_roblox_pid(system: &mut System) -> Option<u32> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"(?i)robloxplayerbeta").unwrap());
    system
        .processes()
        .values()
        .find(|p| re.is_match(p.name().to_string_lossy().as_ref()))
        .map(|p| p.pid().as_u32())
}

#[cfg(target_os = "windows")]
pub(super) fn find_hwnd_by_pid(target_pid: u32) -> Option<HWND> {
    struct SearchState {
        pid: u32,
        result: Option<HWND>,
    }

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam.0 as *mut SearchState);
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == state.pid && IsWindowVisible(hwnd).as_bool() {
            state.result = Some(hwnd);
            return false.into();
        }
        true.into()
    }

    let mut search = SearchState {
        pid: target_pid,
        result: None,
    };
    unsafe {
        let _ = EnumWindows(Some(callback), LPARAM(&mut search as *mut _ as isize));
    }
    search.result
}
