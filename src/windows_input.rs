use crate::player::KeyEvent;


pub const SCANCODES: [u16; 15] = [
    0x15, 0x16, 0x17, 0x18, 0x19,
    0x23, 0x24, 0x25, 0x26, 0x27,
    0x31, 0x32, 0x33, 0x34, 0x35,
];




#[derive(Default)]
struct KeyState {
    held: [bool; 15],
    pending: [bool; 15],
}
impl KeyState {
    fn queue_releases(&mut self, events: &[KeyEvent]) {
        for event in events.iter().filter(|e| !e.down) {
            if let Some(held) = self.held.get(event.key as usize) {
                if *held { self.pending[event.key as usize] = true; }
            }
        }
    }
    fn queue_all_releases(&mut self) {
        for i in 0..15 {
            if self.held[i] { self.pending[i] = true; }
        }
    }
    fn pending_events(&self) -> Vec<KeyEvent> {
        (0..15).filter(|&key| self.pending[key])
            .map(|key| KeyEvent { key: key as u8, down: false }).collect()
    }
    fn accepted(&mut self, events: &[KeyEvent]) {
        for event in events {
            let i = event.key as usize;
            self.held[i] = event.down;
            self.pending[i] = false;
        }
    }
    fn any_pending(&self) -> bool { self.pending.iter().any(|&p| p) }
}

#[cfg(windows)]
mod platform {
    use super::{KeyEvent, KeyState, SCANCODES};
    use std::{mem::size_of, sync::Mutex};
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
        KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    static STATE: Mutex<KeyState> = Mutex::new(KeyState {
        held: [false; 15], pending: [false; 15],
    });

    pub fn sky_is_foreground() -> bool {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_null() { return false; }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid == 0 { return false; }
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() { return false; }
        let mut buffer = [0u16; 1024];
        let mut length = buffer.len() as u32;
        let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) };
        unsafe { CloseHandle(handle) };
        if ok == 0 || length == 0 { return false; }
        let path = String::from_utf16_lossy(&buffer[..length as usize]);
        path.rsplit(['\\', '/']).next()
            .is_some_and(|name| name.eq_ignore_ascii_case("Sky.exe"))
    }

    fn send(events: &[KeyEvent]) -> Result<usize, String> {
        if events.is_empty() { return Ok(0); }
        
        
        if !sky_is_foreground() { return Err("Sky is not the foreground window".into()); }
        let mut inputs = Vec::with_capacity(events.len());
        for event in events {
            let scancode = *SCANCODES.get(event.key as usize)
                .ok_or_else(|| format!("Invalid Sky note index {}", event.key))?;
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0, wScan: scancode,
                        dwFlags: KEYEVENTF_SCANCODE |
                            if event.down { 0 } else { KEYEVENTF_KEYUP },
                        time: 0, dwExtraInfo: 0,
                    },
                },
            });
        }
        
        
        let sent = unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), size_of::<INPUT>() as i32) };
        Ok(sent as usize)
    }

    pub fn has_pending_releases() -> bool {
        STATE.lock().unwrap_or_else(|e| e.into_inner()).any_pending()
    }

    pub fn dispatch(events: &[KeyEvent]) -> Result<(), String> {
        if events.is_empty() { return flush_pending_releases(); }
        if events.iter().any(|event| event.key >= 15) {
            return Err("Invalid Sky note index".into());
        }
        let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
        state.queue_releases(events);
        if !sky_is_foreground() {
            
            
            return if events.iter().any(|event| event.down) {
                Err("Sky is not the foreground window".into())
            } else { Ok(()) };
        }
        let pending = state.pending_events();
        if !pending.is_empty() {
            let sent = send(&pending)?;
            state.accepted(&pending[..sent]);
            if sent != pending.len() {
                return Err(format!("Windows accepted {sent} of {} pending releases", pending.len()));
            }
        }
        
        
        let mut projected_held = state.held;
        let filtered: Vec<KeyEvent> = events.iter().copied().filter(|event| {
            let idx = event.key as usize;
            if event.down {
                projected_held[idx] = true;
                true
            } else if projected_held[idx] {
                projected_held[idx] = false;
                true
            } else {
                false
            }
        }).collect();
        if filtered.is_empty() { return Ok(()); }
        let sent = send(&filtered)?;
        state.accepted(&filtered[..sent]);
        if sent != filtered.len() {
            state.queue_all_releases();
            return Err(format!("Windows accepted {sent} of {} keyboard events", filtered.len()));
        }
        Ok(())
    }

    pub fn flush_pending_releases() -> Result<(), String> {
        if !has_pending_releases() || !sky_is_foreground() { return Ok(()); }
        let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let pending = state.pending_events();
        if pending.is_empty() { return Ok(()); }
        let sent = send(&pending)?;
        state.accepted(&pending[..sent]);
        if sent != pending.len() {
            return Err(format!("Windows accepted {sent} of {} pending releases", pending.len()));
        }
        Ok(())
    }

    pub fn release_every_key() {
        let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
        state.queue_all_releases();
        drop(state);
        let _ = flush_pending_releases();
    }
}
#[cfg(windows)]
pub use platform::{dispatch, flush_pending_releases, has_pending_releases, release_every_key, sky_is_foreground};

#[cfg(not(windows))]
pub fn sky_is_foreground() -> bool { false }
#[cfg(not(windows))]
pub fn dispatch(_: &[KeyEvent]) -> Result<(), String> { Err("Windows only".into()) }
#[cfg(not(windows))]
pub fn release_every_key() {}
#[cfg(not(windows))]
pub fn has_pending_releases() -> bool { false }
#[cfg(not(windows))]
pub fn flush_pending_releases() -> Result<(), String> { Ok(()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sky_keyboard_scancode_mapping_is_stable() {
        assert_eq!(SCANCODES, [
            0x15, 0x16, 0x17, 0x18, 0x19,
            0x23, 0x24, 0x25, 0x26, 0x27,
            0x31, 0x32, 0x33, 0x34, 0x35,
        ]);
    }
    #[test]
    fn tracks_only_confirmed_keys_and_queues_releases() {
        let mut state = KeyState::default();
        assert!(state.pending_events().is_empty());
        state.accepted(&[KeyEvent { key: 2, down: true }]);
        state.queue_releases(&[KeyEvent { key: 2, down: false },
            KeyEvent { key: 4, down: false }]);
        assert_eq!(state.pending_events(), vec![KeyEvent { key: 2, down: false }]);
        state.accepted(&[KeyEvent { key: 2, down: false }]);
        assert!(!state.any_pending());
    }
    #[test]
    fn bulk_release_does_not_queue_unpressed_keys() {
        let mut state = KeyState::default();
        state.queue_all_releases();
        assert_eq!(state.pending_events().len(), 0);
        state.accepted(&[KeyEvent { key: 1, down: true },
            KeyEvent { key: 11, down: true }]);
        state.queue_all_releases();
        assert_eq!(state.pending_events().len(), 2);
    }
}
