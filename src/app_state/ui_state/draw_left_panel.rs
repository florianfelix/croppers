use itertools::Itertools;

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
                ui.horizontal(|ui| {
                    ui.heading("Image Controls");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                elegance::Button::icon(elegance::glyphs::SETTINGS, "Settings")
                                    .outline()
                                    .size(elegance::ButtonSize::Small),
                            )
                            .clicked()
                        {
                            actions.toggle_settings = true;
                        }
                    });
                });

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
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            elegance::Button::new("Open File")
                                .outline()
                                .size(elegance::ButtonSize::Small),
                        )
                        .clicked()
                    {
                        actions.open_file_dialog = true;
                    }

                    ui.add_enabled_ui(!self.crop.enabled, |ui| {
                        if ui
                            .add(
                                elegance::Button::new("Save")
                                    .outline()
                                    .size(elegance::ButtonSize::Small),
                            )
                            .clicked()
                        {
                            actions.save_overwrite = true;
                        }
                        if ui
                            .add(
                                elegance::Button::new("Save Increment")
                                    .outline()
                                    .size(elegance::ButtonSize::Small),
                            )
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
            ui.add(elegance::Switch::new(&mut self.crop.enabled, "Crop"));
            // ui.checkbox(&mut self.crop.enabled, "Crop");

            if !self.crop.enabled {
                return;
            }

            let options = CropMode::ALL
                .into_iter()
                .map(|m| (m, m.label()))
                .collect_vec();
            ui.add(elegance::Select::new("Mode", &mut self.crop.mode).options(options));

            // egui::ComboBox::from_label("Mode")
            //     .selected_text(self.crop.mode.label())
            //     .show_ui(ui, |ui| {
            //         for mode in CropMode::ALL {
            //             ui.selectable_value(&mut self.crop.mode, mode, mode.label());
            //         }
            //     });

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

            if ui
                .add(elegance::Button::new("Apply Crop (Enter)"))
                .clicked()
            {
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

        if ui
            .add(elegance::Switch::new(&mut effect_stack.enabled, "Enabled"))
            .changed()
        {
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

        let mut remove_idx = None;
        let mut swap_index = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let theme = elegance::Theme::current(ui.ctx());

                for (idx, container) in effect_stack.effects.iter_mut().enumerate() {
                    let id = egui::Id::new("effect_dnd").with(container.id());

                    let frame = egui::Frame::new()
                        .fill(theme.palette.card)
                        .stroke(egui::Stroke::new(1.0, theme.palette.border))
                        .corner_radius(egui::CornerRadius::same(theme.card_radius as u8))
                        .inner_margin(egui::Margin {
                            left: 0,
                            right: 8,
                            top: 0,
                            bottom: 0,
                        })
                        .outer_margin(egui::Margin {
                            left: 0,
                            right: 14,
                            top: 0,
                            bottom: 0,
                        });

                    let (_, dropped_payload) = ui.dnd_drop_zone::<usize, _>(frame, |ui| {
                        let inner_avail = ui.available_width();
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            let _drag_resp = ui.dnd_drag_source(id, idx, |ui| {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(28.0, 44.0),
                                    egui::Sense::click_and_drag(),
                                );
                                let painter = ui.painter();
                                let color = if resp.hovered() || resp.dragged() {
                                    ui.visuals().text_color()
                                } else {
                                    theme.palette.focus
                                };
                                let center = rect.center();
                                for col in 0..2 {
                                    for row in 0..3 {
                                        let cx = center.x - 2.0 + col as f32 * 4.0;
                                        let cy = center.y - 5.0 + row as f32 * 5.0;
                                        painter.circle_filled(egui::pos2(cx, cy), 1.3, color);
                                    }
                                }
                                resp
                            });

                            let right_w = (inner_avail - 28.1).max(0.0);
                            ui.allocate_ui_with_layout(
                                egui::vec2(right_w, 0.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.add_space(10.0); // top padding

                                    let is_collapsed = container.ui_collapsed();
                                    let chevron = if is_collapsed {
                                        elegance::glyphs::CHEVRON_RIGHT
                                    } else {
                                        elegance::glyphs::CHEVRON_DOWN
                                    };

                                    ui.horizontal(|ui| {
                                        let title = format!("{} {}", chevron, container.name());
                                        if ui.add(egui::Button::new(title).frame(false)).clicked() {
                                            container.set_ui_collapsed(!is_collapsed);
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add(elegance::Button::icon(
                                                        elegance::glyphs::TRASH,
                                                        "Remove",
                                                    ))
                                                    .clicked()
                                                {
                                                    remove_idx = Some(idx);
                                                    actions.needs_rebuild = true;
                                                    actions.needs_compute = true;
                                                    actions.request_redraw = true;
                                                }
                                                ui.add_space(12.0);
                                                let mut is_enabled = container.enabled();
                                                if ui
                                                    .add(elegance::Switch::new(&mut is_enabled, ""))
                                                    .changed()
                                                {
                                                    container.set_enabled(is_enabled);
                                                    actions.needs_rebuild = true;
                                                    actions.needs_compute = true;
                                                    actions.request_redraw = true;
                                                }
                                            },
                                        );
                                    });

                                    if !is_collapsed {
                                        ui.add_space(4.0);
                                        ui.add_enabled_ui(container.enabled(), |ui| {
                                            if container.draw_ui(ui) {
                                                actions.needs_compute = true;
                                            }
                                        });
                                    }

                                    ui.add_space(10.0); // bottom padding
                                },
                            );
                        });
                    });

                    ui.add_space(6.0);

                    if let Some(source_idx) = dropped_payload {
                        if *source_idx != idx {
                            swap_index = Some((*source_idx, idx));
                        }
                    }
                }
            });

        if let Some(idx) = remove_idx {
            effect_stack.effects.remove(idx);
            actions.needs_rebuild = true;
            actions.needs_compute = true;
            actions.request_redraw = true;
        }

        if let Some((i, j)) = swap_index {
            let moved = effect_stack.effects.remove(i);
            effect_stack.effects.insert(j, moved);
            actions.needs_rebuild = true;
            actions.needs_compute = true;
            actions.request_redraw = true;
        }
    }
}
