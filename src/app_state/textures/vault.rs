use std::sync::Arc;

#[derive(Default, Clone, Debug)]
pub struct TextureSet {
    pub texture: Option<Arc<wgpu::Texture>>,
    pub texture_view: Option<Arc<wgpu::TextureView>>,
    pub dims: [u32; 2],
}

impl TextureSet {
    pub fn new(
        texture: Option<Arc<wgpu::Texture>>,
        texture_view: Option<Arc<wgpu::TextureView>>,
        dims: [u32; 2],
    ) -> Self {
        Self {
            texture,
            texture_view,
            dims,
        }
    }

    pub fn set(
        &mut self,
        texture: Option<Arc<wgpu::Texture>>,
        texture_view: Option<Arc<wgpu::TextureView>>,
        dims: [u32; 2],
    ) {
        self.texture = texture;
        self.texture_view = texture_view;
        self.dims = dims;
    }

    pub fn set_view(&mut self, view: Arc<wgpu::TextureView>, dims: [u32; 2]) {
        self.texture = None;
        self.texture_view = Some(view);
        self.dims = dims;
    }

    pub fn clear(&mut self) {
        self.texture = None;
        self.texture_view = None;
        self.dims = [0, 0];
    }

    pub fn is_valid(&self) -> bool {
        self.texture_view.is_some() && (self.dims[0] > 0 && self.dims[1] > 0)
    }

    pub fn view_and_dims(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        if let Some(ref view) = self.texture_view
            && (self.dims[0] > 0 && self.dims[1] > 0)
        {
            Some((view.clone(), self.dims))
        } else {
            None
        }
    }
}

#[derive(Default, Debug)]
pub struct TextureVault {
    pub srcimg: TextureSet,
    pub effect: TextureSet,
    pub cropped: TextureSet,
}

impl TextureVault {
    #[tracing::instrument(skip(self))]
    pub fn set_srcimg(
        &mut self,
        srcimg_texture: Arc<wgpu::Texture>,
        srcimg_texture_view: Arc<wgpu::TextureView>,
        dims: [u32; 2],
    ) {
        self.srcimg
            .set(Some(srcimg_texture), Some(srcimg_texture_view), dims);
    }

    #[tracing::instrument(skip(self))]
    pub fn set_cropped(
        &mut self,
        texture: Arc<wgpu::Texture>,
        texture_view: Arc<wgpu::TextureView>,
    ) {
        let dims = [texture.width(), texture.height()];
        self.cropped.set(Some(texture), Some(texture_view), dims);
    }

    #[tracing::instrument(skip(self))]
    pub fn set_effect(
        &mut self,
        texture: Option<Arc<wgpu::Texture>>,
        texture_view: Arc<wgpu::TextureView>,
        dims: [u32; 2],
    ) {
        self.effect.set(texture, Some(texture_view), dims);
    }

    #[tracing::instrument(skip(self))]
    pub fn set_effect_view(&mut self, view: Arc<wgpu::TextureView>, dims: [u32; 2]) {
        self.effect.set_view(view, dims);
    }

    pub fn clear_cropped(&mut self) {
        self.cropped.clear();
    }

    pub fn clear_effects(&mut self) {
        self.effect.clear();
    }
}
