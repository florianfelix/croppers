use crate::app_state::{
    files::{
        ImageSource,
        image_source::{ImgError, ImgResult},
    },
    ui_state::UiState,
    widgets::crop_widget::WidgetCroppers,
};

impl UiState {
    pub fn draw_canvas(
        &mut self,
        ctx: &mut egui::Ui,
        source: &ImgResult<ImageSource>,
        src_dims: [u32; 2],
    ) {
        let idname = if let Ok(src) = source {
            src.name.clone()
        } else {
            "noimage".to_string()
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.set_canvas_rect(rect);

                if !source.is_ok() {
                    Self::draw_canvas_status(ui, rect, source);
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

    fn draw_canvas_status(ui: &mut egui::Ui, rect: egui::Rect, source: &ImgResult<ImageSource>) {
        let theme = elegance::Theme::current(ui.ctx());

        egui::Area::new(egui::Id::new("canvas_status_area"))
            .fixed_pos(rect.min)
            .show(ui.ctx(), |ui| {
                ui.painter().rect_filled(rect, 0.0, theme.palette.bg);

                egui::Frame::new().fill(theme.palette.bg).show(ui, |ui| {
                    ui.allocate_ui_with_layout(
                        rect.size(),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let pad_top = (rect.height() * 0.4).max(20.0);
                            ui.add_space(pad_top);
                            ui.set_max_width((rect.width() - 40.0).max(100.0));

                            match source {
                                Err(ImgError::NoImage) => {
                                    ui.add(egui::Label::new(
                                        egui::RichText::new("No Image Loaded")
                                            .color(theme.palette.text_muted)
                                            .size(theme.typography.heading)
                                            .strong(),
                                    ));
                                }
                                Err(err) => {
                                    ui.add(egui::Label::new(
                                        egui::RichText::new("Failed to Load Image")
                                            .color(theme.palette.text_muted)
                                            .size(theme.typography.heading)
                                            .strong(),
                                    ));
                                    ui.add_space(8.0);
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(err.to_string())
                                                .color(egui::Color32::from_rgb(255, 100, 100)),
                                        )
                                        .wrap_mode(egui::TextWrapMode::Wrap),
                                    );
                                }
                                Ok(_) => {}
                            }
                        },
                    );
                });
            });
    }
}
