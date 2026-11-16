
use eframe::egui::{Color32, FontId, Ui};

fn ellipsize_chars(value: &[char], keep: usize) -> String {
    if keep >= value.len() {
        return value.iter().collect();
    }
    if keep == 0 {
        return "…".to_owned();
    }
    let mut result: String = value[..keep].iter().collect();
    result.push('…');
    result
}



pub fn fit(ui: &Ui, label: &str, max_width: f32, font_size: f32, color: Color32) -> String {
    if label.is_empty() || max_width <= 0.0 {
        return String::new();
    }
    let font = FontId::proportional(font_size);
    let fits = |s: &str| {
        ui.painter().layout_no_wrap(s.to_owned(), font.clone(), color).size().x <= max_width
    };
    if fits(label) {
        return label.to_owned();
    }
    let chars: Vec<char> = label.chars().collect();
    if !fits("…") {
        return String::new();
    }
    let mut low = 0usize;
    let mut high = chars.len();
    while low < high {
        let mid = low + (high - low + 1) / 2;
        if fits(&ellipsize_chars(&chars, mid)) {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    ellipsize_chars(&chars, low)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ellipsis_does_not_split_unicode_scalar_values() {
        let chars: Vec<char> = "Звёздный лес 🌟".chars().collect();
        assert_eq!(ellipsize_chars(&chars, 4), "Звёз…");
        assert_eq!(ellipsize_chars(&chars, 0), "…");
        assert_eq!(ellipsize_chars(&chars, chars.len()), "Звёздный лес 🌟");
    }
}
