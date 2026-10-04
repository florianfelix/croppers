use crate::{
    Message,
    app_state::effect_stack::EffectStack,
    compute::{
        crop_renderer::{CropJob, CropRender, CropResult},
        effect_pipeline_process::{EffectProcessor, EffectsResult},
    },
};
use std::sync::Arc;
use winit::{event_loop::EventLoopProxy, window::Window};

pub mod crop_renderer;
pub mod effect_pipeline;
pub mod effect_pipeline_process;

pub fn spawn_worker(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    proxy: EventLoopProxy<Message>,
    rx_job: crossbeam_channel::Receiver<ComputeRequest>,
) {
    let _ = std::thread::spawn(|| compute_worker(device, queue, rx_job, proxy));
}

#[derive(Debug, Clone)]
pub enum ComputeRequest {
    EffectsJob(EffectsJob),
    CropJob(CropJob),
}
#[derive(Debug, Clone)]
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
    // window: Arc<Window>,
    rx_job: crossbeam_channel::Receiver<ComputeRequest>,
    proxy: EventLoopProxy<Message>,
) {
    let mut effect_processor = EffectProcessor::new(device.clone(), queue.clone());
    let mut crop_renderer = CropRender::new(device.clone(), queue.clone());

    while let Ok(request) = rx_job.recv() {
        match request {
            ComputeRequest::EffectsJob(job) => {
                let msg = Message::ComputeResult(ComputeResult::FinishedEffects(
                    effect_processor.process_effects(job),
                ));
                if proxy.send_event(msg).is_ok() {
                    // window.request_redraw();
                }
            }
            ComputeRequest::CropJob(job) => {
                let msg =
                    Message::ComputeResult(ComputeResult::FinishedCrop(crop_renderer.process(job)));
                if proxy.send_event(msg).is_ok() {
                    // window.request_redraw();
                }
            }
        }
    }
    // window.request_redraw();
}
