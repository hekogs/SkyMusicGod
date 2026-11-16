

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
    painter.rect_filled(rect, 10.0, crate::theme::shade(ui.ctx(), PALE_BLUE));
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
) {
    
    
    
    
    let card = ui.max_rect();
    let lang = info.language;
    ui.painter().rect_filled(card, spacing::WINDOW_RADIUS, crate::theme::shade(ui.ctx(), CARD));

    let note = Rect::from_min_size(card.min + vec2(12.0, 12.0), vec2(32.0, 32.0));
    icon_note(ui, note);

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

    
    let grip = pos2(card.right() - 48.0, card.top() + 28.0);
    let grip_area = Rect::from_center_size(grip, vec2(spacing::ICON_BUTTON, spacing::SMALL_BUTTON));
    let grip_hover = ui.interact(grip_area, ui.scope_id().with("mini_grip_drag"), Sense::drag());
    if grip_hover.hovered() {
        ui.painter().circle_filled(grip, 13.0, crate::theme::shade(ui.ctx(), PALE_BLUE));
        ctx.set_cursor_icon(egui::CursorIcon::Grab);
    }
    for dx in [-2.1, 2.1] {
        for dy in [-4.3, 0.0, 4.3] {
            ui.painter().circle_filled(grip + vec2(dx, dy), 1.35, crate::theme::shade(ui.ctx(), MUTED));
        }
    }
    let title_drag_area = Rect::from_min_max(
        pos2(card.left() + 52.0, card.top() + 4.0),
        pos2(card.right() - 64.0, card.top() + 56.0),
    );
    let title_drag = ui.interact(title_drag_area, ui.scope_id().with("mini_header_drag"), Sense::drag());
    if title_drag.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::Grab);
    }
    if grip_hover.drag_started() || title_drag.drag_started() {
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
}
