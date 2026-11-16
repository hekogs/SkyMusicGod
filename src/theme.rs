
use eframe::egui::{Color32, Context};

pub fn shade(ctx: &Context, base: Color32) -> Color32 {
    if !ctx.global_style().visuals.dark_mode { return base; }
    let [r,g,b,a] = base.to_array();
    let (rr,gg,bb): (u8,u8,u8) = match (r,g,b) {
        (255,255,255) => (31,40,56),    
        (247,249,252) => (19,25,36),    
        (24,43,67) => (238,243,252),    
        (34,51,71) => (238,243,252),    
        (88,109,137) => (162,181,205), 
        (88,115,150) => (177,197,220), 
        (96,119,149) => (168,185,210), 
        (49,146,245) | (48,146,246) => (72,161,255),
        (231,243,255) => (39,61,87),
        (229,236,244) | (229,236,246) => (62,76,99),
        (241,243,246) | (242,245,249) => (37,48,65),
        _ if r > 225 && g > 225 && b > 225 => (43,54,71),
        _ if r > 195 && g > 195 && b > 195 => (58,74,94),
        _ if r < 160 && g < 160 && b < 170 => (192,207,226),
        _ => (r,g,b),
    };
    Color32::from_rgba_unmultiplied(rr,gg,bb,a)
}
pub fn rgb(ctx: &Context, r:u8,g:u8,b:u8) -> Color32 {
    shade(ctx, Color32::from_rgb(r,g,b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_light_palette_and_makes_dark_card() {
        let ctx=Context::default();
        ctx.set_theme(eframe::egui::Theme::Light);
        assert_eq!(shade(&ctx,Color32::WHITE),Color32::WHITE);
        ctx.set_theme(eframe::egui::Theme::Dark);
        assert_eq!(shade(&ctx,Color32::WHITE),Color32::from_rgb(31,40,56));
        assert_ne!(rgb(&ctx,247,249,252),Color32::from_rgb(247,249,252));
    }
}
