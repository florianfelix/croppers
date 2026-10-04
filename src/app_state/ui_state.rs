use crate::app_state::widgets::CropSettings;

pub enum GizmoSelection {
    Empty,
    CropHandle(usize), // Top-Left, Top-Right, etc.
    VignetteCenter,
}

pub struct UiState {
    pub canvas_rect: Option<egui::Rect>,
    pub show_panel: bool,
    pub active_gizmo: GizmoSelection,
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
            active_gizmo: GizmoSelection::Empty,
            crop: CropSettings::default(),
        }
    }
}
