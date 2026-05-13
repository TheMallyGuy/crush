use sysinfo::Pid;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_CREATION_FLAGS, PROCESS_SET_INFORMATION, SetPriorityClass
};
use windows::Win32::Foundation::CloseHandle;

pub fn set_priority(pid: Pid, priority : PROCESS_CREATION_FLAGS) {
    unsafe {
        let process = OpenProcess(PROCESS_SET_INFORMATION, false, pid.as_u32()).expect("Failed to open process");

        let _ = SetPriorityClass(process, priority).expect("Failed to set priority");

        CloseHandle(process).unwrap()
    }
}