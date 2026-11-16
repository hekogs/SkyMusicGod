use std::sync::{Arc, Mutex};
use eframe::egui;

pub const RELEASES_URL: &str = "https://github.com/hekogs/SkyMusicGod/releases/latest";
const API_PATH: &str = "/repos/hekogs/SkyMusicGod/releases/latest";
const MAX_RESPONSE: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateState {
    Idle,
    Checking,
    Current,
    Available(String),
    Error(String),
}

impl Default for UpdateState {
    fn default() -> Self { Self::Idle }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u64, u64, u64);

fn parse_version(version: &str) -> Option<Version> {
    let clean = version.trim().strip_prefix('v').unwrap_or(version.trim());
    let pieces: Vec<&str> = clean.split('.').collect();
    if pieces.len() != 3 || pieces.iter().any(|part| part.is_empty()
        || !part.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    Some(Version(pieces[0].parse().ok()?, pieces[1].parse().ok()?, pieces[2].parse().ok()?))
}

fn compare_tag(tag: &str) -> Result<UpdateState, String> {
    let latest = parse_version(tag).ok_or_else(|| "Invalid release version".to_owned())?;
    let current = parse_version(env!("CARGO_PKG_VERSION"))
        .ok_or_else(|| "Invalid installed version".to_owned())?;
    if latest > current { Ok(UpdateState::Available(tag.to_owned())) }
    else { Ok(UpdateState::Current) }
}

fn check_release() -> Result<UpdateState, String> {
    let data = fetch_latest_json()?;
    let response: serde_json::Value = serde_json::from_slice(&data)
        .map_err(|_| "Invalid response from GitHub".to_owned())?;
    let tag = response.get("tag_name").and_then(serde_json::Value::as_str)
        .ok_or_else(|| "GitHub did not return a release version".to_owned())?;
    compare_tag(tag)
}

#[cfg(target_os = "windows")]
fn fetch_latest_json() -> Result<Vec<u8>, String> {
    use std::ptr::{null, null_mut};
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Networking::WinHttp::{
        WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
        WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
        WinHttpSetTimeouts, WINHTTP_ACCESS_TYPE_DEFAULT_PROXY, WINHTTP_FLAG_SECURE,
        WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
    };

    struct Handle(*mut core::ffi::c_void);
    impl Drop for Handle {
        fn drop(&mut self) { if !self.0.is_null() { unsafe { WinHttpCloseHandle(self.0); } } }
    }
    fn wide(s: &str) -> Vec<u16> {
        std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }
    fn valid(handle: *mut core::ffi::c_void) -> Result<Handle, String> {
        if handle.is_null() { Err(format!("Network error {}", std::io::Error::last_os_error())) }
        else { Ok(Handle(handle)) }
    }

    let agent = wide(concat!("SkyMusicGod/", env!("CARGO_PKG_VERSION")));
    let host = wide("api.github.com");
    let method = wide("GET");
    let path = wide(API_PATH);
    let session = valid(unsafe {
        WinHttpOpen(agent.as_ptr(), WINHTTP_ACCESS_TYPE_DEFAULT_PROXY, null(), null(), 0)
    })?;
    unsafe { WinHttpSetTimeouts(session.0, 4000, 4000, 4000, 6500); }
    let connect = valid(unsafe { WinHttpConnect(session.0, host.as_ptr(), 443, 0) })?;
    let request = valid(unsafe {
        WinHttpOpenRequest(connect.0, method.as_ptr(), path.as_ptr(),
            null(), null(), null(), WINHTTP_FLAG_SECURE)
    })?;
    let headers = wide("Accept: application/vnd.github+json\r\nX-GitHub-Api-Version: 2022-11-28\r\n");
    if unsafe { WinHttpSendRequest(request.0, headers.as_ptr(), (headers.len() - 1) as u32,
        null(), 0, 0, 0) } == 0 {
        return Err(format!("Connection failed: {}", std::io::Error::last_os_error()));
    }
    if unsafe { WinHttpReceiveResponse(request.0, null_mut()) } == 0 {
        return Err(format!("No response: {}", std::io::Error::last_os_error()));
    }
    let mut code: u32 = 0;
    let mut bytes = std::mem::size_of_val(&code) as u32;
    if unsafe { WinHttpQueryHeaders(request.0,
        WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
        null(), (&mut code as *mut u32).cast(), &mut bytes, null_mut()) } == 0 {
        return Err("Cannot read GitHub response status".to_owned());
    }
    if code != 200 {
        return Err(format!("GitHub returned HTTP {code}"));
    }
    let mut content = Vec::new();
    loop {
        let mut buf = [0u8; 4096];
        let mut read: u32 = 0;
        if unsafe { WinHttpReadData(request.0, buf.as_mut_ptr().cast(),
            buf.len() as u32, &mut read) } == 0 {
            return Err(format!("Read error: {}", std::io::Error::last_os_error()));
        }
        if read == 0 { break; }
        if content.len() + read as usize > MAX_RESPONSE {
            return Err("GitHub response is too large".to_owned());
        }
        content.extend_from_slice(&buf[..read as usize]);
    }
    Ok(content)
}

#[cfg(not(target_os = "windows"))]
fn fetch_latest_json() -> Result<Vec<u8>, String> {
    Err("Update check is supported on Windows only".to_owned())
}

pub fn start(shared: &Arc<Mutex<UpdateState>>, ctx: egui::Context) {
    let mut state = shared.lock().unwrap_or_else(|error| error.into_inner());
    if matches!(*state, UpdateState::Checking) { return; }
    *state = UpdateState::Checking;
    drop(state);
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        let next = check_release().unwrap_or_else(UpdateState::Error);
        *shared.lock().unwrap_or_else(|error| error.into_inner()) = next;
        ctx.request_repaint();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_semantic_version_comparison() {
        assert_eq!(parse_version("0.3.1"), Some(Version(0, 3, 1)));
        assert_eq!(parse_version("v0.4.0"), Some(Version(0, 4, 0)));
        assert_eq!(parse_version("0.3.1-rc1"), None);
        assert_eq!(parse_version("some release"), None);
        assert_eq!(compare_tag(env!("CARGO_PKG_VERSION")).unwrap(), UpdateState::Current);
        assert_eq!(compare_tag("v999.0.0").unwrap(), UpdateState::Available("v999.0.0".into()));
    }
}
