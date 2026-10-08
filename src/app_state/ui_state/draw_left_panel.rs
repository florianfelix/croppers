use crate::app_state::{
    effect_stack::{EffectStack, effects::EffectSelector},
    files::ImageSource,
    ui_state::{UiActions, UiState},
    widgets::CropMode,
};

impl UiState {
    pub fn draw_panel(
        &mut self,
        ctx: &mut egui::Ui,
        source: Option<&ImageSource>,
        src_dims: [u32; 2],
        effect_stack: &mut EffectStack,
    ) -> UiActions {
        let mut actions = UiActions::default();

        if !self.show_panel {
            return actions;
        }

        egui::Panel::left("settings_panel")
            .resizable(false)
            .default_size(280.0)
            .show(ctx, |ui| {
                ui.heading("Image Controls");

                self.draw_file_info_and_save(ui, source, &mut actions);
                ui.separator();

                self.draw_crop_section(ui, src_dims, &mut actions);
                ui.separator();

                self.draw_effects_section(ui, effect_stack, &mut actions);
            });

        actions
    }

    fn draw_file_info_and_save(
        &mut self,
        ui: &mut egui::Ui,
        source: Option<&ImageSource>,
        actions: &mut UiActions,
    ) {
        if let Some(src) = source {
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.strong(&src.name);
                ui.label(format!(
                    "Resolution: {} × {}",
                    src.image_size.0, src.image_size.1
                ));
                
                ui.add_space(4.0);
                if ui.button("Open File").on_hover_text("Ctrl + O").clicked() {
                    actions.open_file_dialog = true;
                }

                ui.add_space(4.0);
                ui.add_enabled_ui(!self.crop.enabled, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button("Save")
                            .on_hover_text("Save / Overwrite Original")
                            .clicked()
                        {
                            actions.save_overwrite = true;
                        }
                        if ui
                            .button("Save Increment")
                            .on_hover_text("Save as new file (e.g. filename_001)")
                            .clicked()
                        {
                            actions.save_increment = true;
                        }
                    });
                });
            });
        }
    }

    fn draw_crop_section(
        &mut self,
        ui: &mut egui::Ui,
        src_dims: [u32; 2],
        actions: &mut UiActions,
    ) {
        ui.indent("Crop", |ui| {
            ui.checkbox(&mut self.crop.enabled, "Crop");

            if !self.crop.enabled {
                return;
            }

            egui::ComboBox::from_label("Mode")
                .selected_text(self.crop.mode.label())
                .show_ui(ui, |ui| {
                    for mode in CropMode::ALL {
                        ui.selectable_value(&mut self.crop.mode, mode, mode.label());
                    }
                });

            match self.crop.mode {
                CropMode::FixedDimensions => {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.crop.crop_width)
                                .range(1..=10000)
                                .speed(10),
                        );
                        ui.label("target width");
                    });
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.crop.crop_height)
                                .range(1..=10000)
                                .speed(10),
                        );
                        ui.label("target height");
                    });
                }
                CropMode::CustomRatio => {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.crop.custom_ratio[0])
                                .range(1..=100)
                                .speed(1),
                        );
                        ui.label(":");
                        ui.add(
                            egui::DragValue::new(&mut self.crop.custom_ratio[1])
                                .range(1..=100)
                                .speed(1),
                        );
                        ui.label("ratio (W:H)");
                    });
                }
                _ => {}
            }

            if src_dims[0] > 0 && src_dims[1] > 0 {
                let uv = self.crop.uv_rect;
                let crop_px_w = ((src_dims[0] as f32) * uv.width()).round().max(1.0) as u32;
                let crop_px_h = ((src_dims[1] as f32) * uv.height()).round().max(1.0) as u32;

                if self.crop.mode == CropMode::FixedDimensions {
                    ui.label(format!("Crop area: {} × {} px", crop_px_w, crop_px_h));
                    ui.label(format!(
                        "Output: {} × {} px (scaled)",
                        self.crop.crop_width, self.crop.crop_height
                    ));
                } else {
                    ui.label(format!("Output: {} × {} px", crop_px_w, crop_px_h));
                }
            }

            if ui.button("Apply Crop (Enter)").clicked() {
                actions.apply_crop = true;
            }
            // }
        });
    }

    fn draw_effects_section(
        &mut self,
        ui: &mut egui::Ui,
        effect_stack: &mut EffectStack,
        actions: &mut UiActions,
    ) {
        if self.crop.enabled {
            return;
        }

        if ui.checkbox(&mut effect_stack.enabled, "Enabled").changed() {
            actions.needs_rebuild = true;
            actions.needs_compute = true;
            actions.request_redraw = true;
        }

        if effect_stack.draw_presets_ui(ui) {
            actions.needs_compute = true;
            actions.request_redraw = true;
        }

        let _ = egui::ComboBox::from_label("Add Effect")
            .selected_text("Add Effect")
            .show_ui(ui, |ui| {
                EffectSelector::draw_all(ui, effect_stack);
            });

        let mut swap_index = None;
        let mut remove_idx = None;
        let len = effect_stack.effects.len();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (idx, container) in effect_stack.effects.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let mut is_enabled = container.enabled();
                        if ui.checkbox(&mut is_enabled, container.name()).changed() {
                            container.set_enabled(is_enabled);
                            actions.needs_rebuild = true;
                            actions.needs_compute = true;
                            actions.request_redraw = true;
                        }
                        if ui.button("X").clicked() {
                            remove_idx = Some(idx);
                            actions.needs_rebuild = true;
                            actions.needs_compute = true;
                            actions.request_redraw = true;
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
                            actions.needs_compute = true;
                        }
                    }
                    ui.separator();
                }
            });

        if let Some(idx) = remove_idx {
            effect_stack.effects.remove(idx);
            actions.needs_rebuild = true;
            actions.needs_compute = true;
            actions.request_redraw = true;
        }

        if let Some((i, j)) = swap_index {
            effect_stack.effects.swap(i, j);
            actions.needs_rebuild = true;
            actions.needs_compute = true;
            actions.request_redraw = true;
        }
    }
}
