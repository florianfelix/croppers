#![allow(unused)]
use egui::{Color32, Id, Pos2, Rect, Sense, Stroke, Vec2, Widget, util::IdTypeMap};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq)]
enum DragTarget {
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
}
impl DragTarget {
    fn current_target(crop_rect: &Rect, handle_radius: f32, pointer_pos: Pos2) -> Self {
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
            DragTarget::None
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct WidgetCroppersState {
    pub image_rect: Rect,
    pub crop_rect: Rect,
    pub drag_target: DragTarget,
    pub aspect_ratio: Option<f32>,
    pub image_size: Vec2,
}

pub struct WidgetCroppers {
    id: Id,
    widget_size: Vec2,
    canvas_rect: Rect,
    image_size: Vec2,
    aspect_ratio: Option<f32>,
}
impl WidgetCroppers {
    pub fn new(
        id: &str,
        canvas_rect: Rect,
        image_size: impl Into<Vec2>,
        aspect_ratio: Option<f32>,
    ) -> Self {
        Self {
            id: Id::new(id),
            widget_size: (0.0, 0.0).into(),
            canvas_rect,
            image_size: image_size.into(),
            aspect_ratio,
        }
    }

    fn new_image_rect(&self, current_canvas: &Rect, image_size: &Vec2) -> Rect {
        let canvas_aspect = current_canvas.width() / current_canvas.height();
        let image_aspect = image_size.x / image_size.y;

        let (scale_x, scale_y) = if image_aspect > canvas_aspect {
            (1.0, canvas_aspect / image_aspect)
        } else {
            (image_aspect / canvas_aspect, 1.0)
        };

        let scaled_x = current_canvas.width() * scale_x;
        let scaled_y = current_canvas.height() * scale_y;

        let diff_x: f32 = current_canvas.width() - scaled_x;
        let diff_y: f32 = current_canvas.height() - scaled_y;

        let x = current_canvas.min.x + diff_x / 2.;
        let y = current_canvas.min.y + diff_y / 2.;

        Rect {
            min: Pos2 { x, y },
            max: Pos2 {
                x: x + scaled_x,
                y: y + scaled_y,
            },
        }
    }

    fn fresh_state(&self) -> WidgetCroppersState {
        let image_rect = self.new_image_rect(&self.canvas_rect, &self.image_size);

        let crop_rect = if let Some(target_aspect) = self.aspect_ratio {
            let image_aspect = image_rect.width() / image_rect.height();

            let (crop_width, crop_height) = if target_aspect > image_aspect {
                let w = image_rect.width();
                let h = w / target_aspect;
                (w, h)
            } else {
                let h = image_rect.height();
                let w = h * target_aspect;
                (w, h)
            };

            Rect::from_center_size(image_rect.center(), Vec2::new(crop_width, crop_height))
        } else {
            image_rect
        };

        WidgetCroppersState {
            image_rect,
            crop_rect,
            drag_target: DragTarget::None,
            aspect_ratio: self.aspect_ratio,
            image_size: self.image_size,
        }
    }

    pub fn get_uv_crop_rect(ui: &egui::Ui, id_str: &str) -> Option<Rect> {
        let id = Id::new(id_str);

        ui.data_mut(|d| d.get_persisted::<WidgetCroppersState>(id))
            .map(|state| {
                let image = state.image_rect;
                let crop = state.crop_rect;

                if image.width() == 0.0 || image.height() == 0.0 {
                    return Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                }

                let min_x = (crop.min.x - image.min.x) / image.width();
                let min_y = (crop.min.y - image.min.y) / image.height();
                let max_x = (crop.max.x - image.min.x) / image.width();
                let max_y = (crop.max.y - image.min.y) / image.height();

                Rect::from_min_max(
                    Pos2::new(min_x.clamp(0.0, 1.0), min_y.clamp(0.0, 1.0)),
                    Pos2::new(max_x.clamp(0.0, 1.0), max_y.clamp(0.0, 1.0)),
                )
            })
    }
}

impl Widget for WidgetCroppers {
    #[tracing::instrument(name = "WidgetCroppers", skip(self, ui))]
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(self.canvas_rect.size(), Sense::click_and_drag());
        let mut state = ui.data_mut(|d| {
            d.get_persisted::<WidgetCroppersState>(self.id)
                .unwrap_or_else(|| self.fresh_state())
        });

        let aspect_changed = match (state.aspect_ratio, self.aspect_ratio) {
            (Some(a), Some(b)) => (a - b).abs() > 0.0001,
            (None, None) => false,
            _ => true,
        };

        if aspect_changed {
            state.aspect_ratio = self.aspect_ratio;
            if let Some(target_aspect) = self.aspect_ratio {
                let current_center = state.crop_rect.center();
                let mut new_width = state.crop_rect.width().max(20.0);
                let mut new_height = new_width / target_aspect;
                if new_height > state.image_rect.height() {
                    new_height = state.image_rect.height();
                    new_width = new_height * target_aspect;
                }
                if new_width > state.image_rect.width() {
                    new_width = state.image_rect.width();
                    new_height = new_width / target_aspect;
                }
                state.crop_rect =
                    Rect::from_center_size(current_center, Vec2::new(new_width, new_height));
            }
        }

        state.image_rect = self.new_image_rect(&self.canvas_rect, &self.image_size);
        constrain_crop_rect(state.image_rect, &mut state.crop_rect, state.aspect_ratio);

        let painter = ui.painter();
        let handle_radius = 60.0;

        let pointer_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));
        let delta = ui.input(|i| i.pointer.delta());

        if response.dragged() {
            if response.drag_started() {
                state.drag_target =
                    DragTarget::current_target(&state.crop_rect, handle_radius, pointer_pos);
            }
            if let Some(target_aspect) = state.aspect_ratio {
                resize_constrained(&mut state, delta, target_aspect);
            } else {
                resize_freeform(&mut state, delta);
            }
        }

        if response.double_clicked() {
            let pointer_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));

            // Reset only if the double click happens inside the crop box area
            if state
                .crop_rect
                .shrink(handle_radius / 2.)
                .contains(pointer_pos)
            {
                info!("Resetting Crop State");
                let fresh = self.fresh_state();
                state.crop_rect = fresh.crop_rect;
                state.drag_target = DragTarget::None;
            }
        }

        constrain_crop_rect(state.image_rect, &mut state.crop_rect, state.aspect_ratio);

        ui.data_mut(|d| d.insert_persisted(self.id, state));

        painter.rect_stroke(
            state.crop_rect,
            0.0,
            Stroke::new(1.0, Color32::GOLD),
            egui::StrokeKind::Outside,
        );

        if ui.is_rect_visible(state.image_rect) {
            let dim_color = Color32::from_black_alpha(150);
            let img = state.image_rect;
            let crop = state.crop_rect;

            // 1. Top bar
            if crop.min.y > img.min.y {
                let top_rect = Rect::from_min_max(img.min, Pos2::new(img.max.x, crop.min.y));
                painter.rect_filled(top_rect, 0.0, dim_color);
            }

            // 2. Bottom bar
            if crop.max.y < img.max.y {
                let bottom_rect = Rect::from_min_max(Pos2::new(img.min.x, crop.max.y), img.max);
                painter.rect_filled(bottom_rect, 0.0, dim_color);
            }

            // 3. Left bar
            if crop.min.x > img.min.x {
                let left_rect = Rect::from_min_max(
                    Pos2::new(img.min.x, crop.min.y),
                    Pos2::new(crop.min.x, crop.max.y),
                );
                painter.rect_filled(left_rect, 0.0, dim_color);
            }

            // 4. Right bar
            if crop.max.x < img.max.x {
                let right_rect = Rect::from_min_max(
                    Pos2::new(crop.max.x, crop.min.y),
                    Pos2::new(img.max.x, crop.max.y),
                );
                painter.rect_filled(right_rect, 0.0, dim_color);
            }
        }

        response
    }
}

fn constrain_crop_rect(image_rect: Rect, crop_rect: &mut Rect, aspect_ratio: Option<f32>) {
    if let Some(target_aspect) = aspect_ratio {
        // 1. Maintain aspect ratio while clamping maximum bounds to the image container
        if crop_rect.width() > image_rect.width() || crop_rect.height() > image_rect.height() {
            let max_height_by_width = image_rect.width() / target_aspect;

            let (new_width, new_height) = if max_height_by_width <= image_rect.height() {
                (image_rect.width(), max_height_by_width)
            } else {
                (image_rect.height() * target_aspect, image_rect.height())
            };

            let center = crop_rect.center();
            *crop_rect = Rect::from_center_size(center, Vec2::new(new_width, new_height));
        }
    } else {
        if crop_rect.width() > image_rect.width() {
            let center_x = crop_rect.center().x;
            crop_rect.min.x = center_x - image_rect.width() / 2.0;
            crop_rect.max.x = center_x + image_rect.width() / 2.0;
        }
        if crop_rect.height() > image_rect.height() {
            let center_y = crop_rect.center().y;
            crop_rect.min.y = center_y - image_rect.height() / 2.0;
            crop_rect.max.y = center_y + image_rect.height() / 2.0;
        }
    }

    // 2. Safe linear boundary translation shifts
    let mut translation = Vec2::new(0.0, 0.0);

    if crop_rect.min.x < image_rect.min.x {
        translation.x += image_rect.min.x - crop_rect.min.x;
    } else if crop_rect.max.x > image_rect.max.x {
        translation.x -= crop_rect.max.x - image_rect.max.x;
    }

    if crop_rect.min.y < image_rect.min.y {
        translation.y += image_rect.min.y - crop_rect.min.y;
    } else if crop_rect.max.y > image_rect.max.y {
        translation.y -= crop_rect.max.y - image_rect.max.y;
    }

    *crop_rect = crop_rect.translate(translation);

    // Hard clamp boundaries in case dimensions exactly equal the image container
    crop_rect.min.x = crop_rect.min.x.max(image_rect.min.x);
    crop_rect.min.y = crop_rect.min.y.max(image_rect.min.y);
    crop_rect.max.x = crop_rect.max.x.min(image_rect.max.x);
    crop_rect.max.y = crop_rect.max.y.min(image_rect.max.y);
}

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

fn resize_freeform(state: &mut WidgetCroppersState, delta: Vec2) {
    use DragTarget::*;
    let min_size = 20.0;
    let img = state.image_rect;
    match state.drag_target {
        None => {}
        Center => {
            state.crop_rect = state.crop_rect.translate(delta);
            let mut translation = Vec2::ZERO;
            if state.crop_rect.min.x < img.min.x {
                translation.x += img.min.x - state.crop_rect.min.x;
            } else if state.crop_rect.max.x > img.max.x {
                translation.x -= state.crop_rect.max.x - img.max.x;
            }
            if state.crop_rect.min.y < img.min.y {
                translation.y += img.min.y - state.crop_rect.min.y;
            } else if state.crop_rect.max.y > img.max.y {
                translation.y -= state.crop_rect.max.y - img.max.y;
            }
            state.crop_rect = state.crop_rect.translate(translation);
        }
        Top => {
            state.crop_rect.min.y = (state.crop_rect.min.y + delta.y)
                .max(img.min.y)
                .min(state.crop_rect.max.y - min_size);
        }
        Bottom => {
            state.crop_rect.max.y = (state.crop_rect.max.y + delta.y)
                .min(img.max.y)
                .max(state.crop_rect.min.y + min_size);
        }
        Right => {
            state.crop_rect.max.x = (state.crop_rect.max.x + delta.x)
                .min(img.max.x)
                .max(state.crop_rect.min.x + min_size);
        }
        Left => {
            state.crop_rect.min.x = (state.crop_rect.min.x + delta.x)
                .max(img.min.x)
                .min(state.crop_rect.max.x - min_size);
        }
        TopLeft => {
            state.crop_rect.min.x = (state.crop_rect.min.x + delta.x)
                .max(img.min.x)
                .min(state.crop_rect.max.x - min_size);
            state.crop_rect.min.y = (state.crop_rect.min.y + delta.y)
                .max(img.min.y)
                .min(state.crop_rect.max.y - min_size);
        }
        TopRight => {
            state.crop_rect.max.x = (state.crop_rect.max.x + delta.x)
                .min(img.max.x)
                .max(state.crop_rect.min.x + min_size);
            state.crop_rect.min.y = (state.crop_rect.min.y + delta.y)
                .max(img.min.y)
                .min(state.crop_rect.max.y - min_size);
        }
        BottomLeft => {
            state.crop_rect.min.x = (state.crop_rect.min.x + delta.x)
                .max(img.min.x)
                .min(state.crop_rect.max.x - min_size);
            state.crop_rect.max.y = (state.crop_rect.max.y + delta.y)
                .min(img.max.y)
                .max(state.crop_rect.min.y + min_size);
        }
        BottomRight => {
            state.crop_rect.max.x = (state.crop_rect.max.x + delta.x)
                .min(img.max.x)
                .max(state.crop_rect.min.x + min_size);
            state.crop_rect.max.y = (state.crop_rect.max.y + delta.y)
                .min(img.max.y)
                .max(state.crop_rect.min.y + min_size);
        }
    }
}

fn resize_constrained(state: &mut WidgetCroppersState, delta: Vec2, target_aspect: f32) {
    use DragTarget::*;
    match state.drag_target {
        None => {}
        Center => state.crop_rect = state.crop_rect.translate(delta),

        // --- Corner Handles (Scaling matching the Aspect Ratio) ---
        TopLeft => {
            let change = (delta.x + delta.y) * 0.5;
            let new_width = (state.crop_rect.width() - change).max(20.0);
            let new_height = new_width / target_aspect;

            state.crop_rect.min = state.crop_rect.max - Vec2::new(new_width, new_height);
        }
        TopRight => {
            let change = (delta.x - delta.y) * 0.5;
            let new_width = (state.crop_rect.width() + change).max(20.0);
            let new_height = new_width / target_aspect;

            state.crop_rect.max.x = state.crop_rect.min.x + new_width;
            state.crop_rect.min.y = state.crop_rect.max.y - new_height;
        }
        BottomLeft => {
            let change = (-delta.x + delta.y) * 0.5;
            let new_width = (state.crop_rect.width() + change).max(20.0);
            let new_height = new_width / target_aspect;

            state.crop_rect.min.x = state.crop_rect.max.x - new_width;
            state.crop_rect.max.y = state.crop_rect.min.y + new_height;
        }
        BottomRight => {
            let change = (delta.x + delta.y) * 0.5;
            let new_width = (state.crop_rect.width() + change).max(20.0);
            let new_height = new_width / target_aspect;

            state.crop_rect.max = state.crop_rect.min + Vec2::new(new_width, new_height);
        }

        // --- Linear Edge Handles (Uniform Scale from Center) ---
        Top | Bottom => {
            let change = if state.drag_target == Top {
                -delta.y
            } else {
                delta.y
            };
            let new_height = (state.crop_rect.height() + change).max(20.0);
            let new_width = new_height * target_aspect;

            let center = state.crop_rect.center();
            state.crop_rect = Rect::from_center_size(center, Vec2::new(new_width, new_height));
        }
        Left | Right => {
            let change = if state.drag_target == Left {
                -delta.x
            } else {
                delta.x
            };
            let new_width = (state.crop_rect.width() + change).max(20.0);
            let new_height = new_width / target_aspect;

            let center = state.crop_rect.center();
            state.crop_rect = Rect::from_center_size(center, Vec2::new(new_width, new_height));
        }
    }
}
