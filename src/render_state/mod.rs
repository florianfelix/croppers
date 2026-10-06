use std::sync::Arc;
#[allow(unused)]
use tracing::{info, trace, warn};
use winit::window::Window;

use crate::{
    app_state::AppState,
    render_state::{canvas_renderer::CanvasRenderer, ui_renderer::UiRenderer},
};

pub mod canvas_renderer;
pub mod helpers;
pub mod ui_renderer;

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub window: Arc<Window>,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub ui_renderer: UiRenderer,
    pub canvas_renderer: CanvasRenderer,
    pub dummy: (Arc<wgpu::Texture>, Arc<wgpu::TextureView>),
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .expect("Failed to find suitable GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Failed to create logical device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                *f == wgpu::TextureFormat::Bgra8Unorm || *f == wgpu::TextureFormat::Rgba8Unorm
            })
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        let (dummy_texture, dummy_texture_view) =
            Self::create_dummy_texture(device.clone(), queue.clone());

        let ui_renderer = UiRenderer::new(&device, &config, &window);
        let canvas_renderer = CanvasRenderer::new(
            device.clone(),
            window.clone(),
            queue.clone(),
            config.format,
            &dummy_texture_view,
        );

        Self {
            surface,
            device,
            queue,
            window,
            config,
            adapter,

            ui_renderer,
            canvas_renderer,
            dummy: (dummy_texture, dummy_texture_view),
        }
    }
    #[tracing::instrument(skip(self, _app_state))]
    pub fn update(&mut self, _app_state: &mut AppState) {}

    #[tracing::instrument(skip(self, app_state))]
    pub fn render(&mut self, app_state: &mut AppState) -> Result<(), wgpu::hal::SurfaceError> {
        // info!("Render");
        let surface_texture = match self.get_surface_texture() {
            Some(surface_texture) => surface_texture,
            None => return Ok(()),
        };

        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let (tris, screen_descriptor, needs_repaint) = self.ui_renderer.prepare_frame(
            &self.window,
            &self.device,
            &self.queue,
            &self.config,
            app_state,
        );
        if needs_repaint {
            app_state.request_redraw_wake();
        }

        self.ui_renderer.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        let (display_view, display_dims) = match app_state.get_view_auto() {
            Some((display_view, display_dims)) => {
                trace!("Rendering display view");
                (display_view, display_dims)
            }
            None => {
                warn!("No display_view found. Rendering dummy");
                (self.dummy.1.clone(), [1, 1])
            }
        };
        self.canvas_renderer.update_texture_binding(&display_view);

        // -----------------------------------------------------------------
        // FINAL RENDER PASS
        // -----------------------------------------------------------------
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                ..Default::default()
            });

            self.canvas_renderer.render(
                &mut render_pass,
                app_state.ui_state.canvas_rect(),
                Some((display_dims[0] as f32, display_dims[1] as f32)),
                // self.image_size,
            );

            self.ui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &self.config,
                &tris,
                &screen_descriptor,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.queue.present(surface_texture);

        Ok(())
    }
}
