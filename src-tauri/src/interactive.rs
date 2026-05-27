use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, RECT},
        Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE},
        UI::WindowsAndMessaging::{
            EnumWindows, GetSystemMetrics, GetWindowLongW, GetWindowRect, GetWindowTextW,
            IsWindowVisible, SetForegroundWindow, SetLayeredWindowAttributes, SetWindowLongW,
            SetWindowPos, SetWindowTextW, ShowWindow, LAYERED_WINDOW_ATTRIBUTES_FLAGS,
            SET_WINDOW_POS_FLAGS, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN,
            SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WINDOW_LONG_PTR_INDEX,
        },
    },
};

use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};

const GWLW_STYLE: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(-16);
const GWLW_EXSTYLE: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(-20);

const WS_CAPTION: i32 = 0x00C0_0000_u32 as i32;
const WS_THICKFRAME: i32 = 0x0004_0000_u32 as i32;
const WS_BORDER: i32 = 0x0080_0000_u32 as i32;
const WS_SYSMENU: i32 = 0x0008_0000_u32 as i32;
const WS_MINIMIZEBOX: i32 = 0x0002_0000_u32 as i32;
const WS_MAXIMIZEBOX: i32 = 0x0001_0000_u32 as i32;
const WS_EX_LAYERED: i32 = 0x0008_0000_u32 as i32;

pub const LWA_COLORKEY: u32 = 0x0000_0001;
pub const LWA_ALPHA: u32 = 0x0000_0002;

const DWMWA_BORDER_COLOR: i32 = 34;
const DWMWA_CAPTION_COLOR: i32 = 35;

#[inline(always)]
const fn swp(v: u32) -> SET_WINDOW_POS_FLAGS {
    SET_WINDOW_POS_FLAGS(v)
}

pub fn find_windows_by_title(keyword: &str) -> Vec<HWND> {
    struct SearchData {
        keyword: String,
        results: *mut Vec<HWND>,
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> windows_result::BOOL {
        let data = &mut *(lparam.0 as *mut SearchData);

        if !IsWindowVisible(hwnd).as_bool() {
            return windows_result::BOOL(1);
        }

        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len > 0 {
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            if title.to_lowercase().contains(&data.keyword) {
                (*data.results).push(hwnd);
            }
        }

        windows_result::BOOL(1)
    }

    let mut results: Vec<HWND> = Vec::new();
    let mut data = SearchData {
        keyword: keyword.to_lowercase(),
        results: &mut results,
    };

    unsafe {
        let _ = EnumWindows(Some(enum_cb), LPARAM(&mut data as *mut _ as isize));
    }

    results
}

pub fn get_window_rect(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    unsafe {
        let mut r = RECT::default();
        if GetWindowRect(hwnd, &mut r).is_ok() {
            Some((r.left, r.top, r.right - r.left, r.bottom - r.top))
        } else {
            None
        }
    }
}

pub fn get_monitor_info(hwnd: HWND) -> (i32, i32, i32, i32) {
    unsafe {
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(hmon, &mut info);
        let r = info.rcMonitor;
        (r.left, r.top, r.right - r.left, r.bottom - r.top)
    }
}

pub fn get_primary_screen_size() -> (i32, i32) {
    unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
}

pub fn get_virtual_screen_size() -> (i32, i32) {
    unsafe {
        (
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    }
}

pub fn move_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            None,
            x,
            y,
            width,
            height,
            swp(SWP_NOZORDER.0 | SWP_NOACTIVATE.0 | SWP_ASYNCWINDOWPOS.0),
        );
    }
}

pub fn minimize_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_MINIMIZE);
    }
}

pub fn maximize_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_MAXIMIZE);
    }
}

pub fn restore_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
    }
}

pub fn focus_window(hwnd: HWND) {
    unsafe {
        let _ = SetForegroundWindow(hwnd);
    }
}

pub fn set_window_title(hwnd: HWND, title: &str) {
    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
    }
}

pub fn set_borderless(hwnd: HWND, borderless: bool) {
    unsafe {
        let mut style = GetWindowLongW(hwnd, GWLW_STYLE);
        if borderless {
            style &= !(WS_CAPTION
                | WS_THICKFRAME
                | WS_BORDER
                | WS_SYSMENU
                | WS_MINIMIZEBOX
                | WS_MAXIMIZEBOX);
        } else {
            style |= WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
        }
        SetWindowLongW(hwnd, GWLW_STYLE, style);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            swp(SWP_NOMOVE.0
                | SWP_NOSIZE.0
                | SWP_NOZORDER.0
                | SWP_FRAMECHANGED.0
                | SWP_ASYNCWINDOWPOS.0
                | SWP_NOACTIVATE.0),
        );
    }
}

pub fn set_layered_transparency(hwnd: HWND, color: u32, alpha: u8, mode: u32) {
    unsafe {
        let ex = GetWindowLongW(hwnd, GWLW_EXSTYLE);
        if (ex & WS_EX_LAYERED) == 0 {
            SetWindowLongW(hwnd, GWLW_EXSTYLE, ex | WS_EX_LAYERED);
        }
        let _ = SetLayeredWindowAttributes(
            hwnd,
            COLORREF(color),
            alpha,
            LAYERED_WINDOW_ATTRIBUTES_FLAGS(mode),
        );
    }
}

pub fn reset_layered(hwnd: HWND) {
    unsafe {
        let ex = GetWindowLongW(hwnd, GWLW_EXSTYLE);
        SetWindowLongW(hwnd, GWLW_EXSTYLE, ex & !WS_EX_LAYERED);
    }
}

pub fn set_window_color(hwnd: HWND, caption: Option<u32>, border: Option<u32>) {
    #[inline]
    fn rgb_to_bgr(c: u32) -> u32 {
        let r = (c >> 16) & 0xFF;
        let g = (c >> 8) & 0xFF;
        let b = c & 0xFF;
        (b << 16) | (g << 8) | r
    }

    unsafe {
        if let Some(c) = caption {
            let mut v = rgb_to_bgr(c);
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWINDOWATTRIBUTE(DWMWA_CAPTION_COLOR),
                &mut v as *mut u32 as *const core::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
        if let Some(c) = border {
            let mut v = rgb_to_bgr(c);
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWINDOWATTRIBUTE(DWMWA_BORDER_COLOR),
                &mut v as *mut u32 as *const core::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}
