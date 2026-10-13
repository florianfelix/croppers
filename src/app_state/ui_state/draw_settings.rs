use crate::app_state::{
    ui_state::{UiActions, UiState},
    widgets::CropMode,
};

impl UiState {
    pub fn draw_settings_panel(
        &mut self,
        ctx: &mut egui::Ui,
        config: &mut crate::utils::CroppersConfig,
    ) -> UiActions {
        let mut actions = UiActions::default();

        if !self.show_settings {
            return actions;
        }

        egui::Panel::right("config_panel")
            .resizable(true)
            .default_size(300.0)
            .show(ctx, |ui| {
                ui.heading("Settings");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::new()
                        .inner_margin(egui::Margin {
                            left: 0,
                            right: 14,
                            top: 0,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            ui.group(|ui| {
                                ui.strong("Appearance");
                                egui::ComboBox::from_label("")
                                    .selected_text(config.theme_mode.label())
                                    .show_ui(ui, |ui| {
                                        for mode in crate::utils::ThemeMode::ALL {
                                            if ui
                                                .selectable_value(
                                                    &mut config.theme_mode,
                                                    mode,
                                                    mode.label(),
                                                )
                                                .changed()
                                            {
                                                actions.request_redraw = true;
                                            }
                                        }
                                    });

                                let accent_rgba = egui::Rgba::from_rgba_unmultiplied(
                                    config.appearance.accent_color[0],
                                    config.appearance.accent_color[1],
                                    config.appearance.accent_color[2],
                                    config.appearance.accent_color[3],
                                );
                                let mut accent_c32 = accent_rgba.into();
                                if ui
                                    .add(
                                        elegance::ColorPicker::new(
                                            "accent_picker",
                                            &mut accent_c32,
                                        )
                                        .alpha(false),
                                    )
                                    .changed()
                                {
                                    let new_rgba = egui::Rgba::from(accent_c32);
                                    config.appearance.accent_color = new_rgba.to_array();
                                    actions.request_redraw = true;
                                }

                                ui.add_space(8.0);
                                egui::CollapsingHeader::new("Canvas Settings")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("Canvas Background");
                                        let bg_rgba = egui::Rgba::from_rgba_unmultiplied(
                                            config.appearance.canvas_bg_color[0],
                                            config.appearance.canvas_bg_color[1],
                                            config.appearance.canvas_bg_color[2],
                                            config.appearance.canvas_bg_color[3],
                                        );
                                        let mut bg_c32 = bg_rgba.into();
                                        if ui
                                            .add(
                                                elegance::ColorPicker::new(
                                                    "bg_picker",
                                                    &mut bg_c32,
                                                )
                                                .alpha(true),
                                            )
                                            .changed()
                                        {
                                            let new_rgba = egui::Rgba::from(bg_c32);
                                            config.appearance.canvas_bg_color = new_rgba.to_array();
                                            actions.request_redraw = true;
                                        }

                                        ui.add_space(4.0);
                                        ui.label("Shadow Color");
                                        let shadow_rgba = egui::Rgba::from_rgba_unmultiplied(
                                            config.appearance.canvas_shadow_color[0],
                                            config.appearance.canvas_shadow_color[1],
                                            config.appearance.canvas_shadow_color[2],
                                            config.appearance.canvas_shadow_color[3],
                                        );
                                        let mut shadow_c32 = shadow_rgba.into();
                                        if ui
                                            .add(
                                                elegance::ColorPicker::new(
                                                    "shadow_picker",
                                                    &mut shadow_c32,
                                                )
                                                .alpha(true),
                                            )
                                            .changed()
                                        {
                                            let new_rgba = egui::Rgba::from(shadow_c32);
                                            config.appearance.canvas_shadow_color =
                                                new_rgba.to_array();
                                            actions.request_redraw = true;
                                        }

                                        ui.add_space(4.0);
                                        if ui
                                            .add(
                                                elegance::Slider::new(
                                                    &mut config.appearance.canvas_radius,
                                                    0.0..=500.0,
                                                )
                                                .label("Shadow Radius"),
                                            )
                                            .changed()
                                        {
                                            actions.request_redraw = true;
                                        }
                                        if ui
                                            .add(
                                                elegance::Slider::new(
                                                    &mut config.appearance.canvas_shadow_offset_x,
                                                    -100.0..=100.0,
                                                )
                                                .label("Shadow Offset X"),
                                            )
                                            .changed()
                                        {
                                            actions.request_redraw = true;
                                        }

                                        if ui
                                            .add(
                                                elegance::Slider::new(
                                                    &mut config.appearance.canvas_shadow_offset_y,
                                                    -100.0..=100.0,
                                                )
                                                .label("Shadow Offset Y"),
                                            )
                                            .changed()
                                        {
                                            actions.request_redraw = true;
                                        }
                                    });
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.strong("On Image Load");
                                ui.checkbox(&mut config.on_image_load.crop_enabled, "Enable Crop");
                                ui.checkbox(
                                    &mut config.on_image_load.effects_enabled,
                                    "Enable Effects",
                                );
                                ui.checkbox(
                                    &mut config.on_image_load.empty_effects,
                                    "Clear Effects Stack",
                                );
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.strong("On Startup");
                                ui.separator();
                                ui.checkbox(
                                    &mut config.startup.enable_effects,
                                    "Enable Effects on Startup",
                                );

                                ui.checkbox(
                                    &mut config.startup.show_panel,
                                    "Show Left Panel on Startup",
                                );

                                ui.add_space(8.0);
                                ui.strong("Startup Crop Settings");
                                ui.checkbox(
                                    &mut config.startup.default_crop_setting.enabled,
                                    "Enable Crop on Startup",
                                );
                                egui::ComboBox::from_label("Mode")
                                    .selected_text(
                                        config
                                            .startup
                                            .default_crop_setting
                                            .default_crop_mode
                                            .label(),
                                    )
                                    .show_ui(ui, |ui| {
                                        for mode in CropMode::ALL {
                                            ui.selectable_value(
                                                &mut config
                                                    .startup
                                                    .default_crop_setting
                                                    .default_crop_mode,
                                                mode,
                                                mode.label(),
                                            );
                                        }
                                    });
                                if config.startup.default_crop_setting.default_crop_mode
                                    == CropMode::FixedDimensions
                                {
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut config
                                                    .startup
                                                    .default_crop_setting
                                                    .fixed_crop_width,
                                            )
                                            .range(1..=10000),
                                        );
                                        ui.label("Width");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut config
                                                    .startup
                                                    .default_crop_setting
                                                    .fixed_crop_height,
                                            )
                                            .range(1..=10000),
                                        );
                                        ui.label("Height");
                                    });
                                }
                            });

                            ui.add_space(8.0);

                            ui.add_space(16.0);
                            if ui.add(elegance::Button::new("Save Settings")).clicked() {
                                actions.save_config = true;
                            }

                            ui.add_space(16.0);
                            ui.group(|ui| {
                                ui.strong("Keyboard Shortcuts");
                                ui.add_space(4.0);

                                egui::Grid::new("shortcuts_grid")
                                    .num_columns(2)
                                    .spacing([20.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Open File");
                                        ui.label(egui::RichText::new("Ctrl + O").monospace());
                                        ui.end_row();
                                        ui.label("Toggle Left Panel");
                                        ui.label(egui::RichText::new("W").monospace());
                                        ui.end_row();
                                        ui.label("Toggle Settings");
                                        ui.label(
                                            egui::RichText::new("?")
                                                .monospace()
                                                .color(egui::Color32::from_rgb(255, 100, 100)),
                                        );
                                        ui.end_row();
                                        ui.label("Next Image");
                                        ui.label(
                                            egui::RichText::new("N / Right Arrow").monospace(),
                                        );
                                        ui.end_row();
                                        ui.label("Previous Image");
                                        ui.label(egui::RichText::new("B / Left Arrow").monospace());
                                        ui.end_row();
                                        ui.label("Toggle Crop");
                                        ui.label(egui::RichText::new("C").monospace());
                                        ui.end_row();
                                        ui.label("Toggle Effects");
                                        ui.label(egui::RichText::new("E").monospace());
                                        ui.end_row();
                                        ui.label("Reset View");
                                        ui.label(egui::RichText::new("R").monospace());
                                        ui.end_row();
                                        ui.label("Apply Crop");
                                        ui.label(egui::RichText::new("Enter").monospace());
                                        ui.end_row();
                                        ui.label("Save File");
                                        ui.label(egui::RichText::new("Ctrl + S").monospace());
                                        ui.end_row();
                                        ui.label("Save Increment");
                                        ui.label(
                                            egui::RichText::new("Ctrl + Shift + S").monospace(),
                                        );
                                        ui.end_row();
                                        ui.label("Delete File");
                                        ui.label(egui::RichText::new("Delete").monospace());
                                        ui.end_row();
                                        ui.label("Quit");
                                        ui.label(egui::RichText::new("Q").monospace());
                                        ui.end_row();
                                    });
                            });
                        });
                });
            });

        actions
    }
}
