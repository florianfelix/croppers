use std::{path::PathBuf, sync::Arc};

use crossbeam_channel::{Receiver, Sender};
use tracing::{error, info};
use winit::event_loop::EventLoopProxy;

use crate::{
    Message,
    app_state::{
        effect_stack::EffectStack, files::ImageSource, texture_vault::TextureVault,
        ui_state::UiState,
    },
    compute::{
        ComputeRequest, ComputeResult, EffectsJob,
        crop_renderer::{CropJob, CropResult},
        effect_pipeline_process::EffectsResult,
    },
    utils::DimHelper,
};

mod widgets;

pub struct AppState {
    pub gpu: Option<(Arc<wgpu::Device>, Arc<wgpu::Queue>)>,
    pub source: Option<ImageSource>,
    pub source_dirty: bool,
    pub folder: Option<PathBuf>,
    pub texture_vault: TextureVault,
    pub effect_stack: EffectStack,
    pub ui_state: UiState,
    pub needs_compute: bool,
    pub is_computing: bool,
    pub tx_job: Sender<ComputeRequest>,
    pub rx_job: Receiver<ComputeRequest>,
    pub proxy: Option<EventLoopProxy<Message>>,
}

pub mod effect_stack;
pub mod files;
pub mod texture_vault;
pub mod ui_draw;
pub mod ui_state;

impl AppState {
    pub fn new(cli: crate::utils::Cli) -> Self {
        let (tx_job, rx_job) = crossbeam_channel::bounded::<ComputeRequest>(1);
        let mut app_state = Self {
            gpu: None,
            source: None,
            source_dirty: false,
            folder: None,
            texture_vault: TextureVault::default(),
            effect_stack: EffectStack::default(),
            ui_state: UiState::default(),
            needs_compute: true,
            is_computing: false,
            tx_job,
            rx_job,
            proxy: None,
        };
        if let Some(path) = cli.path {
            app_state.try_open_file(&path);
        };
        app_state
    }

    #[tracing::instrument(skip(self))]
    pub fn update_before_reddraw(&mut self) -> bool {
        let mut request_redraw = false;

        if self.source_dirty {
            if let Some(ref source) = self.source {
                self.is_computing = false;
                self.needs_compute = true;
                self.effect_stack.needs_rebuild = true;
            }
            self.texture_vault.clear_cropped();
            // self.texture_vault.clear_effects();
            // self.texture_vault.effect_view = self.texture_vault.srcimg_texture_view.clone();
            self.source_dirty = false;
        }

        if self.is_computing {
            request_redraw = true
        }
        if (self.needs_compute || self.effect_stack.needs_rebuild) && !self.is_computing {
            let res = match self.texture_vault.cropped_texture_view.is_some() {
                true => {
                    info!("Sending cropped view to effects");
                    self.get_cropped_view()
                }
                false => self.get_srcimg_view(),
            };

            // if let Some((view, dims)) = self.get_srcimg_view() {
            if let Some((view, dims)) = res {
                let compute_request = ComputeRequest::EffectsJob(EffectsJob {
                    effect_stack: self.effect_stack.clone(),
                    src_width: dims.width(),
                    src_height: dims.height(),
                    input_view: view,
                });
                if self.tx_job.try_send(compute_request).is_ok() {
                    self.is_computing = true;
                    self.needs_compute = false;
                    self.effect_stack.needs_rebuild = false;
                }
            }
            if let Some(ref proxy) = self.proxy {
                proxy.send_event(Message::Empty);
            }
        }
        request_redraw
    }

    #[tracing::instrument(skip(self, compute_result))]
    pub fn handle_compute_result(&mut self, compute_result: ComputeResult) {
        match compute_result {
            ComputeResult::FinishedEffects(EffectsResult::Finished((view, dims))) => {
                info!("Received Effects {:?}", dims);
                // if self.texture_vault.cropped_texture_view.is_some() {}
                self.texture_vault.effect_view = Some(view);
                self.texture_vault.effect_dims = dims;
                self.is_computing = false;
                self.effect_stack.needs_rebuild = false;
            }
            ComputeResult::FinishedCrop(CropResult::Finished { texture, view }) => {
                info!("Received Cropped {:?}", [texture.width(), texture.height()]);
                self.texture_vault.set_cropped(texture, view);
                self.is_computing = false;
                self.needs_compute = true;
                self.effect_stack.needs_rebuild = true;
            }
            _ => {}
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_view_auto(&self) -> Option<(Arc<wgpu::TextureView>, [u32; 2])> {
        if self.ui_state.crop.enabled {
            info!("GETTING SRC VIEW");
            self.get_srcimg_view()
        } else {
            info!("GETTING EFFECT VIEW");
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

    #[tracing::instrument(skip(self))]
    pub fn set_srcimg_texture_from_source(&mut self) {
        let Some(ref img) = self.source else { return };
        let Some((ref device, ref queue)) = self.gpu else {
            return;
        };

        info!("Creating srcimg textures. {:?}", img.path);

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
            // format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

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

        self.texture_vault
            .set_srcimg(Arc::new(texture), texture_view, dims);
        self.texture_vault.effect_view = self.texture_vault.srcimg_texture_view.clone();
        self.texture_vault.clear_cropped();
        // self.texture_vault.clear_effects();
        self.needs_compute = true;
    }
    pub fn request_redraw_wake(&mut self) {
        if let Some(ref proxy) = self.proxy {
            let _ = proxy.send_event(Message::Empty);
        }
    }

    pub fn send_crop_job(&mut self) {
        if !self.ui_state.crop.enabled {
            return;
        }
        info!("Sending Crop Job");
        let imgdim = self.texture_vault.srcimg_dims;
        let view = self
            .texture_vault
            .srcimg_texture_view
            .as_ref()
            .unwrap()
            .clone();
        let job = CropJob {
            src_view: view,
            src_dimensions: imgdim,
            target_dimensions: [
                self.ui_state.crop.crop_width,
                self.ui_state.crop.crop_height,
            ],
            uv_rect: self.ui_state.crop.uv_rect,
        };
        let _ = self.tx_job.send(ComputeRequest::CropJob(job));
        self.ui_state.crop.enabled = false;
    }
}
