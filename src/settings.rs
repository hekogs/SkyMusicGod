
use std::{collections::{BTreeMap, BTreeSet}, fs, io::{Read, Write}, path::{Path, PathBuf}};

pub const MAX_SETTINGS_BYTES: u64 = 8 * 1024 * 1024;

fn read_bounded_settings(path: &Path) -> Option<Value> {
    let file = fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > MAX_SETTINGS_BYTES { return None; }
    let mut raw = Vec::new();
    file.take(MAX_SETTINGS_BYTES + 1).read_to_end(&mut raw).ok()?;
    if raw.len() as u64 > MAX_SETTINGS_BYTES { return None; }
    let value: Value = serde_json::from_slice(&raw).ok()?;
    value.is_object().then_some(value)
}

use serde_json::{json, Value};
use crate::i18n::Language;
use crate::audio_preview::Instrument;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub speed: f32,
    pub instrument: Instrument,
    pub selected_file: Option<String>,
    pub mini_open: bool,
    pub mini_position: Option<[f32; 2]>,
    pub descending: bool,
    pub dark_theme: bool,
    pub language: Language,
    pub delay_seconds: u64,
    pub auto_next: bool,
    
    pub song_titles: BTreeMap<String, String>,
    pub favorites: BTreeSet<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { speed: 1.0, instrument: Instrument::Guitar, selected_file: None, mini_open: false,
               mini_position: None, descending: false, dark_theme: false, language: Language::Ru, delay_seconds: 4,
               auto_next: false, song_titles: BTreeMap::new(), favorites: BTreeSet::new() }
    }
}

pub fn valid_title(raw: &str) -> Option<String> {
    let title = raw.trim();
    (!title.is_empty() && title.chars().count() <= 80
        && !title.chars().any(char::is_control)).then(|| title.to_owned())
}

impl Settings {
    fn path() -> PathBuf {
        std::env::current_exe().ok()
            .and_then(|exe| exe.parent().map(|parent| parent.join("settings.json")))
            .unwrap_or_else(|| PathBuf::from("settings.json"))
    }

    pub fn from_value(v: &Value) -> Self {
        let mut s = Self::default();
        if let Some(n) = v.get("speed").and_then(Value::as_f64) {
            if n.is_finite() { s.speed = (n as f32).clamp(0.5, 2.0); }
        }
        s.instrument = v.get("instrument").and_then(Value::as_str)
            .and_then(Instrument::from_code).unwrap_or_default();
        s.selected_file = v.get("selected_file").and_then(Value::as_str)
            .filter(|value| !value.is_empty()).map(ToOwned::to_owned);
        s.mini_open = v.get("mini_open").and_then(Value::as_bool).unwrap_or(false);
        s.descending = v.get("descending").and_then(Value::as_bool).unwrap_or(false);
        s.dark_theme = v.get("dark_theme").and_then(Value::as_bool).unwrap_or(false);
        s.language = v.get("language").and_then(Value::as_str).and_then(Language::from_code).unwrap_or(Language::Ru);
        s.auto_next = v.get("auto_next").and_then(Value::as_bool).unwrap_or(false);
        s.delay_seconds = v.get("delay_seconds").and_then(Value::as_u64).unwrap_or(4).min(10);
        if let Some(map) = v.get("song_titles").and_then(Value::as_object) {
            s.song_titles = map.iter().filter_map(|(file, title)| {
                let title = valid_title(title.as_str()?)?;
                if file.is_empty() || file.contains(['/', '\\']) { return None; }
                Some((file.clone(), title))
            }).take(5000).collect();
        }
        if let Some(items) = v.get("favorites").and_then(Value::as_array) {
            s.favorites = items.iter().filter_map(Value::as_str)
                .filter(|name| !name.is_empty() && name.len() <= 255
                    && !name.contains(['/', '\\']) && !name.chars().any(char::is_control))
                .take(5000).map(ToOwned::to_owned).collect();
        }
        s.mini_position = v.get("mini_position").and_then(Value::as_array)
            .filter(|a| a.len() == 2)
            .and_then(|a| Some([
                a[0].as_f64()? as f32,
                a[1].as_f64()? as f32,
            ]))
            .filter(|p| p.iter().all(|v| v.is_finite() && v.abs() < 100000.0));
        s
    }
    pub fn to_value(&self) -> Value {
        json!({
            "schema": 1,
            "speed": self.speed,
            "instrument": self.instrument.code(),
            "selected_file": self.selected_file,
            "mini_open": self.mini_open,
            "mini_position": self.mini_position,
            "descending": self.descending,
            "dark_theme": self.dark_theme,
            "language": self.language.code(),
            "delay_seconds": self.delay_seconds,
            "auto_next": self.auto_next,
            "song_titles": self.song_titles,
            "favorites": self.favorites,
            "send_keys_to_sky": false
        })
    }
    pub fn load() -> Self { Self::load_from(&Self::path()) }

    fn load_from(path: &Path) -> Self {
        let read_valid = |path: &Path| read_bounded_settings(path).map(|value| Self::from_value(&value));
        read_valid(path).or_else(|| read_valid(&path.with_extension("json.bak")))
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> { self.save_to(&Self::path()) }

    fn save_to(&self, file: &Path) -> std::io::Result<()> {
        let temp = file.with_extension("json.tmp");
        let backup = file.with_extension("json.bak");
        let encoded = serde_json::to_vec_pretty(&self.to_value()).map_err(std::io::Error::other)?;
        let mut writer = fs::OpenOptions::new().create(true).truncate(true).write(true).open(&temp)?;
        writer.write_all(&encoded)?;
        writer.sync_all()?;
        drop(writer);

        let result = if file.exists() {
            #[cfg(windows)] {
                
                
                use std::os::windows::ffi::OsStrExt;
                use windows_sys::Win32::Storage::FileSystem::{
                    MoveFileExW, ReplaceFileW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
                };
                let wide = |p: &Path| p.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<u16>>();
                let dst = wide(file);
                let src = wide(&temp);
                
                
                let primary_valid = read_bounded_settings(file).is_some();
                if !primary_valid {
                    if unsafe { MoveFileExW(src.as_ptr(), dst.as_ptr(),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) } == 0 {
                        Err(std::io::Error::last_os_error())
                    } else { Ok(()) }
                } else {
                    if backup.exists() { fs::remove_file(&backup)?; }
                    let bak = wide(&backup);
                    if unsafe { ReplaceFileW(dst.as_ptr(), src.as_ptr(), bak.as_ptr(), 0,
                        std::ptr::null(), std::ptr::null()) } == 0 {
                        Err(std::io::Error::last_os_error())
                    } else { Ok(()) }
                }
            }
            #[cfg(not(windows))] { fs::rename(&temp, file) }
        } else { fs::rename(&temp, file) };
        if result.is_err() { let _ = fs::remove_file(&temp); }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn favorites_round_trip_and_reject_unsafe_filenames() {
        let loaded = Settings::from_value(&json!({
            "favorites": ["KaineSalvation.txt", "Sacred_Grove.txt",
                "KaineSalvation.txt", "../escape.txt", "subdir\\escape.txt", "", 15]
        }));
        assert_eq!(loaded.favorites.len(), 2);
        assert!(loaded.favorites.contains("KaineSalvation.txt"));
        assert!(loaded.favorites.contains("Sacred_Grove.txt"));
        let round_trip = Settings::from_value(&loaded.to_value());
        assert_eq!(round_trip.favorites, loaded.favorites);
        assert!(Settings::from_value(&json!({})).favorites.is_empty());
    }

    #[test]
    fn rejects_oversized_settings_and_recovers_from_backup() {
        let folder = std::env::temp_dir().join(format!("sky-settings-size-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("settings.json");
        let backup = path.with_extension("json.bak");
        fs::write(&backup, r#"{"dark_theme":true,"language":"en"}"#).unwrap();
        fs::write(&path, vec![b' '; MAX_SETTINGS_BYTES as usize + 1]).unwrap();
        assert!(read_bounded_settings(&path).is_none());
        let recovered = Settings::load_from(&path);
        assert!(recovered.dark_theme);
        assert_eq!(recovered.language, Language::En);
        let mut restored = recovered.clone();
        restored.language = Language::Ru;
        restored.save_to(&path).unwrap();
        assert_eq!(Settings::load_from(&path).language, Language::Ru);
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn saves_replaces_and_restores_backup_after_corruption() {
        let folder = std::env::temp_dir().join(format!("sky-settings-hardening-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("settings.json");
        let mut settings = Settings::default();
        settings.dark_theme = true;
        settings.save_to(&path).unwrap();
        settings.language = Language::Ko;
        settings.save_to(&path).unwrap();
        assert_eq!(Settings::load_from(&path).language, Language::Ko);
        assert!(path.with_extension("json.bak").exists());
        fs::write(&path, "{bad json").unwrap();
        let recovered = Settings::load_from(&path);
        assert!(recovered.dark_theme);
        assert_eq!(recovered.language, Language::Ru);
        recovered.save_to(&path).unwrap();
        fs::write(&path, "{bad json again").unwrap();
        
        
        assert!(Settings::load_from(&path).dark_theme);
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn defaults_safe_for_game_input_and_clamps_speed() {
        let s = Settings::from_value(&json!({"speed": 500, "send_keys_to_sky": true,
            "mini_open": true, "selected_file": "test.txt", "mini_position": [210, 110]}));
        assert_eq!(s.speed, 2.0);
        assert!(s.mini_open);
        assert_eq!(s.selected_file.as_deref(), Some("test.txt"));
        assert_eq!(s.mini_position, Some([210.0, 110.0]));
        assert_eq!(s.to_value()["send_keys_to_sky"], false);
        assert_eq!(s.instrument, Instrument::Guitar);
        assert_eq!(Settings::from_value(&json!({"instrument":"harp"})).instrument, Instrument::Harp);
        assert_eq!(Settings::from_value(&json!({"instrument":"Guitar"})).instrument, Instrument::Guitar);
        assert_eq!(Settings::from_value(&json!({"instrument":"piano"})).instrument, Instrument::Guitar);
        assert_eq!(Settings::from_value(&json!({"speed": -4})).speed, 0.5);
        let configured = Settings::from_value(&json!({"dark_theme":true,"delay_seconds":6,"auto_next":true}));
        assert!(configured.dark_theme && configured.auto_next);
        assert_eq!(configured.delay_seconds,6);
        assert_eq!(configured.to_value()["delay_seconds"],6);
        assert_eq!(Settings::from_value(&json!({"delay_seconds":999})).delay_seconds,10);
        let old = Settings::from_value(&json!({"speed":1.5}));
        assert!(old.song_titles.is_empty());
        assert_eq!(old.language, Language::Ru);
        assert_eq!(Settings::from_value(&json!({"language":"ko"})).language, Language::Ko);
        assert_eq!(Settings::from_value(&json!({"language":"unknown"})).language, Language::Ru);
        let named = Settings::from_value(&json!({"song_titles":{
            "Example Text.txt":"  Моя композиция  ", "bad.txt":"   ", "../unsafe.txt":"No"
        }}));
        assert_eq!(named.song_titles.get("Example Text.txt").map(String::as_str), Some("Моя композиция"));
        assert_eq!(named.song_titles.len(), 1);
        assert_eq!(named.to_value()["song_titles"]["Example Text.txt"], "Моя композиция");
        assert!(valid_title("   ").is_none());
        assert!(valid_title(&"x".repeat(81)).is_none());
    }
}
