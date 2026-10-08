use crate::app_state::widgets::rect_helper::RectHelper;
use egui::{Pos2, Rect};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragTarget {
    None,
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    DrawNew(Pos2),
}
impl DragTarget {
    pub fn current_target(
        crop_rect: &Rect,
        handle_radius: f32,
        pointer_pos: Pos2,
        image_rect: Rect,
    ) -> Self {
        if crop_rect
            .top_left_handle(handle_radius)
            .contains(pointer_pos)
        {
            DragTarget::TopLeft
        } else if crop_rect
            .top_right_handle(handle_radius)
            .contains(pointer_pos)
        {
            DragTarget::TopRight
        } else if crop_rect
            .bottom_left_handle(handle_radius)
            .contains(pointer_pos)
        {
            DragTarget::BottomLeft
        } else if crop_rect
            .bottom_right_handle(handle_radius)
            .contains(pointer_pos)
        {
            DragTarget::BottomRight
        }
        // Edges
        else if crop_rect.top_handle(handle_radius).contains(pointer_pos) {
            DragTarget::Top
        } else if crop_rect.bottom_handle(handle_radius).contains(pointer_pos) {
            DragTarget::Bottom
        } else if crop_rect.right_handle(handle_radius).contains(pointer_pos) {
            DragTarget::Right
        } else if crop_rect.left_handle(handle_radius).contains(pointer_pos) {
            DragTarget::Left
        }
        // Center Body
        else if crop_rect.shrink(handle_radius / 2.).contains(pointer_pos) {
            DragTarget::Center
        } else {
            let start = Pos2 {
                x: pointer_pos.x.clamp(image_rect.min.x, image_rect.max.x),
                y: pointer_pos.y.clamp(image_rect.min.y, image_rect.max.y),
            };
            DragTarget::DrawNew(start)
        }
    }
}
