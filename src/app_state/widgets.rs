pub mod crop_widget;

#[derive(Clone)]
pub struct CropSettings {
    pub enabled: bool,
    pub crop_width: u32,
    pub crop_height: u32,
    pub uv_rect: egui::Rect,
}
impl Default for CropSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            crop_width: 3440,
            crop_height: 1440,
            uv_rect: egui::Rect {
                min: egui::Pos2::ZERO,
                max: egui::Pos2 { x: 1.0, y: 1.0 },
            },
        }
    }
}
