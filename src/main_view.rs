

use super::{format_time, MusicApp, Status, Instrument};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Rect, Response, Sense, Stroke, Ui,
    ViewportCommand,
};

use crate::ui_tokens as spacing;

const BACK: Color32 = Color32::from_rgb(247, 249, 252);
const CARD: Color32 = Color32::WHITE;
const TEXT: Color32 = Color32::from_rgb(24, 43, 67);
const SUBTEXT: Color32 = Color32::from_rgb(88, 109, 137);
const CONTROL_INK: Color32 = Color32::from_rgb(88, 115, 150);
const BLUE: Color32 = Color32::from_rgb(49, 146, 245);
const PALE_BLUE: Color32 = Color32::from_rgb(231, 243, 255);
const LINE: Color32 = Color32::from_rgb(229, 236, 244);
const SEARCH_BG: Color32 = Color32::from_rgb(241, 243, 246);

#[derive(Clone, Copy)]
enum Glyph { Plus, Refresh, Gear, Search, Note, Wave, WaveAnimated, Prev, Play, Pause, Next, Min, Max, Close }



const WAVE_HEIGHTS: [f32; 6] = [4.0, 9.7, 16.4, 7.4, 12.3, 4.0];



fn animated_wave_height(bar: usize, time_seconds: f32) -> f32 {
    let offset = bar as f32;
    let motion = (time_seconds * 7.2 + offset * 1.55).sin() * 0.7
        + (time_seconds * 3.8 + offset * 2.35).sin() * 0.3;
    (WAVE_HEIGHTS[bar] * (0.83 + 0.17 * motion)).clamp(2.5, 16.4)
}


fn paint_glyph(ui: &Ui, rect: Rect, icon: Glyph, color: Color32) {
    let painter = ui.painter();
    let c = rect.center();
    let sw = Stroke::new(1.8, color);
    match icon {
        Glyph::Plus => {
            painter.line_segment([pos2(c.x - 4.0, c.y), pos2(c.x + 4.0, c.y)], sw);
            painter.line_segment([pos2(c.x, c.y - 4.0), pos2(c.x, c.y + 4.0)], sw);
        }
        Glyph::Min => { painter.line_segment([pos2(c.x - 4.0, c.y + 1.0), pos2(c.x + 4.0, c.y + 1.0)], sw); }
        Glyph::Max => { painter.rect_stroke(Rect::from_center_size(c, vec2(8.0, 8.0)), 1, Stroke::new(1.0, color), egui::StrokeKind::Inside); }
        Glyph::Close => {
            painter.line_segment([c - vec2(3.5, 3.5), c + vec2(3.5, 3.5)], sw);
            painter.line_segment([c - vec2(3.5, -3.5), c + vec2(3.5, -3.5)], sw);
        }
        Glyph::Search => {
            painter.circle_stroke(c - vec2(1.4, 1.2), 3.3, sw);
            painter.line_segment([c + vec2(1.3, 1.6), c + vec2(5.0, 5.3)], sw);
        }
        Glyph::Gear => crate::ui_icons::paint(ui, rect, crate::ui_icons::Icon::Settings, color),
        Glyph::Refresh => crate::ui_icons::paint(ui, rect, crate::ui_icons::Icon::Refresh, color),
        Glyph::Note => {
            let s = rect.height() / 24.0;
            let head = c + vec2(-2.1 * s, 4.4 * s);
            painter.circle_filled(head, 2.7 * s, color);
            painter.line_segment([head + vec2(2.4 * s, 0.0), head + vec2(2.4 * s, -11.0 * s)], Stroke::new(1.7 * s, color));
            painter.line_segment([head + vec2(2.4 * s, -11.0 * s), head + vec2(7.0 * s, -8.5 * s)], Stroke::new(1.5 * s, color));
        }
        Glyph::Wave | Glyph::WaveAnimated => {
            let time = ui.ctx().input(|input| input.time as f32);
            
            
            for (i, idle_height) in WAVE_HEIGHTS.iter().enumerate() {
                let height = if matches!(icon, Glyph::WaveAnimated) {
                    animated_wave_height(i, time)
                } else {
                    *idle_height
                };
                let x = c.x + (i as f32 - 2.5) * 3.0;
                painter.rect_filled(
                    Rect::from_center_size(pos2(x, c.y), vec2(1.55, height)),
                    0.8, color,
                );
            }
        }
        Glyph::Play => {
            painter.add(egui::Shape::convex_polygon(
                vec![c + vec2(-3.0, -5.3), c + vec2(5.0, 0.0), c + vec2(-3.0, 5.3)],
                color, Stroke::NONE,
            ));
        }
        Glyph::Pause => {
            painter.rect_filled(Rect::from_center_size(c - vec2(2.7, 0.0), vec2(2.5, 10.0)), 1, color);
            painter.rect_filled(Rect::from_center_size(c + vec2(2.7, 0.0), vec2(2.5, 10.0)), 1, color);
        }
        Glyph::Prev | Glyph::Next => {
            
            let dir = if matches!(icon, Glyph::Prev) { -1.0 } else { 1.0 };
            painter.add(egui::Shape::convex_polygon(vec![
                pos2(c.x + dir * 3.4, c.y),
                pos2(c.x - dir * 4.0, c.y - 5.2),
                pos2(c.x - dir * 4.0, c.y + 5.2),
            ], color, Stroke::NONE));
            painter.line_segment([
                pos2(c.x + dir * 5.7, c.y - 5.3),
                pos2(c.x + dir * 5.7, c.y + 5.3),
            ], Stroke::new(2.0, color));
        }
    }
}

fn glyph_button(ui: &mut Ui, rect: Rect, id: &str, glyph: Glyph, tone: Color32, fill: Option<Color32>) -> Response {
    let response = ui.interact(rect, ui.scope_id().with(id), Sense::click());
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, rect.height() / 2.0, if response.hovered() { fill.gamma_multiply(0.91) } else { fill });
    } else if response.hovered() {
        ui.painter().rect_filled(rect, 7, crate::theme::shade(ui.ctx(), PALE_BLUE));
    }
    paint_glyph(ui, rect, glyph, tone);
    response
}

fn write_text(ui: &Ui, p: egui::Pos2, text: impl AsRef<str>, size: f32, color: Color32, align: Align2) {
    ui.painter().text(p, align, text.as_ref(), FontId::proportional(size), color);
}


fn blend_rgb(a: Color32, b: Color32, amount: f32) -> Color32 {
    let t = amount.clamp(0.0, 1.0);
    let [ar, ag, ab, _] = a.to_array();
    let [br, bg, bb, _] = b.to_array();
    let blend = |start: u8, end: u8| (start as f32 + (end as f32 - start as f32) * t).round() as u8;
    Color32::from_rgb(blend(ar, br), blend(ag, bg), blend(ab, bb))
}

fn toggle_row(ui: &mut Ui, rect: Rect, id: &str, text: &str, checked: &mut bool) -> bool {
    let response = ui.interact(rect, ui.scope_id().with(id), Sense::click());
    if response.clicked() { *checked = !*checked; }
    if response.hovered() {
        ui.painter().rect_filled(rect, 8.0, crate::theme::rgb(ui.ctx(), 247, 249, 252));
    }
    let caption = crate::ui_text::fit(ui, text, (rect.width() - spacing::SWITCH_WIDTH - spacing::GAP_SM).max(20.0), 12.5, crate::theme::shade(ui.ctx(), TEXT));
    write_text(ui, pos2(rect.left(), rect.center().y), caption, 12.5, crate::theme::shade(ui.ctx(), TEXT), Align2::LEFT_CENTER);
    let rail = Rect::from_center_size(pos2(rect.right() - spacing::SWITCH_WIDTH / 2.0, rect.center().y), vec2(spacing::SWITCH_WIDTH, spacing::SWITCH_HEIGHT));
    
    
    let progress = ui.ctx().animate_bool_with_time(
        ui.scope_id().with(("ios_toggle", id)), *checked, 0.23);
    let eased = progress * progress * (3.0 - 2.0 * progress);
    let off = crate::theme::rgb(ui.ctx(), 211, 221, 234);
    let on = crate::theme::shade(ui.ctx(), BLUE);
    ui.painter().rect_filled(rail, spacing::SWITCH_HEIGHT / 2.0, blend_rgb(off, on, eased));
    let knob_x = egui::lerp((rail.left() + spacing::SWITCH_HEIGHT / 2.0)..=(rail.right() - spacing::SWITCH_HEIGHT / 2.0), eased);
    let knob = pos2(knob_x, rail.center().y);
    ui.painter().circle_filled(knob + vec2(0.0, 0.6), 9.1,
        Color32::from_rgba_unmultiplied(20, 35, 55, 18));
    ui.painter().circle_filled(knob, 9.0, Color32::WHITE);
    response.clicked()
}

impl MusicApp {
    pub(super) fn render_main(&mut self, ui: &mut Ui) {
        let viewport = ui.max_rect();
        ui.painter().rect_filled(viewport, spacing::WINDOW_RADIUS, crate::theme::shade(ui.ctx(), BACK));
        ui.painter().rect_stroke(viewport.shrink(0.6), spacing::WINDOW_RADIUS,
            Stroke::new(0.8, crate::theme::rgb(ui.ctx(), 235, 240, 247)),
            egui::StrokeKind::Inside);
        let x = viewport.left() + spacing::GAP_MD;
        let y = viewport.top() + spacing::GAP_MD;
        let inner_w = (viewport.width() - spacing::SECTION).max(300.0);

        
        
        if self.delete_candidate.is_some() || self.settings_open || self.rename_candidate.is_some() {
            ui.disable();
        }
        self.draw_title(ui, Rect::from_min_size(pos2(x, y), vec2(inner_w, spacing::HEADER_HEIGHT)));

        let top = y + 48.0;
        let height = (viewport.bottom() - top - spacing::GAP_MD).max(260.0);
        let width = inner_w;
        let left_width = ((width * 0.382) / spacing::GRID).round() * spacing::GRID;
        let left_width = left_width.clamp(240.0, 300.0);
        let left = Rect::from_min_size(pos2(x, top), vec2(left_width, height));
        let right = Rect::from_min_size(pos2(x + left_width + spacing::GAP_SM, top), vec2(width - left_width - spacing::GAP_SM, height));
        ui.painter().rect_filled(left, spacing::CARD_RADIUS, crate::theme::shade(ui.ctx(), CARD));
        ui.painter().rect_filled(right, spacing::CARD_RADIUS, crate::theme::shade(ui.ctx(), CARD));
        self.draw_library(ui, left);
        self.draw_player(ui, right);
        if self.delete_candidate.is_some() {
            self.draw_delete_confirmation(ui.ctx(), viewport);
        } else if self.settings_open {
            self.draw_settings_modal(ui.ctx(), viewport);
        } else if self.rename_candidate.is_some() {
            self.draw_rename_modal(ui.ctx(), viewport);
        }
    }

    fn draw_title(&mut self, ui: &mut Ui, rect: Rect) {
        let icon = Rect::from_min_size(rect.min, vec2(spacing::BUTTON_HEIGHT, spacing::BUTTON_HEIGHT));
        ui.painter().rect_filled(icon, 11, crate::theme::shade(ui.ctx(), BLUE));
        paint_glyph(ui, icon.shrink(6.0), Glyph::Note, Color32::WHITE);
        write_text(ui, pos2(icon.right() + spacing::GAP_SM, icon.center().y), "Sky Music God", 14.0, crate::theme::shade(ui.ctx(), TEXT), Align2::LEFT_CENTER);

        let drag = Rect::from_min_max(rect.min, pos2(rect.right() - 96.0, rect.bottom()));
        if ui.interact(drag, ui.scope_id().with("window_header_drag"), Sense::drag()).drag_started() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        let bar_y = rect.center().y;
        let size = vec2(spacing::ICON_BUTTON, 28.0);
        let close = Rect::from_center_size(pos2(rect.right() - 8.0, bar_y), size);
        let max = Rect::from_center_size(pos2(rect.right() - 36.0, bar_y), size);
        let min = Rect::from_center_size(pos2(rect.right() - 64.0, bar_y), size);
        if glyph_button(ui, min, "window_min", Glyph::Min, crate::theme::shade(ui.ctx(), SUBTEXT), None).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        if glyph_button(ui, max, "window_max", Glyph::Max, crate::theme::shade(ui.ctx(), SUBTEXT), None).clicked() {
            let maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        }
        if glyph_button(ui, close, "window_close", Glyph::Close, crate::theme::shade(ui.ctx(), SUBTEXT), None).clicked() {
            self.stop();
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn draw_library(&mut self, ui: &mut Ui, rect: Rect) {
        let left = rect.left() + spacing::GAP_MD;
        let right = rect.right() - spacing::GAP_MD;
        let top = rect.top();
        write_text(ui, pos2(left, top + 20.0), self.language.tr("library"), 13.3, crate::theme::shade(ui.ctx(), TEXT), Align2::LEFT_CENTER);
        let add = Rect::from_center_size(pos2(right - 8.0, top + 20.0), vec2(spacing::ICON_BUTTON, spacing::ICON_BUTTON));
        if glyph_button(ui, add, "library_import", Glyph::Plus, crate::theme::shade(ui.ctx(), BLUE), None)
            .on_hover_text(self.language.tr("import_song")).clicked() {
            self.import();
        }
        let sort_rect = Rect::from_center_size(pos2(right - 36.0, top + 20.0), vec2(28.0, spacing::ICON_BUTTON));
        let sort_click = ui.interact(sort_rect, ui.scope_id().with("sort_library"), Sense::click());
        if sort_click.hovered() { ui.painter().rect_filled(sort_rect, 6, crate::theme::shade(ui.ctx(), PALE_BLUE)); }
        write_text(ui, sort_rect.center(), if self.language == crate::i18n::Language::Ru {
            if self.descending { "Я–А" } else { "А–Я" }
        } else if matches!(self.language, crate::i18n::Language::Ja | crate::i18n::Language::ZhCn |
                crate::i18n::Language::ZhTw | crate::i18n::Language::Ko) {
            if self.descending { "↓" } else { "↑" }
        } else if self.descending { "Z–A" } else { "A–Z" },
            10.5, crate::theme::shade(ui.ctx(), BLUE), Align2::CENTER_CENTER);
        if sort_click.on_hover_text(self.language.tr("sort_library")).clicked() {
            self.toggle_sort();
        }
        let search = Rect::from_min_max(pos2(left, top + 44.0), pos2(right, top + 84.0));
        ui.painter().rect_filled(search, 20, crate::theme::shade(ui.ctx(), SEARCH_BG));
        paint_glyph(ui, Rect::from_center_size(pos2(search.left() + 16.0, search.center().y), vec2(16.0, 16.0)), Glyph::Search, crate::theme::shade(ui.ctx(), SUBTEXT));
        
        let input = Rect::from_min_max(
            pos2(search.left() + 36.0, search.top() + spacing::GAP_XS),
            pos2(search.right() - spacing::GAP_MD, search.bottom() - spacing::GAP_XS),
        );
        let search_id = ui.scope_id().with("library_search_edit");
        let focused = ui.ctx().memory(|memory| memory.has_focus(search_id));
        let edit = egui::TextEdit::singleline(&mut self.search)
            .id(search_id)
            .hint_text(if focused { "" } else { self.language.tr("search") })
            .frame(egui::Frame::NONE)
            .font(egui::FontId::proportional(12.0))
            .align(egui::Align2::LEFT_CENTER)
            .margin(egui::Margin::ZERO)
            .desired_width(input.width());
        ui.put(input, edit);

        let row_top = search.bottom() + spacing::GAP_MD;
        let row_bottom = rect.bottom() - spacing::ROW_STEP;
        let list_rect = Rect::from_min_max(pos2(left, row_top), pos2(right, row_bottom));
        let row_height = spacing::ROW_STEP;
        let needle = self.search.to_lowercase();
        let matched: Vec<usize> = self.files.iter().enumerate()
            .filter_map(|(i, _)| {
                let name = self.display_file_name(i);
                (needle.is_empty() || name.to_lowercase().contains(&needle)).then_some(i)
            })
            .collect();
        let visible_count = ((list_rect.height() / row_height).floor() as usize).max(1);
        let max_scroll = matched.len().saturating_sub(visible_count) as f32;
        let focused_search = ui.ctx().memory(|m| m.has_focus(ui.scope_id().with("library_search_edit")));
        let hovered = ui.ctx().input(|i| i.pointer.hover_pos().is_some_and(|p| list_rect.contains(p)));
        if hovered {
            let delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            if delta.abs() > 0.05 {
                self.library_scroll -= delta / row_height;
            }
        }
        self.library_scroll = self.library_scroll.clamp(0.0, max_scroll);
        if !focused_search && self.delete_candidate.is_none() && self.rename_candidate.is_none() && !matched.is_empty() {
            let navigation = ui.ctx().input(|i| {
                if i.key_pressed(egui::Key::ArrowDown) { 1 }
                else if i.key_pressed(egui::Key::ArrowUp) { -1 }
                else { 0 }
            });
            if navigation != 0 {
                let current = matched.iter().position(|i| Some(*i) == self.selected)
                    .unwrap_or(if navigation > 0 { matched.len() - 1 } else { 0 });
                let next = (current as i32 + navigation).rem_euclid(matched.len() as i32) as usize;
                self.load(matched[next]);
                if next < self.library_scroll as usize {
                    self.library_scroll = next as f32;
                } else if next >= self.library_scroll as usize + visible_count {
                    self.library_scroll = next.saturating_sub(visible_count - 1) as f32;
                }
            }
        }
        if matched.is_empty() {
            write_text(ui, pos2((left + right) / 2.0, row_top + 35.0),
                self.language.tr("no_results"), 12.3, crate::theme::shade(ui.ctx(), SUBTEXT), Align2::CENTER_CENTER);
        }
        let first = self.library_scroll.floor() as usize;
        let mut draw_y = row_top;
        for index in matched.iter().skip(first).take(visible_count).copied() {
            let row = Rect::from_min_max(pos2(left, draw_y), pos2(right, draw_y + spacing::ROW_HEIGHT));
            let selected = self.selected == Some(index);
            let hovered = ui.interact(row, ui.scope_id().with(("library_song", index)), Sense::click());
            hovered.context_menu(|ui| {
                if ui.button(self.language.tr("rename")).clicked() {
                    self.begin_rename(index);
                    ui.close();
                }
                if ui.button(self.language.tr("delete")).clicked() {
                    self.delete_candidate = Some(index);
                    ui.close();
                }
            });
            if selected {
                ui.painter().rect_filled(row, 9, crate::theme::shade(ui.ctx(), PALE_BLUE));
            } else if hovered.hovered() {
                ui.painter().rect_filled(row, 9, crate::theme::shade(ui.ctx(), SEARCH_BG));
            }
            let icon = Rect::from_center_size(pos2(row.left() + 16.0, row.center().y), vec2(18.0, 18.0));
            let glyph = if selected && self.player.status == Status::Playing {
                Glyph::WaveAnimated
            } else if selected {
                Glyph::Wave
            } else {
                Glyph::Note
            };
            paint_glyph(ui, icon, glyph, if selected { crate::theme::shade(ui.ctx(), BLUE) } else { crate::theme::shade(ui.ctx(), SUBTEXT) });
            let label = crate::ui_text::fit(ui, &self.display_file_name(index), row.width() - 44.0, 12.2, crate::theme::shade(ui.ctx(), TEXT));
            write_text(ui, pos2(row.left() + 32.0, row.center().y), label, 12.2, crate::theme::shade(ui.ctx(), TEXT), Align2::LEFT_CENTER);
            let mut delete_clicked = false;
            if selected {
                let del = Rect::from_center_size(pos2(row.right() - 14.0, row.center().y), vec2(spacing::ICON_BUTTON, spacing::ICON_BUTTON));
                let delete = ui.interact(del, ui.scope_id().with(("delete_song", index)), Sense::click());
                if delete.hovered() { ui.painter().rect_filled(del, 6, crate::theme::rgb(ui.ctx(), 255, 236, 236)); }
                write_text(ui, del.center(), "×", 15.0, if delete.hovered() {
                    crate::theme::rgb(ui.ctx(), 190, 66, 66)
                } else { crate::theme::shade(ui.ctx(), SUBTEXT) }, Align2::CENTER_CENTER);
                if delete.on_hover_text(self.language.tr("delete_song")).clicked() {
                    self.delete_candidate = Some(index);
                    delete_clicked = true;
                }
            }
            if hovered.clicked() && !delete_clicked { self.load(index); }
            draw_y += row_height;
        }
        if max_scroll > 0.0 {
            let rail = Rect::from_min_max(pos2(right + 2.0, row_top), pos2(right + 5.0, row_bottom));
            ui.painter().rect_filled(rail, 2.0, crate::theme::shade(ui.ctx(), SEARCH_BG));
            let height = (rail.height() * visible_count as f32 / matched.len() as f32).max(20.0);
            let top = rail.top() + (rail.height() - height) * (self.library_scroll / max_scroll);
            ui.painter().rect_filled(Rect::from_min_size(pos2(rail.left(), top), vec2(rail.width(), height)),
                2.0, crate::theme::rgb(ui.ctx(), 178, 199, 224));
        }
        let count = self.files.len();
        write_text(ui, pos2(left, rect.bottom() - 20.0), self.language.song_count(count), 10.3, crate::theme::shade(ui.ctx(), SUBTEXT), Align2::LEFT_CENTER);
        if glyph_button(ui, Rect::from_center_size(pos2(right - 38.0, rect.bottom() - 20.0), vec2(24.0, 24.0)), "library_refresh", Glyph::Refresh, crate::theme::shade(ui.ctx(), SUBTEXT), None)
            .on_hover_text(self.language.tr("refresh_library")).clicked() {
            let previous = self.selected.and_then(|i| self.files.get(i).cloned());
            self.refresh();
            self.selected = previous.and_then(|p| self.files.iter().position(|item| item == &p));
        }
        let settings_rect = Rect::from_center_size(
            pos2(right - 8.0, rect.bottom() - 20.0), vec2(24.0, 24.0));
        if glyph_button(ui, settings_rect, "library_settings", Glyph::Gear,
            crate::theme::shade(ui.ctx(), SUBTEXT), None)
            .on_hover_text(self.language.tr("settings")).clicked() {
            self.settings_open = true;
        }
    }

    
    
    fn draw_delete_confirmation(&mut self, ctx: &egui::Context, viewport: Rect) {
        let Some(index) = self.delete_candidate else { return };
        let Some(path) = self.files.get(index) else {
            self.delete_candidate = None;
            return;
        };
        let filename = path.file_stem().unwrap_or_default().to_string_lossy().replace('_', " ");
        let mut cancel = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let mut confirm = false;

        egui::Area::new(egui::Id::unique("delete_confirmation_modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(viewport.min)
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_min_size(viewport.size());
                let painter = ui.painter();
                
                painter.rect_filled(viewport, 0.0, Color32::from_rgba_unmultiplied(36, 51, 73, 48));
                
                ui.interact(viewport, ui.scope_id().with("modal_backdrop"), Sense::click());

                let card = Rect::from_center_size(viewport.center(), vec2(392.0, 196.0));
                painter.rect_filled(card.translate(vec2(0.0, 3.0)).expand(3.0), 19.0,
                    Color32::from_rgba_unmultiplied(60, 84, 116, 21));
                painter.rect_filled(card, 16.0, crate::theme::shade(ui.ctx(), CARD));
                painter.rect_stroke(card, 16.0, Stroke::new(1.0, crate::theme::shade(ui.ctx(), LINE)), egui::StrokeKind::Inside);

                let delete_title = crate::ui_text::fit(ui, self.language.tr("delete_question"), card.width() - 48.0, 16.0, crate::theme::shade(ui.ctx(), TEXT));
                write_text(ui, pos2(card.center().x, card.top() + 33.0),
                    delete_title, 16.0, crate::theme::shade(ui.ctx(), TEXT), Align2::CENTER_CENTER);
                let delete_explainer = crate::ui_text::fit(ui, self.language.tr("delete_explainer"), card.width() - 40.0, 12.8, crate::theme::shade(ui.ctx(), TEXT));
                write_text(ui, pos2(card.center().x, card.top() + 76.0),
                    delete_explainer, 12.8, crate::theme::shade(ui.ctx(), TEXT), Align2::CENTER_CENTER);
                let note = format!("{} {}", self.language.tr("delete_file_prefix"), filename);
                let clipped_note = crate::ui_text::fit(ui, &note, card.width() - 40.0, 11.5, crate::theme::shade(ui.ctx(), SUBTEXT));
                write_text(ui, pos2(card.center().x, card.top() + 99.0),
                    clipped_note, 11.5, crate::theme::shade(ui.ctx(), SUBTEXT), Align2::CENTER_CENTER);

                let divider_y = card.top() + 123.0;
                painter.line_segment(
                    [pos2(card.left() + 20.0, divider_y), pos2(card.right() - 20.0, divider_y)],
                    Stroke::new(1.0, crate::theme::shade(ui.ctx(), LINE)));

                let buttons_y = card.bottom() - 38.0;
                let cancel_rect = Rect::from_center_size(
                    pos2(card.right() - 200.0, buttons_y), vec2(112.0, 36.0));
                let delete_rect = Rect::from_center_size(
                    pos2(card.right() - 80.0, buttons_y), vec2(112.0, 36.0));

                let cancel_click = ui.interact(
                    cancel_rect, ui.scope_id().with("cancel_delete"), Sense::click());
                painter.rect_filled(cancel_rect, 10.0,
                    if cancel_click.hovered() { crate::theme::rgb(ui.ctx(), 235, 241, 248) }
                    else { crate::theme::rgb(ui.ctx(), 245, 247, 250) });
                painter.rect_stroke(cancel_rect, 10.0,
                    Stroke::new(1.0, crate::theme::rgb(ui.ctx(), 221, 230, 241)),
                    egui::StrokeKind::Inside);
                write_text(ui, cancel_rect.center(), self.language.tr("cancel"), 12.7, crate::theme::shade(ui.ctx(), TEXT), Align2::CENTER_CENTER);

                let delete_click = ui.interact(
                    delete_rect, ui.scope_id().with("confirm_delete"), Sense::click());
                painter.rect_filled(delete_rect, 10.0,
                    if delete_click.hovered() { crate::theme::rgb(ui.ctx(), 255, 221, 226) }
                    else { crate::theme::rgb(ui.ctx(), 255, 233, 235) });
                painter.rect_stroke(delete_rect, 10.0,
                    Stroke::new(1.0, crate::theme::rgb(ui.ctx(), 245, 207, 214)),
                    egui::StrokeKind::Inside);
                write_text(ui, delete_rect.center(), self.language.tr("delete"), 12.7,
                    crate::theme::rgb(ui.ctx(), 174, 66, 79), Align2::CENTER_CENTER);
                cancel |= cancel_click.clicked();
                confirm = delete_click.clicked();
            });

        if cancel { self.delete_candidate = None; }
        else if confirm { self.delete_song(index); }
    }

    
    fn draw_settings_modal(&mut self, ctx: &egui::Context, viewport: Rect) {
        let mut close = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        egui::Area::new(egui::Id::unique("player_settings_modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(viewport.min)
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_min_size(viewport.size());
                ui.painter().rect_filled(viewport, 0.0,
                    Color32::from_rgba_unmultiplied(23, 35, 53, 65));
                ui.interact(viewport, ui.scope_id().with("settings_backdrop"), Sense::click());

                let card = Rect::from_center_size(viewport.center(), vec2(440.0, 392.0));
                let bg = crate::theme::shade(ui.ctx(), CARD);
                let line = crate::theme::shade(ui.ctx(), LINE);
                let ink = crate::theme::shade(ui.ctx(), TEXT);
                let muted = crate::theme::shade(ui.ctx(), SUBTEXT);
                let pale = crate::theme::shade(ui.ctx(), PALE_BLUE);

                ui.painter().rect_filled(
                    card.translate(vec2(0.0, 3.0)).expand(2.0), 18.0,
                    Color32::from_rgba_unmultiplied(25, 38, 55, 27));
                ui.painter().rect_filled(card, 16.0, bg);
                ui.painter().rect_stroke(card, 16.0,
                    Stroke::new(1.0, line), egui::StrokeKind::Inside);

                write_text(ui, pos2(card.left() + spacing::SECTION, card.top() + 32.0),
                    self.language.tr("settings"), 17.0, ink, Align2::LEFT_CENTER);
                let cross = Rect::from_center_size(
                    pos2(card.right() - 28.0, card.top() + 32.0), vec2(28.0, 28.0));
                let close_button = ui.interact(
                    cross, ui.scope_id().with("settings_close"), Sense::click());
                if close_button.hovered() { ui.painter().rect_filled(cross, 9.0, pale); }
                paint_glyph(ui, cross, Glyph::Close, muted);
                close |= close_button.clicked();

                ui.painter().line_segment(
                    [pos2(card.left() + spacing::SECTION, card.top() + 56.0),
                     pos2(card.right() - spacing::SECTION, card.top() + 56.0)],
                    Stroke::new(1.0, line));

                let start_x = card.left() + spacing::SECTION;
                let available = card.width() - spacing::SECTION * 2.0;
                let language_y = card.top() + 84.0;
                write_text(ui, pos2(start_x, language_y), self.language.tr("language"), 12.5, ink, Align2::LEFT_CENTER);
                let prev = Rect::from_center_size(pos2(card.right() - 244.0, language_y), vec2(28.0, 28.0));
                let next = Rect::from_center_size(pos2(card.right() - 40.0, language_y), vec2(28.0, 28.0));
                let prev_click = ui.interact(prev, ui.scope_id().with("language_prev"), Sense::click())
                    .on_hover_text(self.language.tr("previous_language"));
                let next_click = ui.interact(next, ui.scope_id().with("language_next"), Sense::click())
                    .on_hover_text(self.language.tr("next_language"));
                for (button, response, arrow) in [(prev, &prev_click, "‹"), (next, &next_click, "›")] {
                    ui.painter().rect_filled(button, 8.0,
                        if response.hovered() { pale } else { crate::theme::shade(ui.ctx(), SEARCH_BG) });
                    write_text(ui, button.center(), arrow, 20.0, ink, Align2::CENTER_CENTER);
                }
                if prev_click.clicked() { self.language = self.language.shifted(-1); }
                if next_click.clicked() { self.language = self.language.shifted(1); }
                let lang_caption = crate::ui_text::fit(ui, self.language.native_name(), 160.0, 12.5, ink);
                write_text(ui, pos2(card.right() - 142.0, language_y), lang_caption, 12.5, ink, Align2::CENTER_CENTER);
                ui.painter().line_segment(
                    [pos2(start_x, card.top() + 112.0),
                     pos2(start_x + available, card.top() + 112.0)],
                    Stroke::new(1.0, line));
                let dark_row = Rect::from_min_size(pos2(start_x, card.top() + 120.0), vec2(available, 40.0));
                toggle_row(ui, dark_row, "settings_dark", self.language.tr("dark_theme"), &mut self.dark_theme);
                write_text(ui, pos2(start_x, card.top() + 168.0),
                    self.language.tr("both_windows"), 10.8, muted, Align2::LEFT_CENTER);
                ui.painter().line_segment(
                    [pos2(start_x, card.top() + 188.0),
                     pos2(start_x + available, card.top() + 188.0)],
                    Stroke::new(1.0, line));
                let button_y = card.top() + 220.0;
                let delay_text = crate::ui_text::fit(ui, self.language.tr("sky_delay"), available - 144.0, 12.5, ink);
                write_text(ui, pos2(start_x, button_y), delay_text, 12.5, ink, Align2::LEFT_CENTER);
                let minus = Rect::from_center_size(pos2(card.right() - 128.0, button_y), vec2(28.0, 28.0));
                let plus = Rect::from_center_size(pos2(card.right() - 40.0, button_y), vec2(28.0, 28.0));
                let minus_hit = ui.interact(minus, ui.scope_id().with("settings_delay_minus"), Sense::click());
                let plus_hit = ui.interact(plus, ui.scope_id().with("settings_delay_plus"), Sense::click());
                for (rect, response, symbol) in [(minus, &minus_hit, "−"), (plus, &plus_hit, "+")] {
                    ui.painter().rect_filled(rect, 8.0,
                        if response.hovered() { pale } else { crate::theme::shade(ui.ctx(), SEARCH_BG) });
                    write_text(ui, rect.center(), symbol, 15.0, ink, Align2::CENTER_CENTER);
                }
                if minus_hit.clicked() { self.delay_seconds = self.delay_seconds.saturating_sub(1); }
                if plus_hit.clicked() { self.delay_seconds = (self.delay_seconds + 1).min(10); }
                write_text(ui, pos2(card.right() - 85.0, button_y),
                    format!("{} {}", self.delay_seconds, self.language.tr("seconds")), 12.6, ink, Align2::CENTER_CENTER);
                let hint = crate::ui_text::fit(ui, self.language.tr("delay_explainer"), available, 10.4, muted);
                write_text(ui, pos2(start_x, card.top() + 248.0), hint, 10.4, muted, Align2::LEFT_CENTER);
                ui.painter().line_segment(
                    [pos2(start_x, card.top() + 268.0),
                     pos2(start_x + available, card.top() + 268.0)],
                    Stroke::new(1.0, line));
                let auto_row = Rect::from_min_size(pos2(start_x, card.top() + 280.0), vec2(available, 40.0));
                toggle_row(ui, auto_row, "settings_auto_next", self.language.tr("auto_next"), &mut self.auto_next);
                write_text(ui, pos2(card.center().x, card.bottom() - 20.0),
                    self.language.tr("auto_save"), 10.5, muted, Align2::CENTER_CENTER);
            });
        if close { self.settings_open = false; }
    }

    
    fn draw_rename_modal(&mut self, ctx: &egui::Context, viewport: Rect) {
        let Some(index) = self.rename_candidate else { return; };
        if self.song_key(index).is_none() {
            self.rename_candidate = None;
            return;
        }
        let mut cancel = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let mut confirm = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        egui::Area::new(egui::Id::unique("rename_song_modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(viewport.min)
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_min_size(viewport.size());
                ui.painter().rect_filled(viewport, 0.0,
                    Color32::from_rgba_unmultiplied(23, 35, 53, 65));
                ui.interact(viewport, ui.scope_id().with("rename_backdrop"), Sense::click());
                let card = Rect::from_center_size(viewport.center(), vec2(416.0, 224.0));
                let bg = crate::theme::shade(ui.ctx(), CARD);
                let line = crate::theme::shade(ui.ctx(), LINE);
                let ink = crate::theme::shade(ui.ctx(), TEXT);
                let muted = crate::theme::shade(ui.ctx(), SUBTEXT);
                let pale = crate::theme::shade(ui.ctx(), PALE_BLUE);
                ui.painter().rect_filled(
                    card.translate(vec2(0.0, 3.0)).expand(2.0), 18.0,
                    Color32::from_rgba_unmultiplied(25, 38, 55, 27));
                ui.painter().rect_filled(card, 16.0, bg);
                ui.painter().rect_stroke(card, 16.0, Stroke::new(1.0, line),
                    egui::StrokeKind::Inside);
                write_text(ui, pos2(card.left() + spacing::SECTION, card.top() + 32.0),
                    self.language.tr("rename_song"), 16.0, ink, Align2::LEFT_CENTER);
                let subtitle = crate::ui_text::fit(ui, self.language.tr("rename_helper"), card.width() - spacing::SECTION * 2.0, 11.7, muted);
                write_text(ui, pos2(card.left() + spacing::SECTION, card.top() + 60.0),
                    subtitle, 11.7, muted, Align2::LEFT_CENTER);

                let field = Rect::from_min_size(
                    pos2(card.left() + spacing::SECTION, card.top() + 84.0),
                    vec2(card.width() - spacing::SECTION * 2.0, 40.0));
                ui.painter().rect_filled(field, 10.0,
                    crate::theme::shade(ui.ctx(), SEARCH_BG));
                let text_id = ui.scope_id().with("rename_song_text_edit");
                if self.rename_focus {
                    ui.ctx().memory_mut(|m| m.request_focus(text_id));
                    self.rename_focus = false;
                }
                
                
                
                let text_rect = field.shrink2(vec2(12.0, 0.0));
                let input = egui::TextEdit::singleline(&mut self.rename_text)
                    .id(text_id)
                    .frame(egui::Frame::NONE)
                    .font(egui::FontId::proportional(13.2))
                    .align(Align2::LEFT_CENTER)
                    .min_size(text_rect.size())
                    .desired_width(text_rect.width());
                let response = ui.put(text_rect, input);
                if response.changed() { self.rename_error.clear(); }

                if self.rename_error.is_empty() {
                    let note = crate::ui_text::fit(ui, self.language.tr("original_preserved"), card.width() - spacing::SECTION * 2.0, 10.6, muted);
                    write_text(ui, pos2(card.left() + spacing::SECTION, card.top() + 136.0),
                        note, 10.6, muted, Align2::LEFT_CENTER);
                } else {
                    let error_tone = crate::theme::rgb(ui.ctx(), 190, 66, 66);
                    let error = crate::ui_text::fit(ui, &self.rename_error, card.width() - spacing::SECTION * 2.0, 10.5, error_tone);
                    write_text(ui, pos2(card.left() + spacing::SECTION, card.top() + 136.0),
                        error, 10.5, error_tone, Align2::LEFT_CENTER);
                }
                let y = card.bottom() - 36.0;
                let cancel_rect = Rect::from_center_size(
                    pos2(card.right() - 200.0, y), vec2(112.0, 36.0));
                let save_rect = Rect::from_center_size(
                    pos2(card.right() - 80.0, y), vec2(112.0, 36.0));
                let no = ui.interact(cancel_rect, ui.scope_id().with("rename_cancel"), Sense::click());
                let yes = ui.interact(save_rect, ui.scope_id().with("rename_save"), Sense::click());
                ui.painter().rect_filled(cancel_rect, 10.0,
                    if no.hovered() { pale } else { crate::theme::shade(ui.ctx(), SEARCH_BG) });
                ui.painter().rect_stroke(cancel_rect, 10.0, Stroke::new(1.0, line),
                    egui::StrokeKind::Inside);
                ui.painter().rect_filled(save_rect, 10.0,
                    if yes.hovered() { crate::theme::rgb(ui.ctx(), 31, 130, 231) }
                    else { crate::theme::shade(ui.ctx(), BLUE) });
                write_text(ui, cancel_rect.center(), self.language.tr("cancel"), 12.8, ink, Align2::CENTER_CENTER);
                write_text(ui, save_rect.center(), self.language.tr("save"), 12.8,
                    Color32::WHITE, Align2::CENTER_CENTER);
                cancel |= no.clicked();
                confirm |= yes.clicked();
            });
        if cancel { self.rename_candidate = None; }
        else if confirm { self.finish_rename(); }
    }

    fn display_file_name(&self, i: usize) -> String {
        self.display_title(i)
    }

    fn draw_player(&mut self, ui: &mut Ui, rect: Rect) {
        let center = rect.center().x;
        let top = rect.top();
        let x = rect.left() + spacing::INSET;
        let width = rect.width() - spacing::INSET * 2.0;
        let art = Rect::from_center_size(pos2(center, top + 36.0), vec2(52.0, 52.0));
        ui.painter().rect_filled(art, spacing::CARD_RADIUS, crate::theme::shade(ui.ctx(), PALE_BLUE));
        paint_glyph(ui, art.shrink(11.0), Glyph::Note, crate::theme::shade(ui.ctx(), BLUE));

        let song_name = self.song.as_ref().map(|s| s.name.as_str()).unwrap_or(self.language.tr("choose_composition"));
        let display_name = self.selected.map(|i| self.display_title(i));
        let title = display_name.as_deref().unwrap_or_else(|| song_name.split(" - ").next().unwrap_or(song_name));
        let subtitle = if let Some(song) = self.song.as_ref() {
            let detail = song.name.split_once(" - ").map(|(_, sub)| sub).unwrap_or(self.language.tr("composition"));
            format!("{} · {} {}", detail, song.notes.len(), self.language.tr("notes"))
        } else {
            self.language.tr("choose_from_library").to_owned()
        };
        let title = crate::ui_text::fit(ui, title, width - spacing::SECTION, 16.0, crate::theme::shade(ui.ctx(), TEXT));
        let subtitle = crate::ui_text::fit(ui, &subtitle, width - spacing::SECTION, 11.7, crate::theme::shade(ui.ctx(), SUBTEXT));
        write_text(ui, pos2(center, top + 76.0), title, 16.0, crate::theme::shade(ui.ctx(), TEXT), Align2::CENTER_CENTER);
        write_text(ui, pos2(center, top + 96.0), subtitle, 11.7, crate::theme::shade(ui.ctx(), SUBTEXT), Align2::CENTER_CENTER);

        let duration = self.song.as_ref().map(|song| song.duration_ms).unwrap_or(1);
        let position = self.song.as_ref().map(|song| self.player.position_ms(song)).unwrap_or(0);
        let fraction = (position as f32 / duration as f32).clamp(0.0, 1.0);
        let track = Rect::from_min_size(pos2(x, top + 112.0), vec2(width, spacing::PROGRESS_HEIGHT));
        ui.painter().rect_filled(track, 3, crate::theme::rgb(ui.ctx(), 225, 234, 245));
        if fraction > 0.0 {
            ui.painter().rect_filled(Rect::from_min_size(track.min, vec2(width * fraction, track.height())), 3, crate::theme::shade(ui.ctx(), BLUE));
        }
        let seek_rect = track.expand2(vec2(0.0, 12.0));
        let seek = ui.interact(seek_rect, ui.scope_id().with("song_seek"), Sense::click_and_drag());
        if seek.hovered() || seek.dragged() {
            ui.painter().circle_filled(
                pos2(track.left() + track.width() * fraction, track.center().y),
                6.4, crate::theme::shade(ui.ctx(), BLUE),
            );
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if (seek.clicked() || seek.dragged()) && self.song.is_some() {
            if let Some(cursor) = seek.interact_pointer_pos() {
                let point = ((cursor.x - track.left()) / track.width()).clamp(0.0, 1.0);
                self.seek_fraction(point);
            }
        }
        write_text(ui, pos2(x, top + 132.0), format_time(self.song.as_ref().map(|song| self.player.position_ms(song)).unwrap_or(0)), 10.7, crate::theme::shade(ui.ctx(), SUBTEXT), Align2::LEFT_CENTER);
        write_text(ui, pos2(x + width, top + 132.0), format_time(if self.song.is_some() { duration } else { 0 }), 10.7, crate::theme::shade(ui.ctx(), SUBTEXT), Align2::RIGHT_CENTER);

        let controls_y = top + 176.0;
        let small = vec2(spacing::BUTTON_HEIGHT, spacing::BUTTON_HEIGHT);
        let ready = self.song.is_some();
        if !ready {
            let disabled = crate::theme::rgb(ui.ctx(), 182, 196, 215);
            ui.painter().circle_filled(pos2(center, controls_y), 21.5, crate::theme::rgb(ui.ctx(), 222, 230, 240));
            for (offset, glyph) in [
                (-spacing::TRACK_CONTROL_STEP, Glyph::Prev),
                (0.0, Glyph::Play),
                (spacing::TRACK_CONTROL_STEP, Glyph::Next)
            ] {
                paint_glyph(ui, Rect::from_center_size(pos2(center + offset, controls_y),
                    vec2(28.0, 28.0)), glyph, if offset == 0.0 { Color32::WHITE } else { disabled });
            }
        }
        if ready && glyph_button(ui, Rect::from_center_size(pos2(center - spacing::TRACK_CONTROL_STEP, controls_y), small), "player_prev", Glyph::Prev, crate::theme::shade(ui.ctx(), CONTROL_INK), None)
            .on_hover_text(self.language.tr("prev_song")).clicked() {
            self.previous();
        }
        let active = matches!(self.player.status, Status::Playing | Status::Armed);
        if ready && glyph_button(ui, Rect::from_center_size(pos2(center, controls_y), vec2(44.0, 44.0)), "player_play_pause", if active { Glyph::Pause } else { Glyph::Play }, Color32::WHITE, Some(crate::theme::shade(ui.ctx(), BLUE)))
            .on_hover_text(self.language.tr(if active { "pause" } else { "play" })).clicked() {
            self.play_pause();
        }
        if ready && glyph_button(ui, Rect::from_center_size(pos2(center + spacing::TRACK_CONTROL_STEP, controls_y), small), "player_next", Glyph::Next, crate::theme::shade(ui.ctx(), CONTROL_INK), None)
            .on_hover_text(self.language.tr("next_song")).clicked() {
            self.advance();
        }

        let separator = top + 220.0;
        ui.painter().line_segment([pos2(x, separator), pos2(x + width, separator)], Stroke::new(1.0, crate::theme::shade(ui.ctx(), LINE)));

        let mini_row = Rect::from_min_size(pos2(x, separator + 16.0), vec2(width, spacing::ROW_HEIGHT));
        toggle_row(ui, mini_row, "toggle_mini", self.language.tr("overlay_player"), &mut self.mini_open);
        let live_row = Rect::from_min_size(pos2(x, separator + 56.0), vec2(width, spacing::ROW_HEIGHT));
        let old_live = self.live_input;
        if toggle_row(ui, live_row, "toggle_live", self.language.tr("send_keys"), &mut self.live_input) && old_live != self.live_input {
            self.stop();
            if !self.live_input { super::windows_input::release_every_key(); }
        }

        write_text(ui, pos2(x, separator + 108.0), self.language.tr("speed"), 11.1, crate::theme::shade(ui.ctx(), TEXT), Align2::LEFT_CENTER);
        let reset = Rect::from_min_max(pos2(x + width - 64.0, separator + 96.0), pos2(x + width, separator + 120.0));
        if ui.interact(reset, ui.scope_id().with("speed_reset"), Sense::click())
            .on_hover_text(self.language.tr("speed_reset")).clicked() {
            self.player.speed = 1.0;
        }
        write_text(ui, pos2(x + width, separator + 108.0), format!("{:.2}×", self.player.speed), 10.8, crate::theme::shade(ui.ctx(), BLUE), Align2::RIGHT_CENTER);
        let slider = Rect::from_min_size(pos2(x, separator + 124.0), vec2(width, spacing::SPEED_HEIGHT));
        ui.painter().rect_filled(slider, 13, crate::theme::rgb(ui.ctx(), 243, 244, 246));
        let response = ui.interact(slider, ui.scope_id().with("speed_slider"), Sense::click_and_drag());
        if response.clicked() || response.dragged() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let normalized = ((pointer.x - slider.left() - 10.0) / (slider.width() - 20.0)).clamp(0.0, 1.0);
                self.player.speed = ((0.5 + 1.5 * normalized) * 20.0).round() / 20.0;
            }
        }
        let adjusted_t = ((self.player.speed - 0.5) / 1.5).clamp(0.0, 1.0);
        let adjusted_x = slider.left() + 10.0 + (slider.width() - 20.0) * adjusted_t;
        
        
        let fill_width = (adjusted_x - slider.left()).max(0.0);
        if fill_width > 0.1 {
            let fill = if ui.ctx().global_style().visuals.dark_mode {
                Color32::from_rgb(65, 133, 201)
            } else {
                Color32::from_rgb(169, 213, 255)
            };
            ui.painter().rect_filled(
                Rect::from_min_size(slider.min, vec2(fill_width, slider.height())),
                (fill_width / 2.0).min(slider.height() / 2.0),
                fill,
            );
        }
        let knob = pos2(adjusted_x, slider.center().y);
        
        
        ui.painter().circle_filled(knob + vec2(0.0, 1.3), spacing::SPEED_THUMB_RADIUS + 0.5,
            Color32::from_rgba_unmultiplied(15, 33, 57, 27));
        ui.painter().circle_filled(knob, spacing::SPEED_THUMB_RADIUS, Color32::WHITE);
        ui.painter().circle_stroke(knob, spacing::SPEED_THUMB_RADIUS,
            Stroke::new(1.0, Color32::from_rgb(198, 211, 228)));

        let band = Rect::from_min_max(pos2(x, rect.bottom() - 52.0), pos2(x + width, rect.bottom() - spacing::GAP_MD));
        ui.painter().rect_filled(band, 9, crate::theme::rgb(ui.ctx(), 238, 246, 255));
        let info = pos2(band.left() + spacing::INSET, band.center().y);
        ui.painter().circle_stroke(info, 5.2, Stroke::new(1.25, crate::theme::shade(ui.ctx(), BLUE)));
        write_text(ui, info + vec2(0.0, 0.4), "i", 9.6, crate::theme::shade(ui.ctx(), BLUE), Align2::CENTER_CENTER);
        let status = self.status_label();
        let status = crate::ui_text::fit(ui, &status, width - 52.0, 11.0, crate::theme::rgb(ui.ctx(), 64, 116, 181));
        write_text(ui, pos2(info.x + spacing::INSET, info.y), status, 11.0, crate::theme::rgb(ui.ctx(), 64, 116, 181), Align2::LEFT_CENTER);
        self.draw_instrument_selector(ui, rect);
    }

    fn draw_instrument_selector(&mut self, ui: &mut Ui, panel: Rect) {
        let id = ui.scope_id().with("instrument_menu");
        let tint = crate::theme::shade(ui.ctx(), TEXT);
        let pale = crate::theme::shade(ui.ctx(), PALE_BLUE);
        let border = crate::theme::shade(ui.ctx(), LINE);
        let background = crate::theme::shade(ui.ctx(), CARD);
        let button = Rect::from_min_size(
            pos2(panel.right() - 144.0, panel.top() + 12.0), vec2(128.0, 28.0));
        let clicked = ui.interact(button, id.with("toggle"), Sense::click());
        if clicked.clicked() { self.instrument_menu_open = !self.instrument_menu_open; }
        ui.painter().rect_filled(button, 8.0,
            if clicked.hovered() || self.instrument_menu_open { pale }
            else { crate::theme::shade(ui.ctx(), SEARCH_BG) });
        ui.painter().rect_stroke(button, 8.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);
        let label = self.language.tr(self.instrument.translation_key());
        let title = crate::ui_text::fit(ui, label, button.width() - 36.0, 11.5, tint);
        write_text(ui, pos2(button.left() + 12.0, button.center().y), title,
            11.5, tint, Align2::LEFT_CENTER);
        let arrow_x = button.right() - 16.0;
        let arrow_y = button.center().y;
        let arrow_sign = if self.instrument_menu_open { -1.0 } else { 1.0 };
        ui.painter().line_segment([pos2(arrow_x - 4.0, arrow_y - 2.0 * arrow_sign),
            pos2(arrow_x, arrow_y + 2.0 * arrow_sign)], Stroke::new(1.6, tint));
        ui.painter().line_segment([pos2(arrow_x, arrow_y + 2.0 * arrow_sign),
            pos2(arrow_x + 4.0, arrow_y - 2.0 * arrow_sign)], Stroke::new(1.6, tint));

        let progress = ui.ctx().animate_bool_with_time(id, self.instrument_menu_open, 0.18);
        if progress < 0.01 { return; }
        ui.ctx().request_repaint();
        let full = Rect::from_min_size(pos2(button.left(), button.bottom() + spacing::GAP_XS),
            vec2(button.width(), 4.0 + Instrument::ALL.len() as f32 * 36.0));
        let visible = Rect::from_min_max(full.min,
            pos2(full.right(), full.top() + full.height() * progress));
        let clip = ui.clip_rect().intersect(visible);
        let painter = ui.painter().with_clip_rect(clip);
        painter.rect_filled(full, 10.0, background);
        painter.rect_stroke(full, 10.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);
        for (index, instrument) in Instrument::ALL.into_iter().enumerate() {
            let row = Rect::from_min_size(
                pos2(full.left() + 4.0, full.top() + 4.0 + index as f32 * 36.0),
                vec2(full.width() - 8.0, 32.0));
            if row.bottom() > visible.bottom() + 1.0 { continue; }
            let hit = ui.interact(row, id.with(("option", index)), Sense::click());
            if hit.hovered() || self.instrument == instrument {
                painter.rect_filled(row, 7.0, pale);
            }
            let row_text = crate::ui_text::fit(ui, self.language.tr(instrument.translation_key()),
                row.width() - 28.0, 11.8, tint);
            painter.text(pos2(row.left() + 8.0, row.center().y),
                Align2::LEFT_CENTER, row_text, FontId::proportional(11.8), tint);
            if self.instrument == instrument {
                painter.text(pos2(row.right() - 12.0, row.center().y),
                    Align2::CENTER_CENTER, "✓", FontId::proportional(13.0),
                    crate::theme::shade(ui.ctx(), BLUE));
            }
            if hit.clicked() {
                self.instrument = instrument;
                self.audio_preview.select(instrument);
                if !self.live_input && self.player.status == Status::Playing {
                    if let Some(song) = self.song.as_ref() {
                        self.audio_preview.play(song, self.player.position_ms(song), self.player.speed);
                    }
                }
                self.audio_error = None;
                self.instrument_menu_open = false;
                self.save_preferences(true);
            }
        }
        let outside = ui.ctx().input(|input| input.pointer.any_pressed()
            && input.pointer.interact_pos().is_some_and(|point| !button.contains(point) && !full.contains(point)));
        if outside && !clicked.clicked() { self.instrument_menu_open = false; }
    }

    pub(super) fn previous(&mut self) {
        if self.files.is_empty() { return; }
        let index = self.selected
            .map(|i| if i == 0 { self.files.len() - 1 } else { i - 1 })
            .unwrap_or(0);
        self.load(index);
    }
}
