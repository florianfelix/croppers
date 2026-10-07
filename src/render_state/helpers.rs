use std::sync::Arc;

use winit::dpi::PhysicalSize;

use crate::render_state::RenderState;

impl RenderState {
    pub fn handle_resize(&mut self, physical_size: PhysicalSize<u32>) {
        if physical_size.width > 0 && physical_size.height > 0 {
            self.config.width = physical_size.width;
            self.config.height = physical_size.height;
            let _ = self.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            });
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn get_surface_texture(&self) -> Option<wgpu::SurfaceTexture> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                let _ = self.device.poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                });
                self.surface.configure(&self.device, &self.config);
                texture
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let _ = self.device.poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                });
                self.surface.configure(&self.device, &self.config);
                return None;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return None;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                eprintln!("Surface validation error encountered");
                return None;
            }
        };
        Some(surface_texture)
    }

    pub fn create_dummy_texture(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> (Arc<wgpu::Texture>, Arc<wgpu::TextureView>, [u32; 2]) {
        let texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 0, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (Arc::new(texture), Arc::new(texture_view), [1, 1])
    }
}
