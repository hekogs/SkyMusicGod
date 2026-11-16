





#[cfg(target_os = "windows")]
mod windows {
    use std::sync::atomic::{AtomicIsize, Ordering};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    use windows_sys::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE,
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId,
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos,
        GetCursorPos, GetWindowRect,
        GWL_EXSTYLE, WS_EX_NOACTIVATE,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    };

    const TITLE: &str = "Sky Music God Mini Player";
    static CONFIGURED_HWND: AtomicIsize = AtomicIsize::new(0);

    struct Search {
        pid: u32,
        hwnd: HWND,
    }

    unsafe extern "system" fn find_window(hwnd: HWND, parameter: LPARAM) -> i32 {
        
        let search = unsafe { &mut *(parameter as *mut Search) };
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid); }
        if pid != search.pid { return 1; }
        let mut title = [0u16; 128];
        let len = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32) };
        if len > 0 && String::from_utf16_lossy(&title[..len as usize]) == TITLE {
            search.hwnd = hwnd;
            0
        } else {
            1
        }
    }

    fn mini_hwnd() -> Option<HWND> {
        let mut found = Search { pid: std::process::id(), hwnd: std::ptr::null_mut() };
        unsafe { EnumWindows(Some(find_window), &mut found as *mut Search as LPARAM); }
        (!found.hwnd.is_null()).then_some(found.hwnd)
    }

    pub(super) fn library_upward(extension: f32, scale: f32) -> bool {
        let Some(hwnd) = mini_hwnd() else { return false };
        let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        if monitor.is_null() { return false }
        let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        let mut rect: RECT = unsafe { std::mem::zeroed() };
        if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0
            || unsafe { GetWindowRect(hwnd, &mut rect) } == 0 { return false; }
        let required = (extension * scale).ceil() as i32 + 8;
        let below = info.rcWork.bottom.saturating_sub(rect.bottom);
        let above = rect.top.saturating_sub(info.rcWork.top);
        below < required && above > below
    }

    pub(super) fn drag_start() -> Option<([i32; 2], [i32; 2])> {
        let hwnd = mini_hwnd()?;
        let mut cursor = POINT { x: 0, y: 0 };
        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if unsafe { GetCursorPos(&mut cursor) } == 0 || unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
            return None;
        }
        Some(([cursor.x, cursor.y], [rect.left, rect.top]))
    }

    pub(super) fn drag_update(start: ([i32; 2], [i32; 2])) {
        let Some(hwnd) = mini_hwnd() else { return };
        let mut cursor = POINT { x: 0, y: 0 };
        if unsafe { GetCursorPos(&mut cursor) } == 0 { return; }
        let x = start.1[0].saturating_add(cursor.x.saturating_sub(start.0[0]));
        let y = start.1[1].saturating_add(cursor.y.saturating_sub(start.0[1]));
        unsafe { SetWindowPos(hwnd, std::ptr::null_mut(), x, y, 0, 0,
            SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER); }
    }

    pub(super) fn apply() {
        let mut found = Search { pid: std::process::id(), hwnd: std::ptr::null_mut() };
        unsafe { EnumWindows(Some(find_window), &mut found as *mut Search as LPARAM); }
        if found.hwnd.is_null() { return; }
        let handle = found.hwnd as isize;
        if CONFIGURED_HWND.load(Ordering::Relaxed) == handle { return; }

        
        
        
        
        let extended = unsafe { GetWindowLongPtrW(found.hwnd, GWL_EXSTYLE) };
        if extended & WS_EX_NOACTIVATE as isize == 0 {
            unsafe {
                SetWindowLongPtrW(found.hwnd, GWL_EXSTYLE,
                    extended | WS_EX_NOACTIVATE as isize);
                SetWindowPos(found.hwnd, std::ptr::null_mut(), 0, 0, 0, 0,
                    SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE |
                    SWP_NOSIZE | SWP_NOZORDER);
            }
        }

        
        
        let no_border = DWMWA_COLOR_NONE;
        
        let round_corners = DWMWCP_DONOTROUND;
        unsafe {
            DwmSetWindowAttribute(
                found.hwnd, DWMWA_BORDER_COLOR as u32,
                &no_border as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
            DwmSetWindowAttribute(
                found.hwnd, DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                &round_corners as *const i32 as *const _,
                std::mem::size_of::<i32>() as u32,
            );
        }
        CONFIGURED_HWND.store(handle, Ordering::Relaxed);
    }

    pub(super) fn reset() {
        CONFIGURED_HWND.store(0, Ordering::Relaxed);
    }
}

#[cfg(target_os = "windows")]
pub fn clip_mini() { windows::apply(); }
#[cfg(target_os = "windows")]
pub fn mini_drag_start() -> Option<([i32; 2], [i32; 2])> { windows::drag_start() }
#[cfg(target_os = "windows")]
pub fn mini_drag_update(start: ([i32; 2], [i32; 2])) { windows::drag_update(start); }
#[cfg(target_os = "windows")]
pub fn mini_library_should_open_upward(extension: f32, scale: f32) -> bool { windows::library_upward(extension, scale) }
#[cfg(target_os = "windows")]
pub fn reset_mini() { windows::reset(); }

#[cfg(not(target_os = "windows"))]
pub fn clip_mini() {}
#[cfg(not(target_os = "windows"))]
pub fn mini_drag_start() -> Option<([i32; 2], [i32; 2])> { None }
#[cfg(not(target_os = "windows"))]
pub fn mini_drag_update(_: ([i32; 2], [i32; 2])) {}
#[cfg(not(target_os = "windows"))]
pub fn mini_library_should_open_upward(_: f32, _: f32) -> bool { false }
#[cfg(not(target_os = "windows"))]
pub fn reset_mini() {}
