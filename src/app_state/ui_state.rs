use self::notifications::Notifications;
use crate::app_state::widgets::CropSettings;

pub mod draw_canvas;
pub mod draw_left_panel;
pub mod draw_settings;
pub mod notifications;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    SaveOverwrite,
    SaveIncrement,
    ApplyCrop,
    NeedsCompute,
    NeedsRebuild,
    RequestRedraw,
    SaveConfig,
    OpenFileDialog,
    ToggleSettings,
    ClearEffectStack,
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
