use crate::app_state::AppState;

impl AppState {
    pub fn draw(&mut self, _raw_input: &egui::RawInput, ctx: &mut egui::Ui) {
        let src_dims = self.texture_vault.srcimg.dims;

        let actions =
            self.ui_state
                .draw_panel(ctx, self.source.as_ref(), src_dims, &mut self.effect_stack);

        self.ui_state
            .draw_canvas(ctx, self.source.as_ref(), src_dims);

        if actions.save_overwrite {
            self.save_effect_texture(false);
        }
        if actions.save_increment {
            self.save_effect_texture(true);
        }
        if actions.apply_crop {
            self.send_crop_job();
        }
        if actions.needs_rebuild {
            self.effect_stack.needs_rebuild = true;
            self.effect_stack.validate_ids();
        }
        if actions.needs_compute {
            self.needs_compute = true;
        }
        if actions.request_redraw {
            self.request_redraw_wake();
        }
        
        self.ui_state.notifications.draw(ctx.ctx());
    }
}
