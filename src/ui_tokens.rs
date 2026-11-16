


pub const GRID: f32 = 4.0;
pub const WINDOW_RADIUS: f32 = GRID * 5.0;
pub const HEADER_HEIGHT: f32 = GRID * 10.0;
pub const PROGRESS_HEIGHT: f32 = GRID;
pub const ICON_BUTTON: f32 = GRID * 6.0;
pub const MINI_PRIMARY_BUTTON: f32 = GRID * 10.0;
pub const GAP_XS: f32 = GRID;
pub const GAP_SM: f32 = GRID * 2.0;
pub const GAP_MD: f32 = GRID * 3.0;
pub const INSET: f32 = GRID * 4.0;
pub const SECTION: f32 = GRID * 6.0;
pub const CARD_RADIUS: f32 = GRID * 4.0;
pub const ROW_HEIGHT: f32 = GRID * 8.0;
pub const ROW_STEP: f32 = ROW_HEIGHT + GAP_XS;
pub const SWITCH_WIDTH: f32 = GRID * 10.0;
pub const SWITCH_HEIGHT: f32 = GRID * 6.0;
pub const SPEED_HEIGHT: f32 = GRID * 6.0;
pub const SPEED_THUMB_RADIUS: f32 = GRID * 2.5;
pub const BUTTON_HEIGHT: f32 = GRID * 9.0;
pub const SMALL_BUTTON: f32 = GRID * 8.0;
pub const MINI_TRACK_INSET: f32 = GAP_MD;
pub const MINI_LIBRARY_GAP: f32 = GRID;
pub const MINI_LIBRARY_ROW_HEIGHT: f32 = GRID * 9.0;
pub const MINI_LIBRARY_HEIGHT: f32 = GRID * 60.0;
pub const MINI_LIBRARY_EXTENSION: f32 = MINI_LIBRARY_GAP + MINI_LIBRARY_HEIGHT;
pub const TRACK_CONTROL_STEP: f32 = GRID * 12.0;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structural_metrics_follow_four_point_grid() {
        let metrics = [GRID,WINDOW_RADIUS,HEADER_HEIGHT,PROGRESS_HEIGHT,ICON_BUTTON,MINI_PRIMARY_BUTTON,
                       GAP_XS,GAP_SM,GAP_MD,INSET,SECTION,CARD_RADIUS,
                       ROW_HEIGHT,ROW_STEP,SWITCH_WIDTH,SWITCH_HEIGHT,
                       SPEED_HEIGHT,BUTTON_HEIGHT,SMALL_BUTTON,MINI_TRACK_INSET,TRACK_CONTROL_STEP,
                       MINI_LIBRARY_GAP,MINI_LIBRARY_ROW_HEIGHT,MINI_LIBRARY_HEIGHT,MINI_LIBRARY_EXTENSION];
        for n in metrics {
            assert!((n / GRID - (n / GRID).round()).abs() < 1e-6, "{n}");
        }
        assert_eq!(MINI_LIBRARY_EXTENSION, MINI_LIBRARY_HEIGHT + MINI_LIBRARY_GAP);
        assert_eq!(MINI_LIBRARY_ROW_HEIGHT * 5.0 + 44.0 + 16.0, MINI_LIBRARY_HEIGHT);
        assert_eq!(ROW_STEP - ROW_HEIGHT, GAP_XS);
        assert!((32.0..=36.0).contains(&ROW_HEIGHT));
        assert_eq!(SWITCH_HEIGHT, SPEED_HEIGHT);
        assert_eq!(SMALL_BUTTON + GAP_SM, MINI_PRIMARY_BUTTON);
        assert_eq!(TRACK_CONTROL_STEP - (SMALL_BUTTON + MINI_PRIMARY_BUTTON) / 2.0, 12.0);
        assert_eq!(TRACK_CONTROL_STEP - (BUTTON_HEIGHT + 44.0) / 2.0, 8.0);
    }
}
