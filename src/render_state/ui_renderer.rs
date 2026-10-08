use crate::app_state::AppState;
use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;
#[allow(unused)]
use tracing::info;
use wgpu::{Device, Queue, SurfaceConfiguration};
use winit::window::Window;

pub struct UiRenderer {
    pub egui_ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
}

impl UiRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration, window: &Arc<Window>) -> Self {
        let egui_ctx = egui::Context::default();
        setup_custom_fonts(&egui_ctx);
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        // Apply modern minimalist theme
        MinimalistTheme::default().apply(&egui_ctx);

        let egui_render_options = egui_wgpu::RendererOptions {
            msaa_samples: 1,
            depth_stencil_format: None,
            dithering: true,
            predictable_texture_filtering: false,
        };
        let egui_renderer = egui_wgpu::Renderer::new(device, config.format, egui_render_options);

        Self {
            egui_ctx,
            egui_winit,
            egui_renderer,
        }
    }

    /// Processes winit events and renders UI panels, returning tessellated primitive data and descriptors.
    pub fn prepare_frame(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
        app_state: &mut AppState,
    ) -> (
        Vec<egui::ClippedPrimitive>,
        egui_wgpu::ScreenDescriptor,
        bool,
    ) {
        let raw_input = self.egui_winit.take_egui_input(window);
        let full_output = self.egui_ctx.run_ui(raw_input.clone(), |ctx| {
            app_state.draw(&raw_input, ctx);
        });

        self.egui_winit
            .handle_platform_output(window, full_output.platform_output);

        let needs_immediate_repaint = full_output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|v| v.repaint_delay.is_zero())
            .unwrap_or(false);

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let mut textures_delta = full_output.textures_delta;
        for (id, delta_list) in &textures_delta.set {
            for delta in delta_list {
                self.egui_renderer.update_texture(device, queue, *id, delta);
            }
        }
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        textures_delta.clear();

        (tris, screen_descriptor, needs_immediate_repaint)
    }
    pub fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'static>,
        config: &SurfaceConfiguration,
        tris: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        render_pass.set_viewport(
            0.0,
            0.0,
            config.width as f32,
            config.height as f32,
            0.0,
            1.0,
        );
        self.egui_renderer
            .render(render_pass, tris, screen_descriptor);
    }
}

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Embed font file at compile time
    fonts.font_data.insert(
        "dejavu_sans".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/DejaVuSans.ttf"
        ))),
    );

    // Put it at the top of the priority list for Proportional text
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "dejavu_sans".to_owned());

    // Also add to Monospace if needed
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("dejavu_sans".to_owned());

    // Apply font definitions to egui context
    ctx.set_fonts(fonts);
}

pub struct MinimalistTheme {
    pub window_fill: egui::Color32,
    pub panel_fill: egui::Color32,
    pub widget_bg: egui::Color32,
    pub accent: egui::Color32,
    pub accent_hover: egui::Color32,
    pub stroke_color: egui::Color32,
    pub text_primary: egui::Color32,
    pub corner_radius: u8,
    pub item_spacing: egui::Vec2,
    pub button_padding: egui::Vec2,
}

impl Default for MinimalistTheme {
    fn default() -> Self {
        Self {
            window_fill: egui::Color32::from_rgb(26, 26, 28),
            panel_fill: egui::Color32::from_rgb(20, 20, 22),
            widget_bg: egui::Color32::from_rgb(32, 32, 35),
            accent: egui::Color32::from_rgb(80, 140, 200), // Clean modern blue accent
            accent_hover: egui::Color32::from_rgb(100, 160, 220),
            stroke_color: egui::Color32::from_rgb(50, 50, 55),
            text_primary: egui::Color32::from_rgb(230, 230, 235),
            corner_radius: 4, // Subtle rounded corners
            item_spacing: egui::vec2(8.0, 6.0),
            button_padding: egui::vec2(8.0, 4.0),
        }
    }
}

impl MinimalistTheme {
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Colors
        visuals.window_fill = self.window_fill;
        visuals.panel_fill = self.panel_fill;
        visuals.widgets.noninteractive.bg_fill = self.widget_bg;
        visuals.widgets.inactive.bg_fill = self.widget_bg;
        visuals.widgets.hovered.bg_fill = self.accent_hover;
        visuals.widgets.active.bg_fill = self.accent;
        visuals.selection.bg_fill = self.accent;

        // Clean strokes
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, self.stroke_color);
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.stroke_color);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.accent_hover);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, self.accent);
        visuals.selection.stroke = egui::Stroke::new(1.0, self.accent);

        // Text
        visuals.widgets.noninteractive.fg_stroke.color = self.text_primary;
        visuals.widgets.inactive.fg_stroke.color = self.text_primary;
        visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
        visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;

        // Override text color on selection and strong backgrounds specifically
        visuals.selection.stroke.color = egui::Color32::WHITE;

        // Modern shape geometry
        visuals.slider_trailing_fill = true;
        let radius = egui::CornerRadius::same(self.corner_radius);
        visuals.window_corner_radius = radius;
        visuals.menu_corner_radius = radius;
        visuals.widgets.noninteractive.corner_radius = radius;
        visuals.widgets.inactive.corner_radius = radius;
        visuals.widgets.hovered.corner_radius = radius;
        visuals.widgets.active.corner_radius = radius;

        ctx.set_visuals(visuals);

        // Spacing & Animation
        ctx.global_style_mut(|style| {
            style.animation_time = 0.0; // Smooth but snappy transitions
            style.spacing.item_spacing = self.item_spacing;
            style.spacing.button_padding = self.button_padding;
        });
    }
}
