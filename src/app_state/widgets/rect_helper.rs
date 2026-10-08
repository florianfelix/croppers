use egui::{Pos2, Rect, Vec2};
pub trait RectHelper {
    fn top_handle(&self, size: f32) -> Rect;
    fn bottom_handle(&self, size: f32) -> Rect;
    fn right_handle(&self, size: f32) -> Rect;
    fn left_handle(&self, size: f32) -> Rect;
    fn top_left_handle(&self, size: f32) -> Rect;
    fn top_right_handle(&self, size: f32) -> Rect;
    fn bottom_left_handle(&self, size: f32) -> Rect;
    fn bottom_right_handle(&self, size: f32) -> Rect;
}

impl RectHelper for Rect {
    fn top_handle(&self, size: f32) -> Rect {
        let min = Pos2 {
            x: self.min.x + size,
            y: self.min.y - size / 2.0,
        };
        let max = Pos2 {
            x: self.min.x + self.width() - size,
            y: self.min.y + size / 2.0,
        };
        Rect { min, max }
    }
    fn bottom_handle(&self, size: f32) -> Rect {
        let min = Pos2 {
            x: self.max.x - self.width() + size,
            y: self.max.y - size / 2.0,
        };
        let max = Pos2 {
            x: self.max.x - size,
            y: self.max.y + size / 2.0,
        };
        Rect { min, max }
    }

    fn right_handle(&self, size: f32) -> Rect {
        let min = Pos2 {
            x: self.max.x - size / 2.0,
            y: self.max.y - self.height() + size,
        };
        let max = Pos2 {
            x: self.max.x + size / 2.0,
            y: self.max.y - size,
        };
        Rect { min, max }
    }

    fn left_handle(&self, size: f32) -> Rect {
        let min = Pos2 {
            x: self.min.x - size / 2.0,
            y: self.min.y + size,
        };
        let max = Pos2 {
            x: self.min.x + size / 2.0,
            y: self.min.y + self.height() - size,
        };
        Rect { min, max }
    }
    fn top_left_handle(&self, size: f32) -> Rect {
        Rect::from_center_size(self.left_top(), Vec2::splat(size))
    }

    fn top_right_handle(&self, size: f32) -> Rect {
        Rect::from_center_size(self.right_top(), Vec2::splat(size))
    }

    fn bottom_left_handle(&self, size: f32) -> Rect {
        Rect::from_center_size(self.left_bottom(), Vec2::splat(size))
    }

    fn bottom_right_handle(&self, size: f32) -> Rect {
        Rect::from_center_size(self.right_bottom(), Vec2::splat(size))
    }
}
