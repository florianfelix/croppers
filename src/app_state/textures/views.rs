use std::sync::Arc;
use tracing::{info, trace};

use crate::{app_state::AppState, utils::DimHelper};

impl AppState {
    #[tracing::instrument(skip(self))]
    pub fn get_view_auto(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        if self.ui_state.crop.enabled {
            trace!("GETTING SRC VIEW");
            self.get_srcimg_view()
        } else {
            trace!("GETTING EFFECT VIEW");
            self.get_effect_view()
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_cropped_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        if let Some(ref cropped_view) = self.texture_vault.cropped_texture_view
            && self.texture_vault.cropped_dims.is_non_zero()
        {
            Some((cropped_view.clone(), self.texture_vault.cropped_dims))
        } else {
            info!("no display view found");
            None
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_effect_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        if let Some(ref effect_view) = self.texture_vault.effect_view
            && self.texture_vault.effect_dims.is_non_zero()
        {
            Some((effect_view.clone(), self.texture_vault.effect_dims))
        } else {
            info!("no effect view found");
            None
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_srcimg_view(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        if let Some(ref srcimg_view) = self.texture_vault.srcimg_texture_view
            && self.texture_vault.srcimg_dims.is_non_zero()
        {
            Some((srcimg_view.clone(), self.texture_vault.srcimg_dims))
        } else {
            info!("no display view found");
            None
        }
    }
}
