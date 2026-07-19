use super::types::RichPresence;
#[cfg(target_os = "windows")]
use crate::interactive::LWA_COLORKEY;
use crate::tray::remove_menu_item;
use std::path::PathBuf;
use std::time::Instant;
use tauri::AppHandle;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

#[derive(Default, Debug)]
pub(super) struct Activity {
    pub(super) place_id: Option<u64>,
    #[cfg(target_os = "windows")]
    pub(super) universe_id: Option<u64>,
    pub(super) instance_id: Option<String>,
    pub(super) in_game: bool,
    pub(super) notified: bool,
    pub(super) join_initiated: bool,
    pub(super) is_private_server: bool,
    pub(super) access_code: Option<String>,
    pub(super) session_id: Option<i64>,
    pub(super) join_signaled: bool,
}

pub(super) struct WatcherState {
    pub(super) current_file: Option<PathBuf>,
    pub(super) offset: u64,
    pub(super) activity: Activity,
    pub(super) last_rpc: Option<Instant>,
    pub(super) udmux_handled: bool,
    pub(super) pending_server_ip: Option<String>,
    pub(super) pending_server_location: Option<String>,
    pub(super) location_notified: bool,
    pub(super) bloxstrap_rpc: Option<RichPresence>,
    #[cfg(target_os = "windows")]
    pub(super) roblox_hwnd: Option<HWND>,
    pub(super) window_started: bool,

    pub(super) sleep_schedule_count: u64,

    #[cfg(target_os = "windows")]
    pub(super) larp_started: bool,

    // window geometry saved at StartWindow
    #[cfg(target_os = "windows")]
    pub(super) starting_x: i32,
    #[cfg(target_os = "windows")]
    pub(super) starting_y: i32,
    #[cfg(target_os = "windows")]
    pub(super) starting_width: i32,
    #[cfg(target_os = "windows")]
    pub(super) starting_height: i32,

    // last applied geometry (updated by SetWindow)
    #[cfg(target_os = "windows")]
    pub(super) last_x: i32,
    #[cfg(target_os = "windows")]
    pub(super) last_y: i32,
    #[cfg(target_os = "windows")]
    pub(super) last_width: i32,
    #[cfg(target_os = "windows")]
    pub(super) last_height: i32,

    // scale reference resolution (updated by scaleWidth/scaleHeight fields)
    #[cfg(target_os = "windows")]
    pub(super) last_sc_width: f64,
    #[cfg(target_os = "windows")]
    pub(super) last_sc_height: f64,

    // transparency state
    #[cfg(target_os = "windows")]
    pub(super) last_transparency: u8,
    #[cfg(target_os = "windows")]
    pub(super) last_window_color: u32,
    #[cfg(target_os = "windows")]
    pub(super) last_transparency_mode: u32,

    // misc window state
    #[cfg(target_os = "windows")]
    pub(super) borderless: bool,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            current_file: None,
            offset: 0,
            activity: Activity::default(),
            last_rpc: None,
            udmux_handled: false,
            pending_server_ip: None,
            pending_server_location: None,
            location_notified: false,
            bloxstrap_rpc: None,
            #[cfg(target_os = "windows")]
            roblox_hwnd: None,
            window_started: false,

            #[cfg(target_os = "windows")]
            larp_started: false,

            sleep_schedule_count: 0,

            #[cfg(target_os = "windows")]
            starting_x: 0,
            #[cfg(target_os = "windows")]
            starting_y: 0,
            #[cfg(target_os = "windows")]
            starting_width: 0,
            #[cfg(target_os = "windows")]
            starting_height: 0,

            #[cfg(target_os = "windows")]
            last_x: 0,
            #[cfg(target_os = "windows")]
            last_y: 0,
            #[cfg(target_os = "windows")]
            last_width: 0,
            #[cfg(target_os = "windows")]
            last_height: 0,

            #[cfg(target_os = "windows")]
            last_sc_width: 1280.0,
            #[cfg(target_os = "windows")]
            last_sc_height: 720.0,

            #[cfg(target_os = "windows")]
            last_transparency: 255,
            #[cfg(target_os = "windows")]
            last_window_color: 0,
            #[cfg(target_os = "windows")]
            last_transparency_mode: LWA_COLORKEY,

            #[cfg(target_os = "windows")]
            borderless: false,
        }
    }
}

impl WatcherState {
    pub(super) fn reset_for_new_game(&mut self, app: &AppHandle) {
        self.activity = Activity::default();
        self.udmux_handled = false;
        self.pending_server_ip = None;
        self.pending_server_location = None;
        self.location_notified = false;
        self.bloxstrap_rpc = None;
        #[cfg(target_os = "windows")]
        {
            self.roblox_hwnd = None;
            self.borderless = false;
        }
        self.window_started = false;

        remove_menu_item(app, "serverinfo").ok();
    }
    pub(super) fn reset_fully(&mut self) {
        *self = WatcherState::default();
    }
}
