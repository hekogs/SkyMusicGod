#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod player;
mod song;
mod windows_input;
mod main_view;
mod mini_view;
mod ui_text;
mod settings;
mod theme;
mod ui_icons;
mod ui_tokens;
mod i18n;
mod window_shape;
mod audio_preview;

use eframe::egui;

use player::{Player, Status, KeyEvent};
use song::Song;
use i18n::Language;
use audio_preview::{AudioPreview, Instrument};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Copy)]
enum Action { PlayPause, Previous, Next, CloseMini, Seek(f32) }

#[derive(Clone)]
struct MiniSnapshot {
    song: String,
    status: Status,
    progress: f32,
    time: String,
    countdown: u64,
    duration_ms: u64,
    seek_revision: u64,
    status_line: String,
    has_song: bool,
    dark_theme: bool,
    language: Language,
    title_override: Option<String>,
}

impl Default for MiniSnapshot {
    fn default() -> Self {
        Self {
            song: "Композиция не выбрана".into(),
            status: Status::Stopped,
            progress: 0.0,
            time: "00:00 / 00:00".into(),
            countdown: 0,
            duration_ms: 0,
            seek_revision: 0,
            status_line: "Выберите композицию из вашей библиотеки.".into(),
            has_song: false,
            dark_theme: false,
            language: Language::Ru,
            title_override: None,
        }
    }
}

struct MusicApp {
    files: Vec<PathBuf>,
    search: String,
    selected: Option<usize>,
    song: Option<Song>,
    player: Player,
    audio_preview: AudioPreview,
    instrument: Instrument,
    instrument_menu_open: bool,
    audio_error: Option<String>,
    mini_open: bool,
    live_input: bool,
    message: String,
    shared: Arc<Mutex<MiniSnapshot>>,
    actions: Arc<Mutex<Vec<Action>>>,
    mini_seek: Arc<Mutex<mini_view::SeekVisualState>>,
    seek_revision: u64,
    descending: bool,
    library_scroll: f32,
    delete_candidate: Option<usize>,
    finished_recently: bool,
    settings: settings::Settings,
    last_saved: Option<std::time::Instant>,
    mini_position: Arc<Mutex<Option<[f32; 2]>>>,
    mini_needs_position: bool,
    dark_theme: bool,
    language: Language,
    delay_seconds: u64,
    auto_next: bool,
    settings_open: bool,
    song_titles: std::collections::BTreeMap<String, String>,
    rename_candidate: Option<usize>,
    rename_text: String,
    rename_error: String,
    rename_focus: bool,
}

impl MusicApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let preferences = settings::Settings::load();
        cc.egui_ctx.set_theme(if preferences.dark_theme { egui::Theme::Dark } else { egui::Theme::Light });
        let mut player = Player::default();
        player.speed = preferences.speed;
        let mut app = Self {
            files: vec![],
            search: String::new(),
            selected: None,
            song: None,
            player,
            audio_preview: AudioPreview::new(preferences.instrument),
            instrument: preferences.instrument,
            instrument_menu_open: false,
            audio_error: None,
            mini_open: preferences.mini_open,
            live_input: false,
            message: "Ready. Preview is active; game input is OFF.".into(),
            shared: Arc::new(Mutex::new(MiniSnapshot::default())),
            actions: Arc::new(Mutex::new(Vec::new())),
            mini_seek: Arc::new(Mutex::new(mini_view::SeekVisualState::default())),
            seek_revision: 0,
            descending: preferences.descending,
            library_scroll: 0.0,
            delete_candidate: None,
            finished_recently: false,
            settings: preferences.clone(),
            last_saved: None,
            mini_position: Arc::new(Mutex::new(preferences.mini_position)),
            mini_needs_position: true,
            dark_theme: preferences.dark_theme,
            language: preferences.language,
            delay_seconds: preferences.delay_seconds,
            auto_next: preferences.auto_next,
            settings_open: false,
            song_titles: preferences.song_titles.clone(),
            rename_candidate: None,
            rename_text: String::new(),
            rename_error: String::new(),
            rename_focus: false,
        };
        app.refresh();
        if let Some(saved) = preferences.selected_file.as_deref() {
            if let Some(index) = app.files.iter().position(|p| p.file_name().is_some_and(|n| n.to_string_lossy() == saved)) {
                app.load(index);
            }
        }
        app
    }

    fn song_dir() -> PathBuf {
        
        let portable = std::env::current_exe().ok()
            .and_then(|exe| exe.parent().map(|parent| parent.join("songs")));
        if let Some(dir) = portable.as_ref() {
            if dir.is_dir() { return dir.clone(); }
        }
        #[cfg(debug_assertions)]
        {
            let development = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("songs");
            if development.is_dir() { return development; }
        }
        portable.unwrap_or_else(|| PathBuf::from("songs"))
    }

    fn song_key(&self, index: usize) -> Option<String> {
        self.files.get(index)?.file_name().map(|name| name.to_string_lossy().to_string())
    }

    fn title_for_path(path: &std::path::Path, titles: &std::collections::BTreeMap<String, String>) -> String {
        if let Some(label) = path.file_name().and_then(|name| titles.get(&name.to_string_lossy().to_string())) {
            return label.clone();
        }
        path.file_stem().map(|s| s.to_string_lossy().replace('_', " "))
            .unwrap_or_default()
            .split(" - ").next().unwrap_or("").to_owned()
    }

    fn display_title(&self, index: usize) -> String {
        self.files.get(index)
            .map(|path| Self::title_for_path(path, &self.song_titles))
            .unwrap_or_default()
    }

    fn begin_rename(&mut self, index: usize) {
        if self.song_key(index).is_none() { return; }
        self.rename_text = self.display_title(index);
        self.rename_error.clear();
        self.rename_focus = true;
        self.rename_candidate = Some(index);
    }

    fn finish_rename(&mut self) {
        let Some(index) = self.rename_candidate else { return };
        let Some(key) = self.song_key(index) else { self.rename_candidate = None; return };
        let Some(title) = settings::valid_title(&self.rename_text) else {
            self.rename_error = self.language.tr("invalid_title").into();
            return;
        };
        let selected_path = self.selected.and_then(|i| self.files.get(i).cloned());
        self.song_titles.insert(key, title);
        self.sort_files();
        self.selected = selected_path.and_then(|p| self.files.iter().position(|file| file == &p));
        self.rename_candidate = None;
        self.rename_error.clear();
        self.message = self.language.tr("label_saved").into();
        self.save_preferences(true);
    }

    fn sort_files(&mut self) {
        self.files.sort_by(|a,b| {
            let a_name = Self::title_for_path(a, &self.song_titles).to_lowercase();
            let b_name = Self::title_for_path(b, &self.song_titles).to_lowercase();
            a_name.cmp(&b_name).then_with(|| a.cmp(b))
        });
        if self.descending { self.files.reverse(); }
    }

    fn toggle_sort(&mut self) {
        let chosen = self.selected.and_then(|i| self.files.get(i).cloned());
        self.descending = !self.descending;
        self.sort_files();
        self.selected = chosen.and_then(|p| self.files.iter().position(|file| file == &p));
        self.library_scroll = 0.0;
    }

    fn status_label(&self) -> String {
        let lang = self.language;
        if !self.live_input && self.audio_error.is_some() { return lang.tr("audio_unavailable").into(); }
        if self.song.is_none() { return lang.tr("choose_composition_status").into() }
        match self.player.status {
            Status::Armed => format!("{} — {} {}", lang.tr("waiting_sky"), self.player.countdown(), lang.tr("seconds")),
            Status::Playing => lang.tr(if self.live_input { "playing_sky" } else { "playing_preview" }).into(),
            Status::Paused => {
                let key = if self.message.contains("lost focus") || self.message.contains("not the foreground") {
                    "paused_focus"
                } else if self.message.starts_with("Input error") {
                    "paused_input"
                } else { "pause" };
                lang.tr(key).into()
            },
            Status::Finished => lang.tr("finished").into(),
            Status::Stopped => lang.tr(if self.finished_recently { "finished_ready" }
                else if self.live_input { "sky_ready" } else { "preview_ready" }).into(),
        }
    }

    fn current_settings(&self) -> settings::Settings {
        settings::Settings {
            speed: self.player.speed,
            instrument: self.instrument,
            selected_file: self.selected.and_then(|i| self.files.get(i))
                .and_then(|p| p.file_name()).map(|v| v.to_string_lossy().to_string()),
            mini_open: self.mini_open,
            mini_position: *self.mini_position.lock().unwrap_or_else(|p| p.into_inner()),
            descending: self.descending,
            dark_theme: self.dark_theme,
            language: self.language,
            delay_seconds: self.delay_seconds,
            auto_next: self.auto_next,
            song_titles: self.song_titles.clone(),
        }
    }

    fn save_preferences(&mut self, force: bool) {
        let next = self.current_settings();
        if next != self.settings &&
            (force || self.last_saved.is_none_or(|t| t.elapsed() >= Duration::from_secs(1))) {
            if let Err(err) = next.save() {
                self.message = format!("Settings could not be saved: {err}");
            } else {
                self.settings = next;
                self.last_saved = Some(std::time::Instant::now());
            }
        }
    }

    fn refresh(&mut self) {
        let dir = Self::song_dir();
        let _ = fs::create_dir_all(&dir);
        self.files = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok().map(|x| x.path()))
            .filter(|path| path.is_file() && path.extension().is_some_and(|x| {
                let ext = x.to_string_lossy().to_ascii_lowercase();
                ["txt", "json", "skysheet"].contains(&ext.as_str())
            }))
            .collect();
        self.sort_files();
    }

    fn delete_song(&mut self, index: usize) {
        let Some(path) = self.files.get(index).cloned() else { return };
        let preserved = self.selected.filter(|i| *i != index)
            .and_then(|i| self.files.get(i).cloned());
        match fs::remove_file(&path) {
            Ok(()) => {
                if let Some(key) = path.file_name().map(|v| v.to_string_lossy().to_string()) {
                    self.song_titles.remove(&key);
                }
                if self.selected == Some(index) {
                    self.stop();
                    self.song = None;
                    self.selected = None;
                }
                self.refresh();
                self.selected = preserved.and_then(|p| self.files.iter().position(|item| item == &p));
                self.message = "Композиция удалена из библиотеки.".into();
            }
            Err(err) => self.message = format!("Не удалось удалить: {err}"),
        }
        self.delete_candidate = None;
        self.save_preferences(true);
    }

    fn load(&mut self, index: usize) {
        self.stop();
        let Some(path) = self.files.get(index) else { return };
        match song::read_song(path) {
            Ok(song) => {
                self.message = format!("Loaded {} notes.", song.notes.len());
                self.song = Some(song);
                self.selected = Some(index);
                self.finished_recently = false;
            }
            Err(e) => self.message = format!("Import failed: {e}"),
        }
    }

    fn import(&mut self) {
        if let Some(source) = rfd::FileDialog::new()
            .add_filter(self.language.tr("file_filter"), &["txt", "json", "skysheet"])
            .pick_file()
        {
            let Some(filename) = source.file_name() else { return };
            let destination = Self::song_dir().join(filename);
            if destination == source {
                self.refresh();
                return;
            }
            if destination.exists() {
                self.message = "A song with this filename already exists. Rename it before importing.".into();
                return;
            }
            match song::import_song(&source, &destination) {
                Ok(()) => {
                    self.refresh();
                    if let Some(index) = self.files.iter().position(|p| p == &destination) {
                        self.load(index);
                    }
                }
                Err(e) => self.message = format!("File copy failed: {e}"),
            }
        }
    }

    fn stop(&mut self) {
        self.audio_preview.stop();
        self.finished_recently = false;
        let events = self.player.stop();
        self.deliver_release(&events);
    }

    fn deliver_release(&self, events: &[KeyEvent]) {
        if self.live_input && !events.is_empty() {
            
            
            let releases: Vec<_> = events.iter().copied().filter(|event| !event.down).collect();
            if !releases.is_empty() {
                let _ = windows_input::dispatch(&releases);
            }
        }
    }

    fn play_pause(&mut self) {
        self.audio_error = None;
        if self.song.is_none() {
            self.message = "Choose a song first.".into();
            return;
        }
        match self.player.status {
            Status::Playing | Status::Armed => {
                let release = self.player.pause();
                self.audio_preview.stop();
                self.deliver_release(&release);
                self.message = "Paused.".into();
            }
            Status::Paused => {
                self.finished_recently = false;
                self.player.resume(if self.live_input { Duration::from_secs(self.delay_seconds) } else { Duration::ZERO });
                if !self.live_input {
                    if let Some(song) = self.song.as_ref() {
                        self.audio_preview.play(song, self.player.position_ms(song), self.player.speed);
                    }
                }
                self.message = "Resuming.".into();
            }
            Status::Stopped | Status::Finished => {
                self.finished_recently = false;
                self.player.start(if self.live_input { Duration::from_secs(self.delay_seconds) } else { Duration::ZERO });
                if !self.live_input {
                    if let Some(song) = self.song.as_ref() {
                        self.audio_preview.play(song, 0, self.player.speed);
                    }
                }
                self.message = if self.live_input {
                    format!("Switch to Sky within {} seconds. Stops if game loses focus.", self.delay_seconds)
                } else {
                    "Preview playing (no keyboard input).".into()
                };
            }
        }
    }

    fn seek_fraction(&mut self, fraction: f32) {
        let Some(song) = self.song.as_ref() else { return };
        if self.player.status == Status::Armed {
            let released = self.player.pause();
            self.deliver_release(&released);
        }
        let duration = song.duration_ms;
        let target = ((fraction.clamp(0.0, 1.0)) * duration as f32) as u64;
        let was_stopped = self.player.status == Status::Stopped;
        let releases = self.player.seek(song, target);
        self.audio_preview.stop();
        self.deliver_release(&releases);
        if !self.live_input && self.player.status == Status::Playing {
            self.audio_preview.play(song, self.player.position_ms(song), self.player.speed);
        }
        if was_stopped {
            self.player.status = Status::Paused;
        }
        self.seek_revision = self.seek_revision.wrapping_add(1);
        self.message = format!("Position: {}.", format_time(target));
    }

    fn next_auto_index(selected: Option<usize>, count: usize) -> Option<usize> {
        selected.and_then(|index| (index + 1 < count).then_some(index + 1))
    }

    fn advance(&mut self) {
        if self.files.is_empty() { return; }
        let index = self.selected.map(|i| (i + 1) % self.files.len()).unwrap_or(0);
        self.load(index);
    }

    fn process(&mut self) {
        
        
        if self.live_input && self.player.status == Status::Paused
            && (self.message.contains("lost focus") || self.message.contains("not the foreground"))
            && windows_input::sky_is_foreground()
        {
            self.message = "Paused. Sky focus restored; press Play to resume.".into();
        }
        
        
        let _ = windows_input::flush_pending_releases();
        if let Err(error) = self.audio_preview.poll() {
            self.audio_error = Some(error);
            self.audio_preview.stop();
            if !self.live_input && self.player.status == Status::Playing {
                self.player.pause();
            }
        }
        let actions = {
            let mut queue = self.actions.lock().unwrap_or_else(|p| p.into_inner());
            std::mem::take(&mut *queue)
        };
        for action in actions {
            match action {
                Action::PlayPause => self.play_pause(),
                Action::Previous => self.previous(),
                Action::Next => self.advance(),
                Action::CloseMini => {
                    self.mini_open = false;
                    self.mini_needs_position = true;
                },
                Action::Seek(fraction) => self.seek_fraction(fraction),
            }
        }

        let Some(song) = self.song.as_ref() else {
            self.audio_preview.stop();
            let mut empty = MiniSnapshot::default();
            empty.dark_theme = self.dark_theme;
            empty.language = self.language;
            empty.song = self.language.tr("no_song").into();
            empty.status_line = self.language.tr("choose_composition_status").into();
            *self.shared.lock().unwrap_or_else(|p| p.into_inner()) = empty;
            return;
        };
        if self.live_input && self.player.status == Status::Playing && !windows_input::sky_is_foreground() {
            let released = self.player.pause();
            self.deliver_release(&released);
            self.message = "Paused: Sky game window lost focus.".into();
        } else {
            
            let events = self.player.tick(song);
            if self.live_input && self.player.status == Status::Playing {
                if !windows_input::sky_is_foreground() {
                    let released = self.player.pause();
                    self.deliver_release(&released);
                    self.message = "Paused: Sky is not the foreground window.".into();
                } else if !events.is_empty() {
                    if let Err(e) = windows_input::dispatch(&events) {
                        let released = self.player.pause();
                        self.deliver_release(&released);
                        self.message = format!("Input error: {e}");
                    }
                }
            } else if self.live_input && self.player.status == Status::Finished {
                self.deliver_release(&events);
            }
        }
        
        if self.player.status == Status::Finished {
            if let Some(index) = self.auto_next.then(|| Self::next_auto_index(self.selected, self.files.len())).flatten() {
                self.load(index);
                self.play_pause();
            } else {
                self.finished_recently = true;
                self.player.stop();
                self.message = "Song finished. Ready to replay.".into();
            }
        }
        let Some(song) = self.song.as_ref() else { return };
        let duration = song.duration_ms;
        let pos = self.player.position_ms(song);
        let snapshot = MiniSnapshot {
            song: song.name.clone(),
            title_override: self.selected.and_then(|index| self.song_key(index))
                .and_then(|key| self.song_titles.get(&key).cloned()),
            status: self.player.status,
            progress: if duration == 0 { 0.0 } else { pos as f32 / duration as f32 },
            time: format!("{} / {}", format_time(pos), format_time(duration)),
            countdown: self.player.countdown(),
            duration_ms: duration,
            seek_revision: self.seek_revision,
            status_line: self.status_label(),
            has_song: true,
            dark_theme: self.dark_theme,
            language: self.language,
        };
        *self.shared.lock().unwrap_or_else(|p| p.into_inner()) = snapshot;
    }

    fn mini_player(&mut self, ctx: &egui::Context) {
        if !self.mini_open { window_shape::reset_mini(); return; }
        let snapshot = self.shared.clone();
        let actions = self.actions.clone();
        let mini_seek = self.mini_seek.clone();
        let mini_position = self.mini_position.clone();
        let viewport_id = egui::ViewportId::from_hash_of("sky-music-god-mini");
        let mut builder = egui::ViewportBuilder::default()
                .with_title("Sky Music God Mini Player")
                .with_inner_size([382.0, 158.0])
                .with_min_inner_size([382.0, 158.0])
                .with_resizable(false)
                .with_has_shadow(false)
                .with_always_on_top()
                .with_decorations(false)
                .with_transparent(true);
        if self.mini_needs_position {
            if let Some([x, y]) = *self.mini_position.lock().unwrap_or_else(|p| p.into_inner()) {
                builder = builder.with_position(egui::pos2(x, y));
            }
            self.mini_needs_position = false;
        }
        ctx.show_viewport_deferred(
            viewport_id, builder,
            move |ctx, _class| {
                
                window_shape::clip_mini();
                
                if let Some(rect) = ctx.input(|i| i.viewport().outer_rect) {
                    let candidate = [rect.min.x, rect.min.y];
                    if candidate.iter().all(|v| v.is_finite()) {
                        *mini_position.lock().unwrap_or_else(|p| p.into_inner()) = Some(candidate);
                    }
                }
                let info = snapshot.lock().unwrap_or_else(|p| p.into_inner()).clone();
                ctx.set_theme(if info.dark_theme { egui::Theme::Dark } else { egui::Theme::Light });
                let ctx_commands = ctx.clone();
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
                    .show(ctx, |ui| {
                        mini_view::render(ui, &ctx_commands, &info, &actions, &mini_seek);
                    });
            },
        );
    }
}

#[cfg(test)]
mod settings_behaviour_tests {
    use super::MusicApp;
    #[test]
    fn names_are_display_only_and_sort_uses_the_custom_title() {
        use std::{collections::BTreeMap, path::Path};
        let mut titles = BTreeMap::new();
        titles.insert("Sacred_Grove_-_Legend_of_Zelda_Twilight_Princess.txt".to_owned(), "QA Twilight Melody".to_owned());
        let file = Path::new("Sacred_Grove_-_Legend_of_Zelda_Twilight_Princess.txt");
        assert_eq!(MusicApp::title_for_path(file, &titles), "QA Twilight Melody");
        assert_eq!(MusicApp::title_for_path(file, &BTreeMap::new()), "Sacred Grove");
        let mut paths = [
            file.to_path_buf(),
            Path::new("Example_Sky.json").to_path_buf(),
        ];
        paths.sort_by_key(|p| MusicApp::title_for_path(p, &titles).to_lowercase());
        assert_eq!(paths[0].file_name().unwrap().to_string_lossy(), "Example_Sky.json");
    }

    #[test]
    fn auto_next_never_wraps_or_starts_without_a_selection() {
        assert_eq!(MusicApp::next_auto_index(Some(0), 3), Some(1));
        assert_eq!(MusicApp::next_auto_index(Some(1), 3), Some(2));
        assert_eq!(MusicApp::next_auto_index(Some(2), 3), None);
        assert_eq!(MusicApp::next_auto_index(None, 3), None);
        assert_eq!(MusicApp::next_auto_index(Some(0), 1), None);
    }
}

fn format_time(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

impl eframe::App for MusicApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        
        
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.process();
        ctx.set_theme(if self.dark_theme { egui::Theme::Dark } else { egui::Theme::Light });
        let previous_mini = self.mini_open;
        self.render_main(ui);
        if !self.live_input && self.player.status == Status::Playing {
            if let Some(song) = self.song.as_ref() {
                self.audio_preview.update_speed(self.player.position_ms(song), self.player.speed);
            }
        }
        if previous_mini != self.mini_open { self.mini_needs_position = true; }
        self.mini_player(&ctx);
        self.save_preferences(false);
        
        
        if self.instrument_menu_open
            || (self.live_input && matches!(self.player.status, Status::Playing | Status::Armed)) {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if matches!(self.player.status, Status::Playing | Status::Armed) {
            ctx.request_repaint_after(Duration::from_millis(32));
        } else if windows_input::has_pending_releases()
            || (self.live_input && self.player.status == Status::Paused
                && (self.message.contains("lost focus") || self.message.contains("not the foreground"))) {
            
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }
}

impl Drop for MusicApp {
    fn drop(&mut self) {
        self.save_preferences(true);
        
        if self.live_input { windows_input::release_every_key(); }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Sky Music God")
            .with_inner_size([736.0, 500.0])
            .with_min_inner_size([736.0, 500.0])
            .with_has_shadow(false)
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "Sky Music God",
        options,
        Box::new(|cc| Ok(Box::new(MusicApp::new(cc)))),
    )
}
