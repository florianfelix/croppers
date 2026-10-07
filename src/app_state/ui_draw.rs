use tracing::info;

use crate::app_state::{
    AppState,
    effect_stack::{EffectContainerObject, EffectSelector},
    widgets::{CropMode, crop_widget::WidgetCroppers},
};

impl AppState {
    pub fn draw(&mut self, raw_input: &egui::RawInput, ctx: &mut egui::Ui) {
        self.draw_panel(raw_input, ctx);
        self.draw_canvas(raw_input, ctx);
    }
    #[tracing::instrument(skip(self, _raw_input, ctx))]
    fn draw_canvas(&mut self, _raw_input: &egui::RawInput, ctx: &mut egui::Ui) {
        let idname = if let Some(source) = self.source.as_ref() {
            source.name.clone()
        } else {
            "noimage".to_string()
        };
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.ui_state
                    .set_canvas_rect(ui.available_rect_before_wrap());

                if self.ui_state.crop.enabled {
                    let img_size = self.texture_vault.srcimg_dims;
                    let image_size = (img_size[0] as f32, img_size[1] as f32);
                    let aspect_ratio = self.ui_state.crop.aspect_ratio(img_size);
                    let test = WidgetCroppers::new(
                        &idname,
                        ui.available_rect_before_wrap(),
                        image_size,
                        aspect_ratio,
                    );

                    ui.add(test);
                    if let Some(uv_rect) = WidgetCroppers::get_uv_crop_rect(ui, &idname) {
                        self.ui_state.crop.uv_rect = uv_rect;
                    }
                }
            });
    }
    #[tracing::instrument(skip(self, _raw_input, ctx))]
    fn draw_panel(&mut self, _raw_input: &egui::RawInput, ctx: &mut egui::Ui) {
        // let mut request_redraw = false;
        if !self.ui_state.show_panel {
            return;
        }

        egui::Panel::left("settings_panel")
            .resizable(false)
            .default_size(280.0)
            .show(ctx, |ui| {
                ui.heading("Image Controls");
                ui.separator();
                if let Some(ref source) = self.source {
                    ui.label(source.name.to_string());
                    ui.label(format!("{}x{}", source.image_size.0, source.image_size.1));
                }
                ui.separator();
                ui.indent("Crop", |ui| {
                    ui.checkbox(&mut self.ui_state.crop.enabled, "Crop");

                    egui::ComboBox::from_label("Mode")
                        .selected_text(self.ui_state.crop.mode.label())
                        .show_ui(ui, |ui| {
                            for mode in CropMode::ALL {
                                ui.selectable_value(
                                    &mut self.ui_state.crop.mode,
                                    mode,
                                    mode.label(),
                                );
                            }
                        });

                    match self.ui_state.crop.mode {
                        CropMode::FixedDimensions => {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut self.ui_state.crop.crop_width)
                                        .range(1..=10000)
                                        .speed(10),
                                );
                                ui.label("target width");
                            });
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut self.ui_state.crop.crop_height)
                                        .range(1..=10000)
                                        .speed(10),
                                );
                                ui.label("target height");
                            });
                        }
                        CropMode::CustomRatio => {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut self.ui_state.crop.custom_ratio[0])
                                        .range(1..=100)
                                        .speed(1),
                                );
                                ui.label(":");
                                ui.add(
                                    egui::DragValue::new(&mut self.ui_state.crop.custom_ratio[1])
                                        .range(1..=100)
                                        .speed(1),
                                );
                                ui.label("ratio (W:H)");
                            });
                        }
                        _ => {}
                    }

                    let src_dims = self.texture_vault.srcimg_dims;
                    if src_dims[0] > 0 && src_dims[1] > 0 {
                        let uv = self.ui_state.crop.uv_rect;
                        let crop_px_w = ((src_dims[0] as f32) * uv.width()).round().max(1.0) as u32;
                        let crop_px_h =
                            ((src_dims[1] as f32) * uv.height()).round().max(1.0) as u32;

                        if self.ui_state.crop.mode == CropMode::FixedDimensions {
                            ui.label(format!("Crop area: {} × {} px", crop_px_w, crop_px_h));
                            ui.label(format!(
                                "Output: {} × {} px (scaled)",
                                self.ui_state.crop.crop_width, self.ui_state.crop.crop_height
                            ));
                        } else {
                            ui.label(format!("Output: {} × {} px", crop_px_w, crop_px_h));
                        }
                    }

                    if ui.button("Apply Crop (Enter)").clicked() {
                        self.send_crop_job();
                    }
                });

                ui.separator();
                let mut request_redraw = false;
                if ui
                    .checkbox(&mut self.effect_stack.enabled, "Enabled")
                    .changed()
                {
                    // container.set_enabled(is_enabled);
                    self.effect_stack.needs_rebuild = true;
                    self.needs_compute = true;
                    request_redraw = true;
                }

                let _ = egui::ComboBox::from_label("Add Effect")
                    .selected_text("Add Effect")
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(false, "Vignette").clicked() {
                            info!("Vignette selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Vignette.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                        }
                        if ui.selectable_label(false, "Blur").clicked() {
                            info!("Blur selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Blur.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Black and White").clicked() {
                            info!("Black and White selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Bw.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "HSV").clicked() {
                            info!("HSV selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Hsv.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Gamma").clicked() {
                            info!("Gamma selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Gamma.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "TiltShift").clicked() {
                            info!("TiltShift selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::TiltShift.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Pixelate Tilt Shift").clicked() {
                            info!("PixelateShift selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::PixelateTilt.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Pixelate Linear").clicked() {
                            info!("PixelateShift selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::PixelateLinear.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Dither").clicked() {
                            info!("Dither selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Dither.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Crt").clicked() {
                            info!("Crt selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Crt.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Glitch").clicked() {
                            info!("Glitch selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Glitch.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Vcr").clicked() {
                            info!("Vcr selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Vcr.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                        if ui.selectable_label(false, "Ascii").clicked() {
                            info!("Ascii selected");
                            let mut new_effect: Box<dyn EffectContainerObject> =
                                EffectSelector::Ascii.into();
                            new_effect.set_id(self.effect_stack.effects.len());
                            self.effect_stack.effects.push(new_effect);
                            self.effect_stack.needs_rebuild = true;
                            request_redraw = true;
                        }
                    });

                let mut swap_index = None;
                let mut remove_idx = None;

                let len = self.effect_stack.effects.len();

                for (idx, container) in &mut self.effect_stack.effects.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let mut is_enabled = container.enabled();
                        if ui.checkbox(&mut is_enabled, container.name()).changed() {
                            container.set_enabled(is_enabled);
                            self.effect_stack.needs_rebuild = true;
                            self.needs_compute = true;
                            request_redraw = true;
                        }
                        if ui.button("X").clicked() {
                            remove_idx = Some(idx);
                            self.effect_stack.needs_rebuild = true;
                            self.needs_compute = true;
                            request_redraw = true;
                        }
                    });

                    if idx > 0 && ui.small_button("▲").clicked() {
                        swap_index = Some((idx, idx - 1));
                    }
                    if idx + 1 < len && ui.small_button("▼").clicked() {
                        swap_index = Some((idx, idx + 1));
                    }

                    if container.enabled() {
                        if container.draw_ui(ui) {
                            self.needs_compute = true;
                        }
                    }
                    ui.separator();
                }

                if request_redraw {
                    self.request_redraw_wake();
                }
                if let Some(idx) = remove_idx {
                    self.effect_stack.effects.remove(idx);
                    self.effect_stack.needs_rebuild = true;
                    self.needs_compute = true;
                    self.effect_stack.validate_ids();
                    self.request_redraw_wake();
                }
                // Perform stack swap and flag dirty state
                if let Some((i, j)) = swap_index {
                    self.effect_stack.effects.swap(i, j);
                    self.effect_stack.needs_rebuild = true;
                    self.needs_compute = true;
                    self.effect_stack.validate_ids();
                    self.request_redraw_wake();
                }
            });
    }
}
