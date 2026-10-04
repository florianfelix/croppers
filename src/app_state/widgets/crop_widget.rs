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
    pub desired_dimension: [u32; 2],
    pub image_size: Vec2,
}

pub struct WidgetCroppers {
    id: Id,
    widget_size: Vec2,
    canvas_rect: Rect,
    image_size: Vec2,
    desired_dimension: [u32; 2],
}
impl WidgetCroppers {
    pub fn new(
        id: &str,
        canvas_rect: Rect,
        image_size: impl Into<Vec2>,
        desired_dimension: [u32; 2],
    ) -> Self {
        Self {
            id: Id::new(id),
            widget_size: (0.0, 0.0).into(),
            canvas_rect,
            image_size: image_size.into(),
            desired_dimension,
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
        // Ziel-Seitenverhältnis der gewünschten Dimension (Breite / Höhe)
        let target_aspect = self.desired_dimension[0] as f32 / self.desired_dimension[1] as f32;
        // Seitenverhältnis des echten, skalierten Bildes auf dem Bildschirm
        let image_aspect = image_rect.width() / image_rect.height();

        // Berechne die maximale Größe, die das Crop-Rechteck innerhalb des Bildes annehmen kann
        let (crop_width, crop_height) = if target_aspect > image_aspect {
            // Das Crop-Format ist breiter als das Bild -> Breite voll ausnutzen
            let w = image_rect.width();
            let h = w / target_aspect;
            (w, h)
        } else {
            // Das Crop-Format ist höher als das Bild -> Höhe voll ausnutzen
            let h = image_rect.height();
            let w = h * target_aspect;
            (w, h)
        };

        // Erstelle das Rechteck exakt zentriert im Bild-Rechteck
        let crop_rect =
            Rect::from_center_size(image_rect.center(), Vec2::new(crop_width, crop_height));

        WidgetCroppersState {
            image_rect,
            crop_rect,
            drag_target: DragTarget::None,
            desired_dimension: self.desired_dimension,
            image_size: self.image_size,
        }
    }
    pub fn get_uv_crop_rect(ui: &egui::Ui, id_str: &str) -> Option<Rect> {
        let id = Id::new(id_str);

        // Zustand aus dem egui Memory-Pool auslesen
        ui.data_mut(|d| d.get_persisted::<WidgetCroppersState>(id))
            .map(|state| {
                let image = state.image_rect;
                let crop = state.crop_rect;

                // Verhindere Division durch Null, falls das Bild noch nicht gerendert wurde
                if image.width() == 0.0 || image.height() == 0.0 {
                    return Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                }

                // Pixel-Koordinaten in den Bereich 0.0 bis 1.0 umrechnen
                let min_x = (crop.min.x - image.min.x) / image.width();
                let min_y = (crop.min.y - image.min.y) / image.height();
                let max_x = (crop.max.x - image.min.x) / image.width();
                let max_y = (crop.max.y - image.min.y) / image.height();

                // Da wir die Grenzen durch `constrain` sichern, weichen die UV-Werte nicht aus [0, 1] ab.
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

        if state.desired_dimension != self.desired_dimension {
            state.desired_dimension = self.desired_dimension;

            // Neues Seitenverhältnis berechnen
            let target_aspect = self.desired_dimension[0] as f32 / self.desired_dimension[1] as f32;

            // Das bestehende Crop-Rechteck anpassen: Breite beibehalten, Höhe basierend auf Aspekt neu berechnen
            let current_center = state.crop_rect.center();
            let new_width = state.crop_rect.width().max(state.image_rect.width() / 2.);
            let new_height = new_width / target_aspect;

            state.crop_rect =
                Rect::from_center_size(current_center, Vec2::new(new_width, new_height));
        }

        state.image_rect = self.new_image_rect(&self.canvas_rect, &self.image_size);
        constrain_crop_rect(
            state.image_rect,
            &mut state.crop_rect,
            state.desired_dimension,
        );

        let painter = ui.painter();
        let handle_radius = 60.0;

        let pointer_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));
        let delta = ui.input(|i| i.pointer.delta());
        // if ui.rect_contains_pointer(state.crop_rect.top_handle(handle_radius)) {
        //     info!("Hovered");
        //     if ui.input(|i| i.pointer.primary_down()) {
        //         info!("Dragging");
        //         state.drag_target = DragTarget::Top;
        //     }
        // }
        // if ui.input(|i| state.crop_rect. ){};

        if response.dragged() {
            if response.drag_started() {
                state.drag_target =
                    DragTarget::current_target(&state.crop_rect, handle_radius, pointer_pos);
            }
            resize_constrained(&mut state, delta);
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

        constrain_crop_rect(
            state.image_rect,
            &mut state.crop_rect,
            state.desired_dimension,
        );

        ui.data_mut(|d| d.insert_persisted(self.id, state));

        painter.rect_stroke(
            state.crop_rect,
            0.0,
            Stroke::new(1.0, Color32::GOLD),
            egui::StrokeKind::Outside,
        );

        //         painter.rect_stroke(
        //             state.crop_rect.right_handle(handle_radius),
        //             0.0,
        //             Stroke::new(2.0, Color32::DARK_GREEN),
        //             egui::StrokeKind::Middle,
        //         );
        //         painter.rect_stroke(
        //             state.crop_rect.top_handle(handle_radius),
        //             0.0,
        //             Stroke::new(2.0, Color32::DARK_GREEN),
        //             egui::StrokeKind::Middle,
        //         );
        //
        //         painter.rect_stroke(
        //             state.crop_rect.shrink(handle_radius / 2.),
        //             0.0,
        //             Stroke::new(1.0, Color32::DARK_GREEN),
        //             egui::StrokeKind::Outside,
        //         );

        if ui.is_rect_visible(state.image_rect) {
            // Die Farbe für die Abdunklung (Schwarz mit 60% Deckkraft / Alpha 150)
            let dim_color = Color32::from_black_alpha(150);
            let img = state.image_rect;
            let crop = state.crop_rect;

            // 1. Oberer Balken (erstreckt sich über die gesamte Breite des Bildes)
            if crop.min.y > img.min.y {
                let top_rect = Rect::from_min_max(img.min, Pos2::new(img.max.x, crop.min.y));
                painter.rect_filled(top_rect, 0.0, dim_color);
            }

            // 2. Unterer Balken (erstreckt sich über die gesamte Breite des Bildes)
            if crop.max.y < img.max.y {
                let bottom_rect = Rect::from_min_max(Pos2::new(img.min.x, crop.max.y), img.max);
                painter.rect_filled(bottom_rect, 0.0, dim_color);
            }

            // 3. Linker Balken (füllt den Raum links neben der Box, zwischen Top und Bottom)
            if crop.min.x > img.min.x {
                let left_rect = Rect::from_min_max(
                    Pos2::new(img.min.x, crop.min.y),
                    Pos2::new(crop.min.x, crop.max.y),
                );
                painter.rect_filled(left_rect, 0.0, dim_color);
            }

            // 4. Rechter Balken (füllt den Raum rechts neben der Box, zwischen Top und Bottom)
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

fn constrain_crop_rect(image_rect: Rect, crop_rect: &mut Rect, desired_dimension: [u32; 2]) {
    let target_aspect = desired_dimension[0] as f32 / desired_dimension[1] as f32;

    // 1. Maintain aspect ratio while clamping maximum bounds to the image container
    if crop_rect.width() > image_rect.width() || crop_rect.height() > image_rect.height() {
        let max_height_by_width = image_rect.width() / target_aspect;

        let (new_width, new_height) = if max_height_by_width <= image_rect.height() {
            // Limited by image width
            (image_rect.width(), max_height_by_width)
        } else {
            // Limited by image height
            (image_rect.height() * target_aspect, image_rect.height())
        };

        // Resize the rect from its current center to preserve user intent
        let center = crop_rect.center();
        *crop_rect = Rect::from_center_size(center, Vec2::new(new_width, new_height));
    }

    // 2. Safe linear boundary translation shifts (safe for aspect ratio)
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
    match state.drag_target {
        None => {}
        Center => state.crop_rect = state.crop_rect.translate(delta),
        Top => state.crop_rect.min.y += delta.y,
        Bottom => state.crop_rect.max.y += delta.y,
        Right => state.crop_rect.max.x += delta.x,
        Left => state.crop_rect.min.x += delta.x,
        TopLeft => {
            state.crop_rect.min.x += delta.x;
            state.crop_rect.min.y += delta.y;
        }
        TopRight => {
            state.crop_rect.max.x += delta.x;
            state.crop_rect.min.y += delta.y;
        }
        BottomLeft => {
            state.crop_rect.min.x += delta.x;
            state.crop_rect.max.y += delta.y;
        }
        BottomRight => {
            state.crop_rect.max.x += delta.x;
            state.crop_rect.max.y += delta.y;
        }
    }
}

fn resize_constrained(state: &mut WidgetCroppersState, delta: Vec2) {
    // 1. Calculate or pass your desired aspect ratio (width / height)
    let target_aspect = state.desired_dimension[0] as f32 / state.desired_dimension[1] as f32;

    use DragTarget::*;
    match state.drag_target {
        None => {}
        Center => state.crop_rect = state.crop_rect.translate(delta),

        // --- Corner Handles (Scaling matching the Aspect Ratio) ---
        TopLeft => {
            // Calculate the dominant movement to determine the scale size change
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
        // Moving an edge scales the whole box uniformly to preserve the aspect ratio
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
