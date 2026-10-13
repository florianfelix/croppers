use crossbeam_channel::{Receiver, Sender};
use std::{path::PathBuf, sync::Arc};
#[allow(unused)]
use tracing::{error, info, trace};
use winit::event_loop::EventLoopProxy;

use crate::{
    Message,
    app_state::{
        effect_stack::EffectStack,
        files::{
            ImageSource,
            image_source::{ImgError, ImgResult, LoadedImage},
        },
        textures::TextureVault,
        ui_state::UiState,
    },
    compute::{
        ComputeRequest, ComputeResult, EffectsJob,
        crop_renderer::{CropJob, CropResult},
        effect_pipeline_process::EffectsResult,
    },
    utils::{CroppersConfig, DimHelper},
};

pub mod effect_stack;
pub mod files;
pub mod textures;
pub mod ui_draw;
pub mod ui_state;
pub mod widgets;

#[derive(Clone)]
pub struct GpuHandles {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub dims_max: u32,
}

pub struct AppState {
    pub config: CroppersConfig,
    pub gpu: Option<GpuHandles>,
    pub source: ImgResult<ImageSource>,
    pub source_dirty: bool,
    pub folder: Option<PathBuf>,
    pub folder_cache: Option<Vec<PathBuf>>,
    pub texture_vault: TextureVault,
    pub effect_stack: EffectStack,
    pub ui_state: UiState,
    pub needs_compute: bool,
    pub is_computing: bool,
    pub is_startup: bool,
    pub source_just_saved: bool,
    pub tx_job: Sender<ComputeRequest>,
    pub rx_job: Receiver<ComputeRequest>,
    pub proxy: Option<EventLoopProxy<Message>>,
    pub load_id: u64,
    pub target_path: Option<PathBuf>,
}

impl AppState {
    #[tracing::instrument(skip(config))]
    pub fn new(config: CroppersConfig) -> Self {
        let (tx_job, rx_job) = crossbeam_channel::unbounded::<ComputeRequest>();
        let mut app_state = Self {
            config,
            gpu: None,
            source: ImgResult::Err(ImgError::NoImage),
            source_dirty: false,
            folder: None,
            folder_cache: None,
            texture_vault: TextureVault::default(),
            effect_stack: EffectStack::default(),
            ui_state: UiState::default(),
            needs_compute: true,
            is_computing: false,
            is_startup: true,
            source_just_saved: false,
            tx_job,
            rx_job,
            proxy: None,
            load_id: 0,
            target_path: None,
        };
        if app_state.config.is_new_config {
            app_state.ui_state.show_settings = true;
        }

        app_state
    }

    pub fn apply_startup_config(&mut self) {
        let startup = &self.config.startup;
        self.ui_state.crop.enabled = startup.default_crop_setting.enabled;
        self.ui_state.crop.mode = startup.default_crop_setting.default_crop_mode;
        self.ui_state.crop.crop_width = startup.default_crop_setting.fixed_crop_width;
        self.ui_state.crop.crop_height = startup.default_crop_setting.fixed_crop_height;

        self.ui_state.show_panel = self.config.startup.show_panel;

        self.effect_stack.enabled = self.config.startup.enable_effects;
        self.is_startup = false;
    }

    #[tracing::instrument(skip(self))]
    pub fn update_before_reddraw(&mut self) -> bool {
        let mut request_redraw = false;

        if self.source_dirty {
            self.ui_state.crop.request_reset();
            if self.source.is_ok() {
                // reset effect stack compute
                self.is_computing = false;
                self.needs_compute = true;
                self.effect_stack.needs_rebuild = true;

                // Apply config on image load
                self.effect_stack.enabled = self.config.on_image_load.effects_enabled;
                self.ui_state.crop.enabled = self.config.on_image_load.crop_enabled;
                if self.config.on_image_load.empty_effects {
                    self.effect_stack.effects.clear();
                }

                if self.source_just_saved {
                    self.effect_stack.enabled = false;
                    self.ui_state.crop.enabled = false;
                    self.source_just_saved = false;
                }

                self.ui_state.reset_view();
            } else {
                self.is_computing = false;
                self.needs_compute = false;
            }
            self.texture_vault.clear_cropped();
            self.source_dirty = false;
        }

        if self.is_computing {
            request_redraw = true;
        }
        if (self.needs_compute || self.effect_stack.needs_rebuild) && !self.is_computing {
            let res = match self.texture_vault.cropped.texture_view.is_some() {
                true => {
                    info!("Sending cropped view to effects");
                    self.texture_vault.get_cropped_view()
                }
                false => self.texture_vault.get_srcimg_view(),
            };

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
                let _ = proxy.send_event(Message::Empty);
            }
        }
        if self.is_startup {
            self.apply_startup_config();
        }
        request_redraw
    }

    #[tracing::instrument(skip(self, compute_result))]
    pub fn handle_compute_result(&mut self, compute_result: ComputeResult) {
        match compute_result {
            ComputeResult::FinishedLoadImage { id, result } => {
                self.handle_image_loaded(id, result);
            }
            ComputeResult::FinishedEffects(EffectsResult::Finished {
                texture,
                view,
                dims,
            }) => {
                info!("Received Effects {:?}", dims);
                self.texture_vault.set_effect(texture, view, dims);
                self.is_computing = false;
                self.effect_stack.needs_rebuild = false;
            }
            ComputeResult::FinishedEffects(EffectsResult::Empty(e)) => {
                error!("Effect compute error {e}")
            }
            ComputeResult::FinishedCrop(CropResult::Finished { texture, view }) => {
                info!("Received Cropped {:?}", [texture.width(), texture.height()]);
                self.texture_vault.set_cropped(texture, view);
                self.is_computing = false;
                self.needs_compute = true;
                self.effect_stack.needs_rebuild = true;
            }
            ComputeResult::FinishedCrop(CropResult::Empty(e)) => self
                .ui_state
                .notifications
                .push(&format!("Crop failed: {e}"), true),
        }
    }

    #[tracing::instrument(skip(self, result))]
    pub fn handle_image_loaded(&mut self, id: u64, result: ImgResult<LoadedImage>) {
        if id != self.load_id {
            info!(
                "Discarding outdated image load request {} (current: {})",
                id, self.load_id
            );
            return;
        }

        self.target_path = None;

        match result {
            Ok(loaded) => {
                info!("Setting new Source Image {:?}", loaded.source.path);
                self.source = Ok(loaded.source);
                self.texture_vault
                    .set_srcimg(loaded.texture, loaded.texture_view, loaded.dims);
                self.texture_vault.effect = self.texture_vault.srcimg.clone();
                self.texture_vault.clear_cropped();
                self.source_dirty = true;
                self.needs_compute = true;
                self.effect_stack.needs_rebuild = true;
            }
            Err(e) => {
                let msg = format!("Failed to load image {e}");
                error!("{msg}");
                self.ui_state.notifications.push(&msg, true);
                self.source = Err(e);
                self.texture_vault.clear_effects();
                self.texture_vault.clear_cropped();
                if let Some(ref gpu) = self.gpu {
                    let (texture, view, dims) =
                        crate::render_state::RenderState::create_dummy_texture(
                            gpu.device.clone(),
                            gpu.queue.clone(),
                        );
                    self.texture_vault.set_srcimg(texture, view, dims);
                }
                self.source_dirty = true;
                self.needs_compute = false;
            }
        }
    }

    pub fn request_redraw_wake(&mut self) {
        if let Some(ref proxy) = self.proxy {
            let _ = proxy.send_event(Message::Empty);
        }
    }

    pub fn crop_toggle(&mut self) {
        self.ui_state.crop.enabled = !self.ui_state.crop.enabled;
    }
    pub fn set_crop_enabled(&mut self, enabled: bool) {
        self.ui_state.crop.enabled = enabled;
    }

    pub fn effects_toggle(&mut self) {
        self.effect_stack.enabled = !self.effect_stack.enabled;
        self.effect_stack.needs_rebuild = true;
        self.needs_compute = true;
    }

    pub fn send_crop_job(&mut self) {
        if !self.ui_state.crop.enabled {
            return;
        }
        info!("Sending Crop Job");
        let imgdim = self.texture_vault.srcimg.dims;
        let view = self
            .texture_vault
            .srcimg
            .texture_view
            .as_ref()
            .unwrap()
            .clone();
        let target_dimensions = self.ui_state.crop.target_dimensions(imgdim);
        let job = CropJob {
            src_view: view,
            src_dimensions: imgdim,
            target_dimensions,
            uv_rect: self.ui_state.crop.target_uv_rect(),
        };
        let _ = self.tx_job.send(ComputeRequest::CropJob(job));
        self.ui_state.crop.enabled = false;
    }
}
