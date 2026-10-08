use std::sync::Arc;
use tracing::{error, info};

use crate::{
    app_state::{AppState, files::ImageSource},
    render_state::RenderState,
};

impl AppState {
    #[tracing::instrument(skip(self))]
    pub fn set_srcimg_texture_from_source(&mut self) {
        let Some(ref img) = self.source else { return };
        let Some((ref device, ref queue)) = self.gpu else {
            return;
        };

        info!("Creating srcimg textures. {:?}", img.path);

        let (width, height) = img.image_size;
        let (texture, view, dims) = if width == 0 || height == 0 || width > 8192 || height > 8192 {
            let msg = format!("Invalid image size {width}x{height}");
            error!("{msg}");
            self.ui_state.notifications.push(&msg, true);
            RenderState::create_dummy_texture(device.clone(), queue.clone())
        } else {
            texture_and_view_from_source_img(img, device.clone(), queue.clone())
        };

        self.texture_vault.set_srcimg(texture, view, dims);
        self.texture_vault.effect = self.texture_vault.srcimg.clone();
        self.texture_vault.clear_cropped();
        self.needs_compute = true;
    }
}

pub fn texture_and_view_from_source_img(
    img: &ImageSource,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
) -> (Arc<wgpu::Texture>, Arc<wgpu::TextureView>, [u32; 2]) {
    let (width, height) = img.image_size;

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Loaded Image Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texture = Arc::new(texture);

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    let texture_view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
    let dims = [width, height];

    (texture, texture_view, dims)
}
