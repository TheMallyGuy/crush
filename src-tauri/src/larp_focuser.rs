// dont fucking question why i name this larp i just cannot think of a name
// inspired by how TASX work, please check them out! https://github.com/8damon/TASX-Roblox-Optimizer/

use tauri_plugin_store::StoreExt;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows::Win32::System::Threading::{
    OpenProcess, SetPriorityClass, SetProcessAffinityMask, SetProcessWorkingSetSize,
    ABOVE_NORMAL_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, PROCESS_CREATION_FLAGS,
    PROCESS_SET_INFORMATION,
};
use windows_result::Error;

use crate::commands::pre_processing::parse_priority;

use std::thread;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub async fn start_larping(hwnd: HWND, app_handle: tauri::AppHandle) {
    let roblox;
    let mut pid: u32;
    let num_cpus = get_cpu_count();
    let full_mask: usize = (1 << num_cpus) - 1;
    let limited_mask: usize = 0b0011; // TODO make this flexable by adding a config to it
    let mut last_focused: bool = false;

    unsafe {
        pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        if pid == 0 {
            log::error!("failed to get roblox's PID via HWND when larping");
            return;
        }

        roblox = OpenProcess(PROCESS_SET_INFORMATION, false, pid);
    }

    loop {
        let focused = is_hwnd_focused(hwnd);

        if focused != last_focused {
            last_focused = focused;
            let priority = get_priority(&app_handle).unwrap_or(ABOVE_NORMAL_PRIORITY_CLASS);

            set_affinity(pid, full_mask);

            unsafe {
                match roblox {
                    Ok(h) => {
                        if let Err(e) = SetPriorityClass(h, priority) {
                            log::error!("failed to set priority for roblox {e}");
                        }
                        let _ = CloseHandle(h);
                    }

                    Err(ref e) => log::error!("failed to open roblox process {e}"),
                }
            }
        } else {
            log::info!("window not focus, larping the window");

            set_affinity(pid, limited_mask);
            trim_working_set(pid);

            unsafe {
                // TODO turn this into a reusable function
                match roblox {
                    Ok(h) => {
                        if let Err(e) = SetPriorityClass(h, IDLE_PRIORITY_CLASS) {
                            log::error!("failed to set priority for roblox {e}");
                        }
                        let _ = CloseHandle(h);
                    }

                    Err(ref e) => log::error!("failed to open roblox process {e}"),
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

// helpers and etc

fn is_hwnd_focused(hwnd: HWND) -> bool {
    unsafe {
        let focused: HWND = GetForegroundWindow();
        focused == hwnd
    }
}

fn get_priority(app: &tauri::AppHandle) -> Option<PROCESS_CREATION_FLAGS> {
    let store = app.store("config.json").ok()?;
    let priority = store.get("priority")?;
    let priority_str = priority.as_str()?;
    parse_priority(priority_str)
}

fn trim_working_set(pid: u32) {
    let roblox: Result<HANDLE, Error>;

    unsafe {
        roblox = OpenProcess(PROCESS_SET_INFORMATION, false, pid);
        match roblox {
            Ok(h) => {
                if let Err(e) = SetProcessWorkingSetSize(h, usize::MAX, usize::MAX) {
                    log::error!("failed to trim roblox {e}");
                }
                let _ = CloseHandle(h);
            }

            Err(ref e) => log::error!("failed to open roblox process {e}"),
        }
    }
}

fn set_affinity(pid: u32, mask: usize) {
    let roblox: Result<HANDLE, Error>;

    unsafe {
        roblox = OpenProcess(PROCESS_SET_INFORMATION, false, pid);
        match roblox {
            Ok(h) => {
                if let Err(e) = SetProcessAffinityMask(h, mask) {
                    log::error!("failed to set affinity roblox {e}");
                }
                let _ = CloseHandle(h);
            }

            Err(ref e) => log::error!("failed to open roblox process {e}"),
        }
    }
}
fn get_cpu_count() -> usize {
    unsafe {
        let mut info = SYSTEM_INFO::default();
        GetSystemInfo(&mut info);
        info.dwNumberOfProcessors as usize
    }
}
