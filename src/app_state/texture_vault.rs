use std::sync::Arc;

use tracing::{info, trace};

#[derive(Default)]
pub struct TextureVault {
    // pub display_view_dims: [u32; 2],
    // pub display_view: Option<Arc<wgpu::TextureView>>,
    pub srcimg_dims: [u32; 2],
    pub srcimg_texture: Option<Arc<wgpu::Texture>>,
    pub srcimg_texture_view: Option<Arc<wgpu::TextureView>>,

    pub effect_dims: [u32; 2],
    pub effect_view: Option<Arc<wgpu::TextureView>>,

    pub cropped_dims: [u32; 2],
    pub cropped_texture: Option<Arc<wgpu::Texture>>,
    pub cropped_texture_view: Option<Arc<wgpu::TextureView>>,
}

impl TextureVault {
    #[tracing::instrument(skip(self))]
    pub fn set_srcimg(
        &mut self,
        srcimg_texture: Arc<wgpu::Texture>,
        srcimg_texture_view: Arc<wgpu::TextureView>,
        dims: [u32; 2],
    ) {
        self.srcimg_texture = Some(srcimg_texture);
        self.srcimg_texture_view = Some(srcimg_texture_view);
        self.srcimg_dims = dims;
    }

    #[tracing::instrument(skip(self))]
    pub fn set_cropped(
        &mut self,
        texture: Arc<wgpu::Texture>,
        texture_view: Arc<wgpu::TextureView>,
    ) {
        self.cropped_dims = [texture.width(), texture.height()];
        self.cropped_texture = Some(texture);
        self.cropped_texture_view = Some(texture_view);
    }
    #[tracing::instrument(skip(self))]
    pub fn clear_cropped(&mut self) {
        self.cropped_dims = [0, 0];
        self.cropped_texture = None;
        self.cropped_texture_view = None;
    }
    #[tracing::instrument(skip(self))]
    pub fn clear_effects(&mut self) {
        self.effect_dims = [0, 0];
        self.effect_view = None;
    }
}
