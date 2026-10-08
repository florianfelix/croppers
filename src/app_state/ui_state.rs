pub mod draw;
use crate::app_state::widgets::CropSettings;

#[derive(Default)]
pub struct UiActions {
    pub save_overwrite: bool,
    pub save_increment: bool,
    pub apply_crop: bool,
    pub needs_compute: bool,
    pub needs_rebuild: bool,
    pub request_redraw: bool,
}

pub struct UiState {
    pub canvas_rect: Option<egui::Rect>,
    pub show_panel: bool,
    pub crop: CropSettings,
}

impl UiState {
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
    pub fn set_canvas_rect(&mut self, rect: egui::Rect) {
        self.canvas_rect = Some(rect);
    }
    pub fn canvas_rect(&self) -> &Option<egui::Rect> {
        &self.canvas_rect
    }
}

#[allow(clippy::derivable_impls)]
impl Default for UiState {
    fn default() -> Self {
        Self {
            canvas_rect: None,
            show_panel: true,
            crop: CropSettings::default(),
        }
    }
}
