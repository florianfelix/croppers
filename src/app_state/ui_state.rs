pub mod draw;
pub mod notifications;
use crate::app_state::widgets::CropSettings;
use self::notifications::Notifications;

pub enum GizmoSelection {
    Empty,
    CropHandle(usize),
    VignetteCenter,
}

pub struct UiState {
    pub canvas_rect: Option<egui::Rect>,
    pub show_panel: bool,
    pub show_settings: bool,
    pub active_gizmo: GizmoSelection,
    pub crop: CropSettings,
    pub notifications: Notifications,
    pub pan: egui::Vec2,
    pub zoom: f32,
}

#[derive(Default)]
pub struct UiActions {
    pub save_overwrite: bool,
    pub save_increment: bool,
    pub apply_crop: bool,
    pub needs_compute: bool,
    pub needs_rebuild: bool,
    pub request_redraw: bool,
    pub save_config: bool,
}

impl UiState {
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
    pub fn toggle_settings(&mut self) {
        self.show_settings = !self.show_settings;
    }
    pub fn set_canvas_rect(&mut self, rect: egui::Rect) {
        self.canvas_rect = Some(rect);
    }
    pub fn canvas_rect(&self) -> &Option<egui::Rect> {
        &self.canvas_rect
    }
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan = egui::vec2(0.0, 0.0);
    }
}

#[allow(clippy::derivable_impls)]
impl Default for UiState {
    fn default() -> Self {
        Self {
            canvas_rect: None,
            show_panel: true,
            show_settings: false,
            active_gizmo: GizmoSelection::Empty,
            crop: CropSettings::default(),
            notifications: Notifications::new(),
            pan: egui::vec2(0.0, 0.0),
            zoom: 1.0,
        }
    }
}
