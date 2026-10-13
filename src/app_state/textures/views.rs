use std::sync::Arc;
#[allow(unused)]
use tracing::{info, trace, warn};

use crate::app_state::{AppState, textures::{TextureSet, TextureVault}};

impl TextureVault {
    #[tracing::instrument(skip(self))]
    pub fn get_view_auto(
        &self,
        is_crop_enabled: bool,
    ) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        if is_crop_enabled {
            trace!("GETTING SRC VIEW");
            self.get_srcimg_view()
        } else {
            trace!("GETTING EFFECT VIEW");
            self.get_effect_view()
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_cropped_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        let res = self.cropped.view_and_dims();
        if res.is_none() {
            warn!("no display view found");
        }
        res
    }

    #[tracing::instrument(skip(self))]
    pub fn get_effect_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        let res = self.effect.view_and_dims();
        if res.is_none() {
            warn!("no effect view found");
        }
        res
    }

    #[tracing::instrument(skip(self))]
    pub fn get_srcimg_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        let res = self.srcimg.view_and_dims();
        if res.is_none() {
            warn!("no display view found");
        }
        res
    }
}

impl AppState {
    #[inline]
    pub fn get_view_auto(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        self.texture_vault.get_view_auto(self.ui_state.crop.enabled)
    }

    #[inline]
    pub fn get_cropped_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        self.texture_vault.get_cropped_view()
    }

    #[inline]
    pub fn get_effect_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        self.texture_vault.get_effect_view()
    }

    #[inline]
    pub fn get_srcimg_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        self.texture_vault.get_srcimg_view()
    }

    #[inline]
    pub fn get_save_texture_set(&self) -> Option<TextureSet> {
        self.texture_vault.get_save_texture_set(self.effect_stack.is_active())
    }
}
