use crate::app_state::AppState;

impl AppState {
    pub fn draw(
        &mut self,
        _raw_input: &egui::RawInput,
        ctx: &mut egui::Ui,
        window_theme: Option<winit::window::Theme>,
    ) {
        let is_dark = match self.config.theme_mode {
            crate::utils::ThemeMode::Auto => {
                window_theme.unwrap_or(winit::window::Theme::Dark) == winit::window::Theme::Dark
            }
            crate::utils::ThemeMode::Slate => true,
            crate::utils::ThemeMode::Charcoal => true,
            crate::utils::ThemeMode::Frost => false,
            crate::utils::ThemeMode::Paper => false,
        };

        let mut theme = match self.config.theme_mode {
            crate::utils::ThemeMode::Slate => elegance::Theme::slate(),
            crate::utils::ThemeMode::Charcoal => elegance::Theme::charcoal(),
            crate::utils::ThemeMode::Frost => elegance::Theme::frost(),
            crate::utils::ThemeMode::Paper => elegance::Theme::paper(),
            crate::utils::ThemeMode::Auto => {
                if is_dark {
                    elegance::Theme::slate()
                } else {
                    elegance::Theme::frost()
                }
            }
        };

        let accent_rgba = egui::Rgba::from_rgba_unmultiplied(
            self.config.appearance.accent_color[0],
            self.config.appearance.accent_color[1],
            self.config.appearance.accent_color[2],
            self.config.appearance.accent_color[3],
        );
        let accent_c32: egui::Color32 = accent_rgba.into();
        let accent_c32_muted = accent_c32.gamma_multiply(0.5);

        theme.palette.focus = accent_c32; // focus rings, selection, active states
        theme.palette.blue = accent_c32; // the default (Blue) primary button
        theme.palette.blue_hover = accent_c32_muted;

        theme.install(ctx);
        let src_dims = self.texture_vault.srcimg.dims;
        let crop_dims = if self.texture_vault.cropped.dims[0] > 0 {
            Some(self.texture_vault.cropped.dims)
        } else {
            None
        };

        let mut actions = self.ui_state.draw_panel(
            ctx,
            &self.source,
            src_dims,
            crop_dims,
            &mut self.effect_stack,
        );

        let settings_actions = self.ui_state.draw_settings_panel(ctx, &mut self.config);
        actions.save_config = actions.save_config || settings_actions.save_config;

        self.ui_state.draw_canvas(ctx, &self.source, src_dims);

        if actions.save_overwrite {
            self.save_effect_texture(false);
        }
        if actions.save_increment {
            self.save_effect_texture(true);
        }
        if actions.apply_crop {
            self.send_crop_job();
        }
        if actions.save_config {
            if let Err(e) = self.config.save() {
                self.ui_state
                    .notifications
                    .push(&format!("Failed to save config: {}", e), true);
            } else {
                self.ui_state
                    .notifications
                    .push("Config saved successfully", false);
            }
        }
        if actions.needs_rebuild {
            self.effect_stack.needs_rebuild = true;
            self.effect_stack.validate_ids();
        }
        if actions.open_file_dialog {
            self.open_file_dialog();
        }
        if actions.needs_compute {
            self.needs_compute = true;
        }
        if actions.request_redraw {
            self.request_redraw_wake();
        }
        if actions.toggle_settings {
            self.ui_state.toggle_settings();
        }

        self.ui_state.notifications.draw(ctx.ctx());
    }
}
