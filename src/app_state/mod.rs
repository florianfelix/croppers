use crossbeam_channel::{Receiver, Sender};
use std::{path::PathBuf, sync::Arc};
#[allow(unused)]
use tracing::{error, info, trace};
use winit::event_loop::EventLoopProxy;

use crate::{
    Message,
    app_state::{
        effect_stack::EffectStack, files::ImageSource, textures::TextureVault, ui_state::UiState,
    },
    compute::{
        ComputeRequest, ComputeResult, EffectsJob,
        crop_renderer::{CropJob, CropResult},
        effect_pipeline_process::EffectsResult,
    },
    render_state::RenderState,
    utils::{CroppersConfig, DimHelper},
};

pub mod effect_stack;
pub mod files;
pub mod textures;
pub mod ui_draw;
pub mod ui_state;
pub mod widgets;

pub struct AppState {
    pub config: CroppersConfig,
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

impl AppState {
    pub fn new(cli: crate::utils::Cli, config: CroppersConfig) -> Self {
        let (tx_job, rx_job) = crossbeam_channel::bounded::<ComputeRequest>(1);
        let mut app_state = Self {
            config,
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
        if app_state.config.is_new_config {
            app_state.ui_state.show_settings = true;
        }
        if let Some(path) = cli.path {
            app_state.try_open_file(&path);
        };

        app_state.apply_startup_config();
        app_state
    }

    pub fn apply_startup_config(&mut self) {
        self.ui_state.crop.enabled = self.config.startup_crop.enabled;
        self.ui_state.crop.mode = self.config.startup_crop.default_crop_mode;
        self.ui_state.crop.crop_width = self.config.startup_crop.fixed_crop_width;
        self.ui_state.crop.crop_height = self.config.startup_crop.fixed_crop_height;

        self.ui_state.show_panel = self.config.startup_ui.show_panel;

        self.effect_stack.enabled = self.config.startup_effects.enabled;
    }

    #[tracing::instrument(skip(self))]
    pub fn update_before_reddraw(&mut self) -> bool {
        let mut request_redraw = false;

        if !self.source.is_some() {
            if let Some((ref device, ref queue)) = self.gpu {
                let (texture, view, dims) =
                    RenderState::create_dummy_texture(device.clone(), queue.clone());
                self.texture_vault.set_srcimg(texture, view, dims);
            }
        }

        if self.source_dirty {
            if let Some(ref _source) = self.source {
                self.is_computing = false;
                self.needs_compute = true;
                self.effect_stack.needs_rebuild = true;
                self.effect_stack.enabled = self.config.on_image_load.effects_enabled;
                self.ui_state.crop.enabled = self.config.on_image_load.crop_enabled;

                if self.config.on_image_load.empty_effects {
                    self.effect_stack.effects.clear()
                }
                self.ui_state.reset_view();
            }
            self.texture_vault.clear_cropped();
            self.source_dirty = false;
            self.needs_compute = true;
        }

        if self.is_computing {
            request_redraw = true
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
        request_redraw
    }

    #[tracing::instrument(skip(self, compute_result))]
    pub fn handle_compute_result(&mut self, compute_result: ComputeResult) {
        match compute_result {
            ComputeResult::FinishedEffects(EffectsResult::Finished((view, dims))) => {
                info!("Received Effects {:?}", dims);
                self.texture_vault.set_effect_view(view, dims);
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
            uv_rect: self.ui_state.crop.uv_rect,
        };
        let _ = self.tx_job.send(ComputeRequest::CropJob(job));
        self.ui_state.crop.enabled = false;
    }
}
