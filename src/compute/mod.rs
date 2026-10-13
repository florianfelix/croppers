use crate::{
    Message,
    app_state::{effect_stack::EffectStack, textures::TextureSet},
    compute::{
        crop_renderer::{CropJob, CropRender, CropResult},
        effect_pipeline_process::{EffectProcessor, EffectsResult},
        texture_saver::TextureSaver,
    },
};
use std::{path::PathBuf, sync::Arc};
use tracing::error;
use winit::event_loop::EventLoopProxy;

pub mod crop_renderer;
pub mod effect_pipeline;
pub mod effect_pipeline_process;
pub mod texture_saver;

pub fn spawn_worker(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    proxy: EventLoopProxy<Message>,
    rx_job: crossbeam_channel::Receiver<ComputeRequest>,
) {
    let _ = std::thread::spawn(|| compute_worker(device, queue, rx_job, proxy));
}

#[derive(Clone)]
pub enum ComputeRequest {
    EffectsJob(EffectsJob),
    CropJob(CropJob),
    SaveTexture {
        path_for_saving: PathBuf,
        texture_set: TextureSet,
    },
}
#[derive(Clone)]
pub struct EffectsJob {
    pub effect_stack: EffectStack,
    pub src_width: u32,
    pub src_height: u32,
    pub input_view: Arc<wgpu::TextureView>,
}

#[derive(Debug, Clone)]
pub enum ComputeResult {
    FinishedEffects(EffectsResult),
    FinishedCrop(CropResult),
}

#[tracing::instrument(skip(device, queue, rx_job, proxy))]
pub fn compute_worker(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    rx_job: crossbeam_channel::Receiver<ComputeRequest>,
    proxy: EventLoopProxy<Message>,
) {
    let mut effect_processor = EffectProcessor::new(device.clone(), queue.clone());
    let mut crop_renderer = CropRender::new(device.clone(), queue.clone());
    let texture_saver = TextureSaver::new(device.clone(), queue.clone());

    while let Ok(request) = rx_job.recv() {
        match request {
            ComputeRequest::EffectsJob(job) => {
                let msg = Message::ComputeResult(ComputeResult::FinishedEffects(
                    effect_processor.process_effects(job),
                ));
                let _ = proxy.send_event(msg);
            }
            ComputeRequest::CropJob(job) => {
                let msg =
                    Message::ComputeResult(ComputeResult::FinishedCrop(crop_renderer.process(job)));
                let _ = proxy.send_event(msg);
            }

            ComputeRequest::SaveTexture {
                path_for_saving,
                texture_set,
            } => match texture_saver.save(&texture_set, &path_for_saving) {
                Ok(()) => {
                    let _ = proxy.send_event(Message::FileSaved(path_for_saving));
                }
                Err(e) => {
                    error!("Failed to Save file {e}");
                    let _ = proxy.send_event(Message::FileSaveError(e.to_string()));
                }
            },
        }
    }
}
