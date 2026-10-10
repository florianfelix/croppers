use crate::app_state::{
    files::ImageSource, ui_state::UiState, widgets::crop_widget::WidgetCroppers,
};

impl UiState {
    pub fn draw_canvas(
        &mut self,
        ctx: &mut egui::Ui,
        source: Option<&ImageSource>,
        src_dims: [u32; 2],
    ) {
        let idname = if let Some(src) = source {
            src.name.clone()
        } else {
            "noimage".to_string()
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.set_canvas_rect(rect);

                if !self.has_source_image {
                    ui.centered_and_justified(|ui| {
                        elegance::Card::new()
                            .heading("No Image Loded")
                            .show(ui, |_ui| {});
                    });
                    return;
                }
                // Handle Pan and Zoom
                // Create an invisible, sense-less interaction just for hovering.
                // We don't use Sense::click_and_drag() because that steals the click focus
                // from the crop widget that is rendered below it.
                let _response =
                    ui.interact(rect, ui.id().with("canvas_interact"), egui::Sense::hover());

                // For panning, we can poll the pointer globally if hovered, or rely on Middle button explicitly.
                let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                let is_hovered = pointer_pos.is_some_and(|pos| rect.contains(pos));

                if is_hovered && ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
                    self.pan += ui.input(|i| i.pointer.delta());
                }

                // Scroll zoom
                if is_hovered {
                    let scroll = ui.input(|i| i.smooth_scroll_delta);
                    if scroll.y != 0.0 {
                        let zoom_factor = (scroll.y * 0.006).exp();
                        self.zoom *= zoom_factor;
                        self.zoom = self.zoom.clamp(0.1, 100.0);
                    }
                }

                if self.crop.enabled {
                    if self.crop.mode != crate::app_state::widgets::CropMode::Scaling {
                        let image_size = (src_dims[0] as f32, src_dims[1] as f32);
                        let aspect_ratio = self.crop.aspect_ratio(src_dims);
                        let mut crop_widget =
                            WidgetCroppers::new(&idname, rect, image_size, aspect_ratio);

                        // Let crop widget know about current transform
                        crop_widget = crop_widget.with_transform(self.zoom, self.pan);

                        ui.add(crop_widget);
                        if let Some(uv_rect) = WidgetCroppers::get_uv_crop_rect(ui, &idname) {
                            self.crop.uv_rect = uv_rect;
                        }
                    }
                }
            });
    }
}
