use crate::app_state::AppState;
use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;
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
    ) -> (Vec<egui::ClippedPrimitive>, egui_wgpu::ScreenDescriptor) {
        let raw_input = self.egui_winit.take_egui_input(window);
        let full_output = self.egui_ctx.run_ui(raw_input.clone(), |ctx| {
            app_state.draw(&raw_input, ctx);
        });

        self.egui_winit
            .handle_platform_output(window, full_output.platform_output);

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

        (tris, screen_descriptor)
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
