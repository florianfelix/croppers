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
                    ui.group(|ui| {
                        ui.strong("On Image Load");
                        ui.checkbox(&mut config.on_image_load.crop_enabled, "Enable Crop");
                        ui.checkbox(&mut config.on_image_load.effects_enabled, "Enable Effects");
                        ui.checkbox(
                            &mut config.on_image_load.empty_effects,
                            "Clear Effects Stack",
                        );
                    });

                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.strong("Startup Effects");
                        ui.checkbox(
                            &mut config.startup_effects.enabled,
                            "Enable Effects on Startup",
                        );
                    });

                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.strong("Startup UI");
                        ui.checkbox(
                            &mut config.startup_ui.show_panel,
                            "Show Left Panel on Startup",
                        );
                    });

                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.strong("Startup Crop");
                        ui.checkbox(&mut config.startup_crop.enabled, "Enable Crop on Startup");
                        egui::ComboBox::from_label("Default Crop Mode")
                            .selected_text(config.startup_crop.default_crop_mode.label())
                            .show_ui(ui, |ui| {
                                for mode in CropMode::ALL {
                                    ui.selectable_value(
                                        &mut config.startup_crop.default_crop_mode,
                                        mode,
                                        mode.label(),
                                    );
                                }
                            });
                        if config.startup_crop.default_crop_mode == CropMode::FixedDimensions {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut config.startup_crop.fixed_crop_width)
                                        .range(1..=10000),
                                );
                                ui.label("Width");
                            });
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(
                                        &mut config.startup_crop.fixed_crop_height,
                                    )
                                    .range(1..=10000),
                                );
                                ui.label("Height");
                            });
                        }
                    });

                    ui.add_space(16.0);
                    if ui.button("Save Settings").clicked() {
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
                                ui.label(egui::RichText::new("N / Right Arrow").monospace());
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
                                ui.label(egui::RichText::new("Ctrl + Shift + S").monospace());
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

        actions
    }
}
