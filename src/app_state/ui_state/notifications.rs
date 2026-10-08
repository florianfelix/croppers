use std::time::Instant;

pub struct Notification {
    pub message: String,
    pub created_at: Instant,
    pub duration: f32, // in seconds
    pub is_error: bool,
}

pub struct Notifications {
    items: Vec<Notification>,
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, message: &str, is_error: bool) {
        self.items.push(Notification {
            message: message.into(),
            created_at: Instant::now(),
            duration: 3.0,
            is_error,
        });
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        self.items
            .retain(|n| now.duration_since(n.created_at).as_secs_f32() < n.duration);

        if self.items.is_empty() {
            return;
        }

        // Keep animating while there are notifications
        ctx.request_repaint();

        egui::Window::new("Notifications")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    // Iterate in reverse so newest is at the bottom
                    for n in self.items.iter().rev() {
                        let age = now.duration_since(n.created_at).as_secs_f32();
                        let fade_in = 0.15;
                        let fade_out = 0.5;

                        let alpha = if age < fade_in {
                            age / fade_in
                        } else if age > n.duration - fade_out {
                            (n.duration - age) / fade_out
                        } else {
                            1.0
                        }
                        .clamp(0.0, 1.0);

                        let mut bg_color = if n.is_error {
                            egui::Color32::from_rgb(180, 50, 50)
                        } else {
                            egui::Color32::from_rgb(50, 160, 80)
                        };

                        let fg_color = egui::Color32::from_white_alpha((255.0 * alpha) as u8);

                        // Manually apply alpha to background
                        bg_color = egui::Color32::from_rgba_premultiplied(
                            (bg_color.r() as f32 * alpha) as u8,
                            (bg_color.g() as f32 * alpha) as u8,
                            (bg_color.b() as f32 * alpha) as u8,
                            (255.0 * alpha * 0.9) as u8, // slightly transparent background
                        );

                        egui::Frame::window(ui.style())
                            .fill(bg_color)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(12.0)
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 4],
                                blur: 12,
                                spread: 0,
                                color: egui::Color32::from_black_alpha((100.0 * alpha) as u8),
                            })
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&n.message)
                                        .color(fg_color)
                                        .size(14.0)
                                        .strong(),
                                );
                            });
                        ui.add_space(8.0);
                    }
                });
            });
    }
}
