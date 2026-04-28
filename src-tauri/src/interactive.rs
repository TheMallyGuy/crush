use windows::{
    core::{PSTR, PCSTR},
    Win32::Foundation::{BOOL, COLORREF, HWND, LPARAM, RECT},
    Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextA, GetWindowRect, IsWindowVisible,
        SetForegroundWindow, SetWindowPos, ShowWindow,
        GetWindowLongA, SetWindowLongA,
        SetLayeredWindowAttributes, SetWindowTextA,
        LAYERED_WINDOW_ATTRIBUTES_FLAGS,
        SWP_NOZORDER, SWP_NOACTIVATE, SWP_FRAMECHANGED,
        SW_MINIMIZE, SW_MAXIMIZE, SW_RESTORE,
        GWL_STYLE, GWL_EXSTYLE,
    },
};

// https://github.com/DawndreamerStudios/funkstrap/blob/main/Bloxstrap/Integrations/WindowController.cs


const WS_CAPTION: i32 = 0x00C00000;
const WS_THICKFRAME: i32 = 0x00040000;
const WS_BORDER: i32 = 0x00800000;

const WS_EX_LAYERED: i32 = 0x00080000;
const LWA_ALPHA: u32 = 0x2;

pub fn find_windows_by_title(keyword: &str) -> Vec<HWND> {
    let mut results: Vec<HWND> = Vec::new();

    struct SearchData {
        keyword: String,
        results: *mut Vec<HWND>,
    }

    unsafe extern "system" fn enum_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut SearchData);

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let mut buffer = [0u8; 512];
        let len = GetWindowTextA(hwnd, &mut buffer);

        if len > 0 {
            let title = String::from_utf8_lossy(&buffer[..len as usize]);
            if title.to_lowercase().contains(&data.keyword) {
                unsafe {
                    (*data.results).push(hwnd);
                }
            }
        }

        BOOL(1)
    }

    let mut data = SearchData {
        keyword: keyword.to_lowercase(),
        results: &mut results,
    };

    unsafe {
        EnumWindows(
            Some(enum_windows),
            LPARAM(&mut data as *mut _ as isize),
        );
    }

    results
}

pub fn move_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        SetWindowPos(
            hwnd,
            HWND(0),
            x,
            y,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

pub fn minimize_window(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_MINIMIZE);
    }
}

pub fn maximize_window(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_MAXIMIZE);
    }
}

pub fn restore_window(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_RESTORE);
    }
}

pub fn focus_window(hwnd: HWND) {
    unsafe {
        SetForegroundWindow(hwnd);
    }
}

pub fn get_window_rect(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            Some((
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
            ))
        } else {
            None
        }
    }
}

pub fn set_window_title(hwnd: HWND, title: &str) {
    let mut bytes = title.as_bytes().to_vec();
    bytes.push(0);

    unsafe {
        SetWindowTextA(hwnd, PCSTR(bytes.as_ptr()));
    }
}

pub fn set_borderless(hwnd: HWND, enabled: bool) {
    unsafe {
        let mut style = GetWindowLongA(hwnd, GWL_STYLE);

        if enabled {
            style &= !WS_CAPTION;
            style &= !WS_THICKFRAME;
            style &= !WS_BORDER;
        } else {
            style |= WS_CAPTION;
            style |= WS_THICKFRAME;
            style |= WS_BORDER;
        }

        SetWindowLongA(hwnd, GWL_STYLE, style);

        SetWindowPos(
            hwnd,
            HWND(0),
            0,
            0,
            0,
            0,
            SWP_NOZORDER | SWP_FRAMECHANGED,
        );
    }
}


pub fn set_transparency(hwnd: HWND, alpha: u8) {
    unsafe {
        let ex_style = GetWindowLongA(hwnd, GWL_EXSTYLE);
        
        if (ex_style & WS_EX_LAYERED) == 0 {
            SetWindowLongA(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED);
        }

        let _ = SetLayeredWindowAttributes(
            hwnd,
            COLORREF(0),
            alpha,
            LAYERED_WINDOW_ATTRIBUTES_FLAGS(LWA_ALPHA),
        );
    }
}