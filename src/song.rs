use std::{fs, io::{Read, Write}, path::Path};


pub const MAX_SONG_BYTES: u64 = 2 * 1024 * 1024;

fn bounded_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    if size > MAX_SONG_BYTES { return Err(format!("Song exceeds {} MiB limit", MAX_SONG_BYTES / 1024 / 1024)); }
    let mut bytes = Vec::with_capacity((size as usize).min(MAX_SONG_BYTES as usize));
    Read::by_ref(&mut file).take(MAX_SONG_BYTES + 1).read_to_end(&mut bytes)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    if bytes.len() as u64 > MAX_SONG_BYTES { return Err("Song grew beyond the import limit".into()); }
    Ok(bytes)
}



pub fn import_song(source: &Path, destination: &Path) -> Result<(), String> {
    let bytes = bounded_bytes(source)?;
    let raw = std::str::from_utf8(&bytes).map_err(|e| format!("Song must be UTF-8 text: {e}"))?;
    let fallback = source.file_stem().unwrap_or_default().to_string_lossy();
    parse_song(raw, &fallback)?;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(destination)
        .map_err(|e| format!("Cannot create {}: {e}", destination.display()))?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(destination);
        return Err(format!("Song import failed: {error}"));
    }
    Ok(())
}

use serde_json::Value;

pub const LABELS: [&str; 15] = [
    "A1", "B1", "C1", "D1", "E1",
    "A2", "B2", "C2", "D2", "E2",
    "A3", "B3", "C3", "D3", "E3",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub time_ms: u64,
    pub key: u8,
    pub hold_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub name: String,
    pub notes: Vec<Note>,
    pub duration_ms: u64,
}

pub fn read_song(path: &Path) -> Result<Song, String> {
    let bytes = bounded_bytes(path)?;
    let contents = std::str::from_utf8(&bytes)
        .map_err(|e| format!("Song is not valid UTF-8: {e}"))?;
    let fallback = path.file_stem().unwrap_or_default().to_string_lossy();
    parse_song(contents, &fallback)
}

pub fn parse_song(input: &str, fallback_name: &str) -> Result<Song, String> {
    if input.len() as u64 > MAX_SONG_BYTES { return Err("Song exceeds 2 MiB limit".into()); }
    let input = input.trim_start_matches('\u{feff}').trim();
    if input.is_empty() {
        return Err("Song file is empty".into());
    }
    if input.starts_with('{') || input.starts_with('[') {
        return parse_json(input, fallback_name);
    }
    parse_text(input, fallback_name)
}

fn parse_json(input: &str, fallback_name: &str) -> Result<Song, String> {
    let value: Value = serde_json::from_str(input)
        .map_err(|e| format!("Invalid song JSON: {e}"))?;
    let item = if let Some(items) = value.as_array() {
        items.first().ok_or("JSON list contains no songs")?
    } else {
        &value
    };
    let name = item.get("name").and_then(Value::as_str).unwrap_or(fallback_name);
    let events = item.get("songNotes")
        .and_then(Value::as_array)
        .ok_or("JSON is missing a songNotes array")?;
    if events.len() > 100_000 { return Err("Too many notes (limit 100000)".into()); }
    let mut notes = Vec::with_capacity(events.len());
    for (index, event) in events.iter().enumerate() {
        let time = event.get("time").and_then(Value::as_u64)
            .ok_or_else(|| format!("Event #{index}: missing non-negative integer time"))?;
        let key_str = event.get("key").and_then(Value::as_str)
            .ok_or_else(|| format!("Event #{index}: missing key"))?;
        let key = parse_key(key_str)
            .ok_or_else(|| format!("Event #{index}: unknown key '{key_str}'"))?;
        let hold_ms = event.get("duration")
            .and_then(Value::as_u64)
            .unwrap_or(95)
            .clamp(15, 2000);
        notes.push(Note { time_ms: time, key, hold_ms });
    }
    finish_song(name.to_string(), notes)
}

fn parse_text(input: &str, fallback_name: &str) -> Result<Song, String> {
    let mut notes = Vec::new();
    for (number, line) in input.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<_> = line.split_whitespace().collect();
        if !(2..=3).contains(&parts.len()) {
            return Err(format!("Line {}: expected 'time note[+note] [duration]'", number + 1));
        }
        let time_ms = parts[0].parse::<u64>()
            .map_err(|_| format!("Line {}: time must be in milliseconds", number + 1))?;
        let hold_ms = if parts.len() == 3 {
            parts[2].parse::<u64>()
                .map_err(|_| format!("Line {}: invalid hold duration", number + 1))?
                .clamp(15, 2000)
        } else {
            95
        };
        for part in parts[1].split(['+', ',']) {
            if notes.len() >= 100_000 { return Err("Too many notes (limit 100000)".into()); }
            let key = parse_key(part)
                .ok_or_else(|| format!("Line {}: unknown note '{part}'", number + 1))?;
            notes.push(Note { time_ms, key, hold_ms });
        }
    }
    finish_song(fallback_name.to_string(), notes)
}

fn parse_key(source: &str) -> Option<u8> {
    let source = source.trim();
    if let Some(index) = LABELS.iter().position(|s| s.eq_ignore_ascii_case(source)) {
        return Some(index as u8);
    }
    let numeric = source.strip_prefix("1Key")
        .or_else(|| source.strip_prefix("Key"))
        .unwrap_or(source);
    let index = numeric.parse::<u8>().ok()?;
    (index < 15).then_some(index)
}

fn finish_song(name: String, mut notes: Vec<Note>) -> Result<Song, String> {
    if notes.is_empty() {
        return Err("No notes found in song".into());
    }
    if notes.len() > 100_000 {
        return Err("Too many notes (limit 100000)".into());
    }
    notes.sort_by_key(|note| note.time_ms);
    let duration_ms = notes.iter()
        .map(|note| note.time_ms.saturating_add(note.hold_ms))
        .max()
        .unwrap_or(0)
        .saturating_add(350);
    if duration_ms > 24 * 60 * 60 * 1000 {
        return Err("Song is longer than 24 hours".into());
    }
    Ok(Song { name, notes, duration_ms })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_text_without_allocating_notes() {
        assert!(parse_song(&"0 A1\n".repeat(500_000), "oversized").is_err());
        let big_json = format!(r#"{{"songNotes":[{}]}}"#, "0,".repeat(1_000_000));
        assert!(parse_song(&big_json, "huge").is_err());
    }

    #[test]
    fn importer_rejects_oversize_and_preserves_existing_data() {
        let folder = std::env::temp_dir().join(format!("sky-song-limits-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let input = folder.join("big.txt");
        let target = folder.join("safe.txt");
        fs::write(&input, vec![b'a'; MAX_SONG_BYTES as usize + 1]).unwrap();
        assert!(read_song(&input).is_err());
        assert!(import_song(&input, &target).is_err());
        assert!(!target.exists());
        fs::write(&input, "0 A1 95").unwrap();
        fs::write(&target, "original").unwrap();
        assert!(import_song(&input, &target).is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "original");
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn reads_sky_export_and_chords() {
        let song = parse_song(r#"[{"name":"Test","songNotes":[{"time":200,"key":"Key9"},{"time":0,"key":"1Key0"},{"time":200,"key":"Key12"}]}]"#, "fallback").unwrap();
        assert_eq!(song.name, "Test");
        assert_eq!(song.notes.iter().map(|n| n.key).collect::<Vec<_>>(), vec![0, 9, 12]);
        assert_eq!(song.notes[1].time_ms, song.notes[2].time_ms);
    }

    #[test]
    fn reads_plain_text() {
        let song = parse_song("0 A1 100\n250 B1+C3 160", "text").unwrap();
        assert_eq!(song.notes.len(), 3);
        assert_eq!(song.notes[2].key, 12);
        assert_eq!(song.notes[1].time_ms, 250);
    }

    #[test]
    fn rejects_bad_notes() {
        assert!(parse_song("0 Key15", "x").is_err());
        assert!(parse_song(r#"{"songNotes":[{"time":1,"key":"oops"}]}"#, "x").is_err());
    }
}


#[cfg(test)]
mod provided_song_tests {
    use super::*;

    #[test]
    fn parses_long_text_sheet() {
        let data = (0..128)
            .map(|step| format!("{} A1+B2 100", step * 800))
            .collect::<Vec<_>>()
            .join("\n");
        let song = parse_song(&data, "Regression Sheet").unwrap();
        assert_eq!(song.name, "Regression Sheet");
        assert_eq!(song.notes.len(), 256);
        assert!(song.duration_ms > 90_000);
        assert!(song.notes.iter().all(|note| note.key < 15));
    }
}
