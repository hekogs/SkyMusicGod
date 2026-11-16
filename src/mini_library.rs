use std::time::{Duration, Instant};
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui};
use crate::{Action, MiniSnapshot};
use crate::mini_view::MiniFoldState;
use crate::ui_tokens as spacing;
use std::sync::{Arc, Mutex};

const PANEL: Color32 = Color32::WHITE;
const TEXT: Color32 = Color32::from_rgb(34, 51, 71);
const MUTED: Color32 = Color32::from_rgb(96, 119, 149);
const BLUE: Color32 = Color32::from_rgb(48, 146, 246);
const PALE_BLUE: Color32 = Color32::from_rgb(231, 243, 255);
const SOFT: Color32 = Color32::from_rgb(242, 245, 249);
const DURATION: Duration = Duration::from_millis(260);
const VISIBLE_ROWS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SongEntry {
    pub key: String,
    pub title: String,
}

pub(super) struct LibraryState {
    pub progress: f32,
    from: f32,
    target: f32,
    started: Option<Instant>,
    pub upward: bool,
    pub mini_origin: Option<egui::Pos2>,
    pub scroll: usize,
    pub favorites_only: bool,
    cached_library: Option<Arc<Vec<SongEntry>>>,
    cached_favorites: Option<Arc<std::collections::BTreeSet<String>>>,
    cached_only: bool,
    cached_matches: Vec<usize>,
    cache_valid: bool,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self { progress: 0.0, from: 0.0, target: 0.0,
            started: None, upward: false, mini_origin: None, scroll: 0, favorites_only: false,
            cached_library: None, cached_favorites: None, cached_only: false,
            cached_matches: Vec::new(), cache_valid: false }
    }
}

impl LibraryState {
    pub fn animating(&self) -> bool { self.started.is_some() }

    pub fn extension(&self) -> f32 {
        spacing::MINI_LIBRARY_EXTENSION * self.progress
    }

    pub fn window_to_mini(&self, position: egui::Pos2) -> egui::Pos2 {
        position + vec2(0.0, if self.upward { self.extension() } else { 0.0 })
    }

    pub fn set_closed(&mut self) {
        if self.target > 0.0 {
            self.from = self.progress;
            self.target = 0.0;
            self.started = Some(Instant::now());
        }
    }

    pub fn toggle(&mut self, mini_origin: Option<egui::Pos2>, scale: f32, selected: Option<usize>) {
        if self.animating() { return; }
        let Some(origin) = mini_origin else { return };
        self.mini_origin = Some(origin);
        if self.target <= 0.0 {
            self.upward = crate::window_shape::mini_library_should_open_upward(
                spacing::MINI_LIBRARY_EXTENSION, scale);
            self.scroll = selected.unwrap_or(0).saturating_sub(2);
        }
        self.from = self.progress;
        self.target = if self.target > 0.0 { 0.0 } else { 1.0 };
        self.started = Some(Instant::now());
    }

    pub fn tick(&mut self) {
        let Some(started) = self.started else { return };
        let time = (started.elapsed().as_secs_f32() / DURATION.as_secs_f32()).min(1.0);
        let eased = time * time * (3.0 - 2.0 * time);
        self.progress = self.from + (self.target - self.from) * eased;
        if time >= 1.0 {
            self.progress = self.target;
            self.started = None;
        }
    }

    fn ensure_matches(&mut self, info: &MiniSnapshot) {
        let library_valid = self.cached_library.as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &info.library));
        let favorites_valid = self.cached_favorites.as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &info.favorites));
        if self.cache_valid && library_valid && favorites_valid
            && self.cached_only == self.favorites_only { return; }
        self.cached_matches.clear();
        self.cached_matches.extend(info.library.iter().enumerate()
            .filter(|(_, song)| !self.favorites_only || info.favorites.contains(&song.key))
            .map(|(index, _)| index));
        self.cached_library = Some(Arc::clone(&info.library));
        self.cached_favorites = Some(Arc::clone(&info.favorites));
        self.cached_only = self.favorites_only;
        self.cache_valid = true;
    }

    pub fn scroll_by(&mut self, delta: f32, total: usize) {
        let max = total.saturating_sub(VISIBLE_ROWS);
        if delta.abs() < 0.5 || max == 0 { return; }
        let steps = (delta.abs() / spacing::MINI_LIBRARY_ROW_HEIGHT).ceil().max(1.0) as usize;
        self.scroll = if delta > 0.0 {
            self.scroll.saturating_sub(steps)
        } else {
            self.scroll.saturating_add(steps).min(max)
        };
    }
}

fn label(ui: &Ui, point: egui::Pos2, value: &str, size: f32, color: Color32, align: Align2) {
    ui.painter().text(point, align, value, FontId::proportional(size), color);
}

pub(super) fn toggle_button(ui: &mut Ui, card: Rect, state: &mut LibraryState, selected: Option<usize>) {
    let center = pos2(card.left() + 28.0, card.top() + 128.0);
    let button = Rect::from_center_size(center, vec2(spacing::SMALL_BUTTON, spacing::SMALL_BUTTON));
    let response = ui.interact(button, ui.scope_id().with("mini_library_toggle"), Sense::click());
    let active = state.target > 0.0;
    let fill = if active || response.hovered() { PALE_BLUE } else { SOFT };
    ui.painter().rect_filled(button, spacing::GAP_SM, crate::theme::shade(ui.ctx(), fill));
    let color = crate::theme::shade(ui.ctx(), if active { BLUE } else { MUTED });
    for y in [-5.0, 0.0, 5.0] {
        ui.painter().line_segment(
            [center + vec2(-7.0, y), center + vec2(5.0, y)],
            Stroke::new(1.7, color));
    }
    if response.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
    if response.clicked() {
        let origin = ui.ctx().input(|i| i.viewport().outer_rect.map(|r| state.window_to_mini(r.min)));
        state.toggle(origin, ui.ctx().pixels_per_point(), selected);
        ui.ctx().request_repaint();
    }
}

pub(super) fn render(
    ui: &mut Ui,
    info: &MiniSnapshot,
    actions: &Arc<Mutex<Vec<Action>>>,
    state: &mut LibraryState,
    fold: &MiniFoldState,
    mini_top: egui::Pos2,
) {
    if state.progress <= 0.0 || fold.progress() >= 1.0 { return; }
    let top = if state.upward {
        mini_top.y - spacing::MINI_LIBRARY_EXTENSION
    } else {
        mini_top.y + crate::mini_view::EXPANDED_SIZE[1] + spacing::MINI_LIBRARY_GAP
    };
    let panel = Rect::from_min_size(
        pos2(mini_top.x, top),
        vec2(crate::mini_view::EXPANDED_SIZE[0], spacing::MINI_LIBRARY_HEIGHT));
    let visible = panel.intersect(ui.clip_rect());
    if visible.is_negative() { return; }
    let enabled = !fold.animating() && !state.animating() && state.progress >= 1.0;
    let alpha = if state.animating() { (state.progress * 2.0).min(1.0) } else { 1.0 };
    ui.scope(|ui| {
        ui.set_clip_rect(visible);
        ui.set_opacity(alpha);
        ui.painter().rect_filled(panel, spacing::CARD_RADIUS, crate::theme::shade(ui.ctx(), PANEL));
        ui.painter().rect_stroke(panel.shrink(0.5), spacing::CARD_RADIUS,
            Stroke::new(0.8, crate::theme::rgb(ui.ctx(), 229, 236, 244)),
            egui::StrokeKind::Inside);
        label(ui, panel.min + vec2(spacing::GAP_MD, 24.0),
            info.language.tr("library"), 13.0,
            crate::theme::shade(ui.ctx(), TEXT), Align2::LEFT_CENTER);
        state.ensure_matches(info);
        let count = info.language.song_count(state.cached_matches.len());
        label(ui, pos2(panel.right() - spacing::INSET, panel.top() + 24.0), &count,
            11.0, crate::theme::shade(ui.ctx(), MUTED), Align2::RIGHT_CENTER);

        for (part, active) in [(0, !state.favorites_only), (1, state.favorites_only)] {
            let width = (panel.width() - 2.0 * spacing::GAP_MD) / 2.0;
            let rect = Rect::from_min_size(
                panel.min + vec2(spacing::GAP_MD + part as f32 * width, 44.0),
                vec2(width, 28.0));
            let resp = ui.interact(rect, ui.scope_id().with(("mini_favorites_filter", part)),
                if enabled { Sense::click() } else { Sense::hover() });
            ui.painter().rect_filled(rect.shrink2(vec2(2.0, 0.0)), 8.0,
                crate::theme::shade(ui.ctx(), if active { PALE_BLUE } else { SOFT }));
            let caption = if part == 0 {
                info.language.tr("favorites_all").to_string()
            } else {
                format!("♥ {}", info.language.tr("favorites_only"))
            };
            label(ui, rect.center(), &caption, 11.0,
                crate::theme::shade(ui.ctx(), if active { BLUE } else { MUTED }), Align2::CENTER_CENTER);
            if enabled && resp.clicked() {
                state.favorites_only = part == 1;
                state.scroll = 0;
            }
        }
        state.ensure_matches(info);
        let matches = std::mem::take(&mut state.cached_matches);
        state.scroll = state.scroll.min(matches.len().saturating_sub(VISIBLE_ROWS));
        if enabled {
            let hover = ui.ctx().input(|i| i.pointer.hover_pos()).is_some_and(|p| panel.contains(p));
            if hover {
                let wheel = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                state.scroll_by(wheel, matches.len());
            }
        }
        if matches.is_empty() {
            label(ui, panel.center(), if state.favorites_only { info.language.tr("favorites_empty") }
                else { info.language.tr("choose_from_library") }, 11.5,
                crate::theme::shade(ui.ctx(), MUTED), Align2::CENTER_CENTER);
        }
        for (index, song_index) in matches.iter().enumerate().skip(state.scroll).take(VISIBLE_ROWS) {
            let entry = &info.library[*song_index];
            let row = Rect::from_min_size(
                panel.min + vec2(spacing::GAP_MD, 80.0 + (index - state.scroll) as f32 * spacing::MINI_LIBRARY_ROW_HEIGHT),
                vec2(panel.width() - 2.0 * spacing::GAP_MD, spacing::MINI_LIBRARY_ROW_HEIGHT));
            let song_area = Rect::from_min_max(row.min, pos2(row.right() - 44.0, row.bottom()));
            let response = if enabled {
                ui.interact(song_area, ui.scope_id().with(("mini_song", &entry.key)), Sense::click())
            } else {
                ui.interact(song_area, ui.scope_id().with(("mini_song", &entry.key)), Sense::hover())
            };
            let selected = info.selected_file.as_deref() == Some(entry.key.as_str());
            if selected || response.hovered() {
                ui.painter().rect_filled(row.shrink2(vec2(0.0, 1.0)), spacing::GAP_SM,
                    crate::theme::shade(ui.ctx(), if selected { PALE_BLUE } else { SOFT }));
            }
            let color = crate::theme::shade(ui.ctx(), if selected { BLUE } else { TEXT });
            let fitted = crate::ui_text::fit(ui, &entry.title, row.width() - 68.0, 12.0, color);
            label(ui, row.min + vec2(spacing::GAP_MD, row.height() / 2.0), &fitted, 12.0, color, Align2::LEFT_CENTER);
            let heart_rect = Rect::from_center_size(
                pos2(row.right() - 24.0, row.center().y), vec2(24.0, 24.0));
            let heart = ui.interact(heart_rect, ui.scope_id().with(("mini_favorite", &entry.key)),
                if enabled { Sense::click() } else { Sense::hover() });
            let is_favorite = info.favorites.contains(&entry.key);
            if heart.hovered() {
                ui.painter().rect_filled(heart_rect, 6.0, crate::theme::shade(ui.ctx(), PALE_BLUE));
            }
            label(ui, heart_rect.center(), if is_favorite { "♥" } else { "♡" }, 17.0,
                if is_favorite { crate::theme::rgb(ui.ctx(), 220, 77, 102) }
                else { crate::theme::shade(ui.ctx(), MUTED) }, Align2::CENTER_CENTER);
            if enabled && heart.clicked() {
                actions.lock().unwrap_or_else(|p| p.into_inner()).push(Action::ToggleFavorite(entry.key.clone()));
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
            if selected {
                let marker = pos2(row.right() - 52.0, row.center().y);
                ui.painter().line_segment([marker + vec2(-4.0, 0.0), marker + vec2(-1.0, 3.0)], Stroke::new(1.8, color));
                ui.painter().line_segment([marker + vec2(-1.0, 3.0), marker + vec2(5.0, -4.0)], Stroke::new(1.8, color));
            }
            if enabled && response.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
            if enabled && response.clicked() {
                actions.lock().unwrap_or_else(|p| p.into_inner()).push(Action::SelectSong(entry.key.clone()));
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
        }
        let overflow = matches.len().saturating_sub(VISIBLE_ROWS);
        if overflow > 0 {
            let bar = Rect::from_min_size(
                pos2(panel.right() - 8.0, panel.top() + 80.0),
                vec2(spacing::GRID, spacing::MINI_LIBRARY_ROW_HEIGHT * VISIBLE_ROWS as f32));
            ui.painter().rect_filled(bar, 2.0, crate::theme::shade(ui.ctx(), SOFT));
            let thumb_h = (bar.height() * VISIBLE_ROWS as f32 / matches.len() as f32)
                .clamp(24.0, bar.height());
            let thumb_y = bar.top() + (bar.height() - thumb_h) * state.scroll as f32 / overflow as f32;
            ui.painter().rect_filled(Rect::from_min_size(pos2(bar.left(), thumb_y), vec2(bar.width(), thumb_h)),
                2.0, crate::theme::shade(ui.ctx(), BLUE));
        }
        state.cached_matches = matches;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolling_stays_in_bounds() {
        let mut state = LibraryState::default();
        state.scroll_by(-2000.0, 18);
        assert_eq!(state.scroll, 14);
        state.scroll_by(2000.0, 18);
        assert_eq!(state.scroll, 0);
        state.scroll_by(-2000.0, 2);
        assert_eq!(state.scroll, 0);
    }

    #[test]
    fn favorites_filter_preserves_song_order() {
        let mut info = MiniSnapshot::default();
        info.library = Arc::new(vec![
            SongEntry { key: "first.txt".into(), title: "First".into() },
            SongEntry { key: "second.txt".into(), title: "Second".into() },
            SongEntry { key: "third.txt".into(), title: "Third".into() },
        ]);
        info.favorites = Arc::new(["third.txt".to_owned(), "first.txt".to_owned()].into_iter().collect());
        let mut state = LibraryState::default();
        state.ensure_matches(&info);
        assert_eq!(state.cached_matches.len(), 3);
        state.favorites_only = true;
        state.ensure_matches(&info);
        let filtered: Vec<&str> = state.cached_matches.iter()
            .map(|index| info.library[*index].key.as_str()).collect();
        assert_eq!(filtered, vec!["first.txt", "third.txt"]);
    }

    #[test]
    fn upward_panel_offset_is_stable() {
        let mut state = LibraryState::default();
        state.upward = true;
        state.progress = 1.0;
        assert_eq!(state.extension(), spacing::MINI_LIBRARY_EXTENSION);
        assert_eq!(state.window_to_mini(pos2(40.0, 50.0)),
            pos2(40.0, 50.0 + spacing::MINI_LIBRARY_EXTENSION));
    }
}
