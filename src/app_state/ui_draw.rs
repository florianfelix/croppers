use crate::app_state::{AppState, ui_state::UiAction};

impl AppState {
    pub fn handle_ui_actions(&mut self, actions: &[UiAction]) {
        for action in actions {
            match action {
                UiAction::SaveOverwrite => self.save_effect_texture(false),
                UiAction::SaveIncrement => self.save_effect_texture(true),
                UiAction::ApplyCrop => self.send_crop_job(),
                UiAction::SaveConfig => {
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
                UiAction::NeedsRebuild => {
                    self.effect_stack.needs_rebuild = true;
                    self.effect_stack.validate_ids();
                }
                UiAction::OpenFileDialog => self.open_file_dialog(),
                UiAction::NeedsCompute => self.needs_compute = true,
                UiAction::RequestRedraw => self.request_redraw_wake(),
                UiAction::ToggleSettings => self.ui_state.toggle_settings(),
                UiAction::ClearEffectStack => {
                    self.effect_stack.effects.clear();
                    self.effect_stack.needs_rebuild = true;
                }
            }
        }
    }

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

        let mut actions = Vec::new();

        self.ui_state.draw_panel(
            ctx,
            &self.source,
            src_dims,
            crop_dims,
            &mut self.effect_stack,
            &mut actions,
        );

        self.ui_state
            .draw_settings_panel(ctx, &mut self.config, &mut actions);

        self.ui_state.draw_canvas(ctx, &self.source, src_dims);

        self.handle_ui_actions(&actions);

        self.ui_state.notifications.draw(ctx.ctx());
    }
}
