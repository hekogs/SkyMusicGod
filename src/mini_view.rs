

use std::sync::{Arc, Mutex};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui, ViewportCommand,
};
use crate::{Action, MiniSnapshot, Status};
use crate::ui_tokens as spacing;

const CARD: Color32 = Color32::WHITE;
const TITLE: Color32 = Color32::from_rgb(34, 51, 71);
const MUTED: Color32 = Color32::from_rgb(96, 119, 149);
const BLUE: Color32 = Color32::from_rgb(48, 146, 246);
const PALE_BLUE: Color32 = Color32::from_rgb(231, 243, 255);
const SOFT: Color32 = Color32::from_rgb(242, 245, 249);
const TRACK: Color32 = Color32::from_rgb(229, 236, 246);

pub(super) const EXPANDED_SIZE: [f32; 2] = [382.0, 158.0];
pub(super) const COMPACT_SIZE: [f32; 2] = [48.0, 48.0];
const FOLD_DURATION: std::time::Duration = std::time::Duration::from_millis(260);

pub(super) struct MiniFoldState {
    progress: f32,
    from: f32,
    target: f32,
    started: Option<std::time::Instant>,
    anchor: Option<egui::Pos2>,
    drag_origin: Option<([i32; 2], [i32; 2])>,
}

impl Default for MiniFoldState {
    fn default() -> Self {
        Self { progress: 0.0, from: 0.0, target: 0.0, started: None, anchor: None, drag_origin: None }
    }
}

impl MiniFoldState {
    fn note_rect(origin: egui::Pos2, progress: f32) -> Rect {
        let inset = 12.0 * (1.0 - progress);
        let size = 32.0 + 16.0 * progress;
        Rect::from_min_size(origin + vec2(inset, inset), vec2(size, size))
    }

    fn note_offset(progress: f32) -> egui::Vec2 {
        vec2(28.0 - 4.0 * progress, 28.0 - 4.0 * progress)
    }

    pub(super) fn expanded_origin(&self, window_origin: egui::Pos2) -> egui::Pos2 {
        window_origin + Self::note_offset(self.progress) - Self::note_offset(0.0)
    }

    pub(super) fn animating(&self) -> bool { self.started.is_some() }
    pub(super) fn progress(&self) -> f32 { self.progress }

    pub(super) fn tick(&mut self) {
        let Some(started) = self.started else { return };
        let linear = (started.elapsed().as_secs_f32() / FOLD_DURATION.as_secs_f32()).min(1.0);
        let eased = linear * linear * (3.0 - 2.0 * linear);
        self.progress = self.from + (self.target - self.from) * eased;
        if linear >= 1.0 {
            self.progress = self.target;
            self.started = None;
        }
    }

    pub(super) fn animated_geometry(&self) -> Option<(egui::Vec2, egui::Pos2)> {
        let anchor = self.anchor?;
        let size = vec2(
            EXPANDED_SIZE[0] + (COMPACT_SIZE[0] - EXPANDED_SIZE[0]) * self.progress,
            EXPANDED_SIZE[1] + (COMPACT_SIZE[1] - EXPANDED_SIZE[1]) * self.progress,
        );
        Some((size, anchor - Self::note_offset(self.progress)))
    }

    fn toggle(&mut self, window_origin: Option<egui::Pos2>) {
        if self.animating() { return; }
        let Some(origin) = window_origin else { return };
        self.anchor = Some(origin + Self::note_offset(self.progress));
        self.from = self.progress;
        self.target = if self.progress > 0.5 { 0.0 } else { 1.0 };
        self.started = Some(std::time::Instant::now());
    }
}

#[derive(Clone, Copy)]
enum Control { Previous, Play, Pause, Next }



#[derive(Default)]
pub(super) struct SeekVisualState {
    preview: Option<f32>,
    dragging: bool,
    pending_revision: Option<u64>,
    song: String,
}


fn enqueue(ctx: &egui::Context, actions: &Arc<Mutex<Vec<Action>>>, action: Action) {
    actions.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push(action);
    
    
    ctx.request_repaint_of(egui::ViewportId::ROOT);
}

fn song_labels(song: &str) -> (&str, &str) {
    song.split_once(" - ").unwrap_or((song, "Sky Music God"))
}

fn text(ui: &Ui, at: egui::Pos2, text: impl AsRef<str>, size: f32, color: Color32, align: Align2) {
    ui.painter().text(at, align, text.as_ref(), FontId::proportional(size), color);
}

fn icon_note(ui: &Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, 10.0 + (rect.width() - 32.0) * 0.375, crate::theme::shade(ui.ctx(), PALE_BLUE));
    let center = rect.center();
    let blue = crate::theme::shade(ui.ctx(), BLUE);
    painter.circle_filled(pos2(center.x - 2.7, center.y + 5.0), 3.2, blue);
    painter.line_segment([pos2(center.x, center.y + 4.5), pos2(center.x, center.y - 8.0)], Stroke::new(1.75, blue));
    painter.line_segment([pos2(center.x, center.y - 8.0), pos2(center.x + 5.4, center.y - 5.0)], Stroke::new(1.7, blue));
}

fn control(ui: &mut Ui, center: egui::Pos2, size: f32, kind: Control, id: &str, enabled: bool) -> bool {
    let rect = Rect::from_center_size(center, vec2(size, size));
    let resp = ui.interact(rect, ui.scope_id().with(id), if enabled { Sense::click() } else { Sense::hover() });
    let main = matches!(kind, Control::Play | Control::Pause);
    let fill = if !enabled {
        crate::theme::rgb(ui.ctx(), 238, 241, 246)
    } else if main {
        if resp.hovered() { crate::theme::rgb(ui.ctx(), 31, 130, 231) } else { crate::theme::shade(ui.ctx(), BLUE) }
    } else if resp.hovered() {
        crate::theme::rgb(ui.ctx(), 229, 237, 246)
    } else {
        crate::theme::shade(ui.ctx(), SOFT)
    };
    ui.painter().circle_filled(center, size / 2.0, fill);
    let ink = if !enabled { crate::theme::rgb(ui.ctx(), 187, 199, 215) }
        else if main { Color32::WHITE } else { crate::theme::rgb(ui.ctx(), 94, 120, 154) };
    match kind {
        Control::Previous => {
            ui.painter().add(egui::Shape::convex_polygon(
                vec![center + vec2(4.0, -5.0), center + vec2(-2.0, 0.0), center + vec2(4.0, 5.0)],
                ink, Stroke::NONE,
            ));
            ui.painter().line_segment(
                [center + vec2(-4.0, -5.0), center + vec2(-4.0, 5.0)],
                Stroke::new(1.95, ink),
            );
        }
        Control::Play => {
            ui.painter().add(egui::Shape::convex_polygon(
                vec![center + vec2(-3.1, -5.5), center + vec2(5.2, 0.0), center + vec2(-3.1, 5.5)],
                ink, Stroke::NONE,
            ));
        }
        Control::Pause => {
            ui.painter().circle_stroke(center, 7.4, Stroke::new(1.25, ink));
            ui.painter().rect_filled(
                Rect::from_center_size(center + vec2(-2.5, 0.0), vec2(1.75, 7.5)), 0.6, ink,
            );
            ui.painter().rect_filled(
                Rect::from_center_size(center + vec2(2.5, 0.0), vec2(1.75, 7.5)), 0.6, ink,
            );
        }
        Control::Next => {
            ui.painter().add(egui::Shape::convex_polygon(
                vec![center + vec2(-4.0, -5.0), center + vec2(2.0, 0.0), center + vec2(-4.0, 5.0)],
                ink, Stroke::NONE,
            ));
            ui.painter().line_segment(
                [center + vec2(4.0, -5.0), center + vec2(4.0, 5.0)],
                Stroke::new(1.95, ink),
            );
        }
    }
    resp.clicked() && enabled
}

pub(super) fn render(
    ui: &mut Ui,
    ctx: &egui::Context,
    info: &MiniSnapshot,
    actions: &Arc<Mutex<Vec<Action>>>,
    seek_state: &Arc<Mutex<SeekVisualState>>,
    fold: &mut MiniFoldState,
    library: &mut crate::mini_library::LibraryState,
) {
    let progress = fold.progress;
    let top = if library.upward { library.extension() } else { 0.0 };
    let card = Rect::from_min_size(ui.max_rect().min + vec2(0.0, top),
        vec2(EXPANDED_SIZE[0] + (COMPACT_SIZE[0] - EXPANDED_SIZE[0]) * progress,
             EXPANDED_SIZE[1] + (COMPACT_SIZE[1] - EXPANDED_SIZE[1]) * progress));
    let corner = spacing::WINDOW_RADIUS * (1.0 - progress) + spacing::CARD_RADIUS * progress;
    ui.painter().rect_filled(card, corner, crate::theme::shade(ui.ctx(), CARD));

    let note = MiniFoldState::note_rect(card.min, progress);
    let hit_area = if progress >= 1.0 { card } else { note };
    let response = ui.interact(hit_area, ui.scope_id().with("mini_fold_toggle"), Sense::click_and_drag());
    icon_note(ui, note);
    if response.hovered() {
        ctx.set_cursor_icon(if progress >= 1.0 { egui::CursorIcon::Grab } else { egui::CursorIcon::PointingHand });
    }
    if progress >= 1.0 && response.drag_started() {
        fold.drag_origin = crate::window_shape::mini_drag_start();
    }
    if progress >= 1.0 && response.dragged() {
        if let Some(start) = fold.drag_origin {
            crate::window_shape::mini_drag_update(start);
        }
    }
    if response.drag_stopped() {
        fold.drag_origin = None;
    }
    if response.clicked() {
        library.set_closed();
        fold.toggle(ctx.input(|i| i.viewport().outer_rect.map(|rect| library.window_to_mini(rect.min))));
        ctx.request_repaint();
    }

    if progress < 0.5 {
        ui.scope(|ui| {
            if fold.animating() {
                ui.disable();
                ui.set_opacity((1.0 - progress * 2.0).clamp(0.0, 1.0));
            }
            render_expanded_content(ui, ctx, info, actions, seek_state, library);
        });
    }
    if library.progress > 0.0 && progress < 0.5 {
        crate::mini_library::render(ui, info, actions, library, fold, card.min);
    }
    if fold.animating() {
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn render_expanded_content(
    ui: &mut Ui,
    ctx: &egui::Context,
    info: &MiniSnapshot,
    actions: &Arc<Mutex<Vec<Action>>>,
    seek_state: &Arc<Mutex<SeekVisualState>>,
    library: &mut crate::mini_library::LibraryState,
) {
    let top = if library.upward { library.extension() } else { 0.0 };
    let card = Rect::from_min_size(ui.max_rect().min + vec2(0.0, top), vec2(EXPANDED_SIZE[0], EXPANDED_SIZE[1]));
    let lang = info.language;

    let (original_title, subtitle) = song_labels(&info.song);
    let title = info.title_override.as_deref().unwrap_or(original_title);
    let headline = Rect::from_min_max(
        pos2(card.left() + 52.0, card.top() + 8.0),
        pos2(card.right() - 72.0, card.top() + 52.0),
    );
    let title = crate::ui_text::fit(ui, title, headline.width(), 14.3, crate::theme::shade(ui.ctx(), TITLE));
    let clipped = ui.painter().with_clip_rect(headline);
    clipped.text(
        pos2(headline.left(), card.top() + 24.0),
        Align2::LEFT_CENTER, title, FontId::proportional(14.3), crate::theme::shade(ui.ctx(), TITLE),
    );
    let secondary = if info.countdown > 0 {
        format!("{} {} {}", lang.tr("countdown_prefix"), info.countdown, lang.tr("seconds"))
    } else {
        subtitle.to_string()
    };
    let secondary = crate::ui_text::fit(ui, &secondary, headline.width(), 11.2, crate::theme::shade(ui.ctx(), MUTED));
    clipped.text(
        pos2(headline.left(), card.top() + 44.0),
        Align2::LEFT_CENTER, secondary, FontId::proportional(11.2), crate::theme::shade(ui.ctx(), MUTED),
    );

    
    let title_drag_area = Rect::from_min_max(
        pos2(card.left() + 52.0, card.top() + 4.0),
        pos2(card.right() - 64.0, card.top() + 56.0),
    );
    let title_drag = ui.interact(title_drag_area, ui.scope_id().with("mini_header_drag"), Sense::drag());
    if title_drag.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::Grab);
    }
    if title_drag.drag_started() {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }

    let close_center = pos2(card.right() - 24.0, card.top() + 28.0);
    let close_area = Rect::from_center_size(close_center, vec2(spacing::ICON_BUTTON, spacing::ICON_BUTTON));
    let close_resp = ui.interact(close_area, ui.scope_id().with("mini_close"), Sense::click());
    ui.painter().circle_filled(
        close_center, 12.0,
        if close_resp.hovered() { crate::theme::rgb(ui.ctx(), 226, 233, 242) } else { crate::theme::shade(ui.ctx(), SOFT) },
    );
    ui.painter().line_segment(
        [close_center + vec2(-3.2, -3.2), close_center + vec2(3.2, 3.2)],
        Stroke::new(1.35, crate::theme::shade(ui.ctx(), MUTED)),
    );
    ui.painter().line_segment(
        [close_center + vec2(3.2, -3.2), close_center + vec2(-3.2, 3.2)],
        Stroke::new(1.35, crate::theme::shade(ui.ctx(), MUTED)),
    );
    if close_resp.clicked() {
        enqueue(ctx, actions, Action::CloseMini);
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    let track = Rect::from_min_size(
        pos2(card.left() + spacing::MINI_TRACK_INSET, card.top() + 60.0),
        vec2(card.width() - spacing::MINI_TRACK_INSET * 2.0, spacing::PROGRESS_HEIGHT),
    );
    ui.painter().rect_filled(track, 3.0, crate::theme::shade(ui.ctx(), TRACK));
    let seek_area = track.expand2(vec2(0.0, (spacing::SPEED_HEIGHT - track.height()) / 2.0));
    let seek_response = ui.interact(seek_area, ui.scope_id().with("mini_seek"), Sense::click_and_drag());

    let mut state = seek_state.lock().unwrap_or_else(|e| e.into_inner());
    if state.song != info.song {
        *state = SeekVisualState { song: info.song.clone(), ..Default::default() };
    }
    if !state.dragging && state.pending_revision.is_some_and(|rev| info.seek_revision >= rev) {
        state.preview = None;
        state.pending_revision = None;
    }

    let pointer_fraction = seek_response.interact_pointer_pos()
        .map(|point| ((point.x - track.left()) / track.width()).clamp(0.0, 1.0));
    if seek_response.drag_started() {
        state.dragging = true;
    }
    if (state.dragging || seek_response.clicked()) && pointer_fraction.is_some() {
        state.preview = pointer_fraction;
    }
    let commit = seek_response.drag_stopped() || seek_response.clicked();
    if commit {
        if let Some(fraction) = pointer_fraction.or(state.preview) {
            state.preview = Some(fraction);
            state.pending_revision = Some(info.seek_revision.wrapping_add(1));
            enqueue(ctx, actions, Action::Seek(fraction));
        }
        state.dragging = false;
    }
    
    
    let visible_fraction = state.preview.unwrap_or(info.progress).clamp(0.0, 1.0);
    let filled = track.width() * visible_fraction;
    if filled > 0.0 {
        ui.painter().rect_filled(
            Rect::from_min_size(track.min, vec2(filled, track.height())), 3.0, crate::theme::shade(ui.ctx(), BLUE),
        );
    }
    if seek_response.hovered() || state.dragging {
        ui.painter().circle_filled(
            pos2(track.left() + filled, track.center().y), 6.0, crate::theme::shade(ui.ctx(), BLUE)
        );
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let (actual_position, total) = info.time.split_once(" / ").unwrap_or(("00:00", "00:00"));
    let display_time = if state.preview.is_some() {
        crate::format_time((visible_fraction * info.duration_ms as f32) as u64)
    } else {
        actual_position.to_owned()
    };
    drop(state);
    text(ui, pos2(track.left(), card.top() + 80.0), display_time, 10.1, crate::theme::shade(ui.ctx(), MUTED), Align2::LEFT_CENTER);
    text(ui, pos2(track.right(), card.top() + 80.0), total, 10.1, crate::theme::shade(ui.ctx(), MUTED), Align2::RIGHT_CENTER);
    let status = crate::ui_text::fit(ui, &info.status_line, track.width() - 12.0, 10.0, crate::theme::shade(ui.ctx(), MUTED));
    text(ui, pos2(card.center().x, card.top() + 96.0),
        status, 10.0, crate::theme::shade(ui.ctx(), MUTED), Align2::CENTER_CENTER);

    crate::mini_library::toggle_button(ui, card, library, info.library.iter().position(|entry| Some(entry.key.as_str()) == info.selected_file.as_deref()));
    let buttons_y = card.top() + 128.0;
    let middle = card.center().x;
    if control(ui, pos2(middle - spacing::TRACK_CONTROL_STEP, buttons_y), spacing::SMALL_BUTTON, Control::Previous, "mini_previous", info.has_song) {
        enqueue(ctx, actions, Action::Previous);
    }
    let playing = matches!(info.status, Status::Playing | Status::Armed);
    if control(
        ui, pos2(middle, buttons_y), spacing::MINI_PRIMARY_BUTTON,
        if playing { Control::Pause } else { Control::Play }, "mini_play_pause", info.has_song,
    ) {
        enqueue(ctx, actions, Action::PlayPause);
    }
    if control(ui, pos2(middle + spacing::TRACK_CONTROL_STEP, buttons_y), spacing::SMALL_BUTTON, Control::Next, "mini_next", info.has_song) {
        enqueue(ctx, actions, Action::Next);
    }
    let interval = if matches!(info.status, Status::Playing | Status::Armed) {
        40
    } else {
        120
    };
    ctx.request_repaint_after(std::time::Duration::from_millis(interval));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn title_and_subtitle_are_split_without_losing_song_name() {
        assert_eq!(song_labels("Speak Softly, Love - Godfather Theme"),
            ("Speak Softly, Love", "Godfather Theme"));
        assert_eq!(song_labels("Sacred Grove"),
            ("Sacred Grove", "Sky Music God"));
    }

    #[test]
    fn mini_fold_keeps_note_center_fixed_on_screen() {
        let expanded = pos2(120.0, 80.0);
        let mut fold = MiniFoldState::default();
        assert_eq!(fold.expanded_origin(expanded), expanded);
        fold.toggle(Some(expanded));
        fold.progress = 1.0;
        fold.started = None;
        let (size, compact) = fold.animated_geometry().unwrap();
        assert_eq!(size, vec2(COMPACT_SIZE[0], COMPACT_SIZE[1]));
        assert_eq!(compact, pos2(124.0, 84.0));
        assert_eq!(compact + MiniFoldState::note_offset(1.0), expanded + MiniFoldState::note_offset(0.0));
        assert_eq!(fold.expanded_origin(compact), expanded);

        fold.toggle(Some(compact));
        fold.progress = 0.0;
        fold.started = None;
        let (size, restored) = fold.animated_geometry().unwrap();
        assert_eq!(size, vec2(EXPANDED_SIZE[0], EXPANDED_SIZE[1]));
        assert_eq!(restored, expanded);
    }
}
