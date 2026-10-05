use tracing::info;

use crate::app_state::{
    AppState, effect_stack::EffectSelector, widgets::crop_widget::WidgetCroppers,
};

impl AppState {
    // #[tracing::instrument(skip(self, raw_input, ctx))]
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
                    // if let Some(img_size) = self.texture_vault.image_size {
                    let img_size = self.texture_vault.srcimg_dims;
                    let desired = [
                        self.ui_state.crop.crop_width,
                        self.ui_state.crop.crop_height,
                    ];
                    let image_size = (img_size[0] as f32, img_size[1] as f32);
                    let test = WidgetCroppers::new(
                        &idname,
                        ui.available_rect_before_wrap(),
                        image_size,
                        desired,
                    );

                    ui.add(test);
                    if let Some(uv_rect) = WidgetCroppers::get_uv_crop_rect(ui, &idname) {
                        self.ui_state.crop.uv_rect = uv_rect;
                    }
                    // }
                }
            });
    }
    #[tracing::instrument(skip(self, _raw_input, ctx))]
    fn draw_panel(&mut self, _raw_input: &egui::RawInput, ctx: &mut egui::Ui) {
        let mut request_redraw = false;
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
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.ui_state.crop.crop_width)
                                .range(1..=6000)
                                .speed(10),
                        );
                        ui.label("desired width");
                    });
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.ui_state.crop.crop_height)
                                .range(1..=6000)
                                .speed(10),
                        );
                        ui.label("desired height");
                    });
                });

                ui.separator();

                // let res = egui::ComboBox::from_label("Add Effect")
                //     .selected_text("Add Effect")
                //     .show_ui(ui, |ui| {
                //         if ui.selectable_label(false, "Vignette").clicked() {
                //             info!("Vignette selected");
                //             let mut new_effect: EffectContainer = EffectSelector::Vignette.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "Blur").clicked() {
                //             info!("Blur selected");
                //             let mut new_effect: EffectContainer = EffectSelector::Blur.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "Black and White").clicked() {
                //             info!("Black and White selected");
                //             let mut new_effect: EffectContainer = EffectSelector::Bw.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "HSV").clicked() {
                //             info!("HSV selected");
                //             let mut new_effect: EffectContainer = EffectSelector::Hsv.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "Gamma").clicked() {
                //             info!("Gamma selected");
                //             let mut new_effect: EffectContainer = EffectSelector::Gamma.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "TiltShift").clicked() {
                //             info!("TiltShift selected");
                //             let mut new_effect: EffectContainer = EffectSelector::TiltShift.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "Pixelate Tilt Shift").clicked() {
                //             info!("PixelateShift selected");
                //             let mut new_effect: EffectContainer =
                //                 EffectSelector::PixelateTilt.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "Pixelate Linear").clicked() {
                //             info!("PixelateShift selected");
                //             let mut new_effect: EffectContainer =
                //                 EffectSelector::PixelateLinear.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "Dither").clicked() {
                //             info!("Dither selected");
                //             let mut new_effect: EffectContainer = EffectSelector::Dither.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //         if ui.selectable_label(false, "Crt").clicked() {
                //             info!("Crt selected");
                //             let mut new_effect: EffectContainer = EffectSelector::Crt.into();
                //             new_effect.id = self.effect_stack.effects.len();
                //             self.effect_stack.effects.push(new_effect);
                //             self.effect_stack.needs_rebuild = true;
                //         }
                //     });

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
                        }
                        if ui.button("X").clicked() {
                            remove_idx = Some(idx);
                            self.effect_stack.needs_rebuild = true;
                            self.needs_compute = true;
                        }
                    });

                    if idx > 0 && ui.small_button("▲").clicked() {
                        swap_index = Some((idx, idx - 1));
                    }
                    if idx + 1 < len && ui.small_button("▼").clicked() {
                        swap_index = Some((idx, idx + 1));
                    }

                    if container.enabled() {
                        // Jedes Trait-Objekt zeichnet sich vollkommen autonom selbst!
                        if container.draw_ui(ui) {
                            self.needs_compute = true;
                        }
                    }
                    ui.separator();
                }
                //                 for (idx, container) in &mut self.effect_stack.effects.iter_mut().enumerate() {
                //                     ui.horizontal(|ui| {
                //                         if ui
                //                             .checkbox(&mut container.enabled, &container.name)
                //                             .changed()
                //                         {
                //                             self.effect_stack.needs_rebuild = true;
                //                             self.needs_compute = true;
                //                         }
                //                         if ui.button("X").clicked() {
                //                             remove_idx = Some(idx);
                //                             self.effect_stack.needs_rebuild = true;
                //                             self.needs_compute = true;
                //                         }
                //                     });
                //
                //                     // --- REORDERING BUTTONS ---
                //                     if idx > 0 && ui.small_button("▲").clicked() {
                //                         swap_index = Some((idx, idx - 1));
                //                     }
                //                     if idx + 1 < len && ui.small_button("▼").clicked() {
                //                         swap_index = Some((idx, idx + 1));
                //                     }
                //
                //                     if container.enabled {
                //                         match &mut container.effect {
                //                             Effect::Crt(settings) => {
                //                                 ui.indent("dither_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.curvature, 0.0..=1.0)
                //                                                 .text("curvature"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(
                //                                                 &mut settings.scanline_count,
                //                                                 0.0..=1000.0,
                //                                             )
                //                                             .text("scanlines"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(
                //                                                 &mut settings.scanline_strength,
                //                                                 0.0..=1.0,
                //                                             )
                //                                             .text("scanline strength"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(
                //                                                 &mut settings.rgb_mask_strength,
                //                                                 0.0..=1.0,
                //                                             )
                //                                             .text("mask strength"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                 });
                //                             }
                //                             Effect::Dither(settings) => {
                //                                 ui.indent("dither_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(
                //                                                 &mut settings.color_steps,
                //                                                 0.0..=10.0,
                //                                             )
                //                                             .text("color steps"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.spread, 0.0..=10.0)
                //                                                 .text("spread"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.scale, 0.0..=10.0)
                //                                                 .text("scale"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.palette_index, 0..=8)
                //                                                 .text("palette"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.matrix_type, 0..=1)
                //                                                 .text("matrix_type"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                 });
                //                             }
                //                             Effect::PixelateLinear(settings) => {
                //                                 ui.indent("pixelate_linear_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(
                //                                                 &mut settings.max_pixel_size,
                //                                                 0.0..=128.0,
                //                                             )
                //                                             .text("pixel size"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.angle, 0.0..=10.0)
                //                                                 .text("angle"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.start_x, 0.0..=1.0)
                //                                                 .text("start x"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.start_y, 0.0..=1.0)
                //                                                 .text("start y"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.length, 0.0..=1.0)
                //                                                 .text("length"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.invert, 0..=1)
                //                                                 .text("invert"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                 });
                //                             }
                //                             Effect::PixelateTilt(settings) => {
                //                                 ui.indent("pixelate_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(
                //                                                 &mut settings.max_pixel_size,
                //                                                 0.0..=128.0,
                //                                             )
                //                                             .text("pixel size"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.angle, 0.0..=10.0)
                //                                                 .text("angle"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.center_x, 0.0..=1.0)
                //                                                 .text("center x"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.center_y, 0.0..=1.0)
                //                                                 .text("center y"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.focus_size, 0.0..=1.0)
                //                                                 .text("focus size"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.feather, 0.0..=1.0)
                //                                                 .text("feather"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                 });
                //                             }
                //                             Effect::TiltShift(settings) => {
                //                                 ui.indent("tiltshift_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(
                //                                                 &mut settings.max_radius,
                //                                                 0.0..=200.0,
                //                                             )
                //                                             .text("Radius"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     };
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.center, 0.0..=1.0)
                //                                                 .text("Center"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     };
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.focus_size, 0.0..=1.0)
                //                                                 .text("focus size"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     };
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.feather, 0.0..=1.0)
                //                                                 .text("feather"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     };
                //                                 });
                //                             }
                //                             Effect::Gamma(settings) => {
                //                                 ui.indent("gamma_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.gr, 0.0..=10.0)
                //                                                 .text("gamma red"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.gg, 0.0..=10.0)
                //                                                 .text("gamma green"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.gb, 0.0..=10.0)
                //                                                 .text("gamma blue"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                 });
                //                             }
                //                             Effect::Hsv(settings) => {
                //                                 ui.indent("hsv_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.hue, 0.0..=1.0)
                //                                                 .text("hue"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.saturation, 0.0..=2.0)
                //                                                 .text("saturation"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.value, 0.0..=2.0)
                //                                                 .text("value"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.factor, 0.0..=1.0)
                //                                                 .text("factor"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                 });
                //                             }
                //                             Effect::Vignette(settings) => {
                //                                 ui.indent("vignette_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.intensity, 0.0..=1.0)
                //                                                 .text("Intensity"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.radius, 0.0..=2.0)
                //                                                 .text("Radius"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.smoothness, 0.0..=1.0)
                //                                                 .text("Smoothness"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     }
                //                                 });
                //                             }
                //                             Effect::Blur(settings) => {
                //                                 ui.indent("blur_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.radius, 0.0..=200.0)
                //                                                 .text("Radius"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     };
                //                                 });
                //                             }
                //                             Effect::Bw(settings) => {
                //                                 ui.indent("bw_settings", |ui| {
                //                                     if ui
                //                                         .add(
                //                                             egui::Slider::new(&mut settings.select, 0..=3)
                //                                                 .text("mode"),
                //                                         )
                //                                         .changed()
                //                                     {
                //                                         self.needs_compute = true;
                //                                     };
                //                                 });
                //                             }
                //                         }
                //                     }
                //                     ui.separator();
                //                 }
                //                 if self.effect_stack.needs_rebuild {
                //                     self.request_redraw_wake();
                //                 }
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
