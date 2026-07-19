#[cfg(target_os = "windows")]
use crate::priorites::set_priority;
use sysinfo::{ProcessesToUpdate, System};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS,
    NORMAL_PRIORITY_CLASS, PROCESS_CREATION_FLAGS, REALTIME_PRIORITY_CLASS,
};

#[cfg(target_os = "windows")]
pub fn parse_priority(name: &str) -> Option<PROCESS_CREATION_FLAGS> {
    match name {
        "BELOW_NORMAL_PRIORITY_CLASS" => Some(BELOW_NORMAL_PRIORITY_CLASS),
        "NORMAL_PRIORITY_CLASS" => Some(NORMAL_PRIORITY_CLASS),
        "ABOVE_NORMAL_PRIORITY_CLASS" => Some(ABOVE_NORMAL_PRIORITY_CLASS),
        "HIGH_PRIORITY_CLASS" => Some(HIGH_PRIORITY_CLASS),
        "REALTIME_PRIORITY_CLASS" => Some(REALTIME_PRIORITY_CLASS),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn set_process_priority(priority: &str) -> Result<(), String> {
    log::info!("priority called");
    let priority =
        parse_priority(priority).ok_or_else(|| format!("Unknown priority class: {}", priority))?;
    let mut sys = System::new_all();

    sys.refresh_processes(ProcessesToUpdate::All, true);

    for process in sys.processes_by_name("RobloxPlayerBeta".as_ref()) {
        let pid = process.pid();

        log::info!("found Roblox PID: {}, setting priority", pid);

        set_priority(pid, priority);
    }

    Ok(())
}

#[tauri::command]
pub async fn close_crash_handler() -> Result<(), String> {
    log::info!("close crash handler called");
    let mut sys = System::new_all();

    sys.refresh_processes(ProcessesToUpdate::All, true);

    for process in sys.processes_by_name("RobloxCrashHandler".as_ref()) {
        let pid = process.pid();

        log::info!("found Roblox's crash handler PID: {}, killing it ", pid);

        let _ = process.kill();
    }

    Ok(())
}
