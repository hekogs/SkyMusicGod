



use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Ui};

#[derive(Clone, Copy)]
pub enum Icon {
    Refresh,
    Settings,
}

const SIZE: usize = 64;

fn rle_alpha(bytes: &[u8]) -> Vec<u8> {
    let mut alpha = Vec::with_capacity(SIZE * SIZE);
    for pair in bytes.chunks_exact(2) {
        alpha.extend(std::iter::repeat_n(pair[1], pair[0] as usize));
    }
    assert_eq!(alpha.len(), SIZE * SIZE, "Corrupted embedded icon");
    alpha
}

fn image_for(icon: Icon) -> ColorImage {
    let encoded: &[u8] = match icon {
        Icon::Refresh => include_bytes!("../assets/refresh-cw.rle"),
        Icon::Settings => include_bytes!("../assets/settings.rle"),
    };
    let rgba: Vec<u8> = rle_alpha(encoded)
        .into_iter().flat_map(|a| [255,255,255,a]).collect();
    ColorImage::from_rgba_unmultiplied([SIZE,SIZE], &rgba)
}

fn texture(ui: &Ui, icon: Icon) -> TextureHandle {
    let id = egui::Id::unique(match icon {
        Icon::Refresh => "sky-refresh-cw-svg",
        Icon::Settings => "sky-settings-svg",
    });
    if let Some(existing) = ui.ctx().data(|d| d.get_temp::<TextureHandle>(id)) {
        return existing;
    }
    let name = match icon {
        Icon::Refresh => "Lucide Refresh CW (user SVG)",
        Icon::Settings => "Lucide Settings (user SVG)",
    };
    let handle = ui.ctx().load_texture(name, image_for(icon), TextureOptions::LINEAR);
    ui.ctx().data_mut(|d| d.insert_temp(id, handle.clone()));
    handle
}

pub fn paint(ui: &Ui, area: Rect, icon: Icon, tint: Color32) {
    let handle = texture(ui, icon);
    let target = Rect::from_center_size(area.center(), egui::vec2(17.5,17.5));
    ui.painter().image(handle.id(), target,
        Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0,1.0)), tint);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resources_decode_to_64_by_64_transparent_icons() {
        for icon in [Icon::Refresh, Icon::Settings] {
            let image = image_for(icon);
            assert_eq!(image.size, [64,64]);
            let pixels = &image.pixels;
            assert!(pixels.iter().any(|p| p.a() == 0));
            assert!(pixels.iter().any(|p| p.a() == 255));
        }
    }
}
