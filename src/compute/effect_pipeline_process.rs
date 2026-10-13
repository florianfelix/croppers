use crate::compute::{
    EffectsJob,
    effect_pipeline::{GpuEffectPass, GpuEffectPipeline},
};
use std::{collections::HashMap, sync::Arc};
use tracing::info;
use wgpu::Device;

#[derive(Debug, Clone)]
pub enum EffectsResult {
    Empty(String),
    Finished {
        texture: Option<Arc<wgpu::Texture>>,
        view: Arc<wgpu::TextureView>,
        dims: [u32; 2],
    },
}

pub struct EffectProcessor {
    device: Arc<Device>,
    queue: Arc<wgpu::Queue>,
    shader_modules: HashMap<String, wgpu::ShaderModule>,
    compute_pipeline: Option<GpuEffectPipeline>,
}
impl EffectProcessor {
    pub fn new(device: Arc<Device>, queue: Arc<wgpu::Queue>) -> Self {
        let mut shader_modules = HashMap::new();
        shader_modules.insert(
            "vignette".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/vignette_comp.wgsl")),
        );
        shader_modules.insert(
            "brightness_contrast".to_string(),
            device.create_shader_module(wgpu::include_wgsl!(
                "../../shaders/brightness_contrast_comp.wgsl"
            )),
        );
        shader_modules.insert(
            "cutoff".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/cutoff_comp.wgsl")),
        );
        shader_modules.insert(
            "flip".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/flip_comp.wgsl")),
        );
        shader_modules.insert(
            "blur".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/blur_comp.wgsl")),
        );
        shader_modules.insert(
            "bw".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/bw_comp.wgsl")),
        );
        shader_modules.insert(
            "hsv".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/hsv_comp.wgsl")),
        );
        shader_modules.insert(
            "gamma".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/gamma_comp.wgsl")),
        );
        shader_modules.insert(
            "tiltshift".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/tiltshift_comp.wgsl")),
        );

        shader_modules.insert(
            "pixelate".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/pixelate_comp.wgsl")),
        );
        shader_modules.insert(
            "pixelate_linear".to_string(),
            device.create_shader_module(wgpu::include_wgsl!(
                "../../shaders/pixelate_linear_comp.wgsl"
            )),
        );
        shader_modules.insert(
            "dither".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/dither_comp.wgsl")),
        );
        shader_modules.insert(
            "crt".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/crt_comp.wgsl")),
        );
        shader_modules.insert(
            "ascii".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/ascii_comp.wgsl")),
        );
        shader_modules.insert(
            "vcr".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/vcr_comp.wgsl")),
        );
        shader_modules.insert(
            "glitch".to_string(),
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/glitch_comp.wgsl")),
        );

        Self {
            device,
            queue,
            shader_modules,
            compute_pipeline: None,
        }
    }
    pub fn process_effects(&mut self, mut job: EffectsJob) -> EffectsResult {
        if !job.effect_stack.enabled {
            for c in job.effect_stack.effects.iter_mut() {
                c.set_enabled(false);
            }
        }
        // info!("{:?}", job.effect_stack);
        if job.src_width == 0 || job.src_height == 0 {
            return EffectsResult::Empty("src_image width or height is zero".into());
        }
        // Rebuild or initialize compute pipeline if dimensions changed
        if self
            .compute_pipeline
            .as_ref()
            .is_none_or(|p| p.dimensions != (job.src_width, job.src_height))
        {
            info!("Dimensions Changed. New GpuEffectPipeline");
            self.compute_pipeline = Some(GpuEffectPipeline::new(
                self.device.clone(),
                job.src_width,
                job.src_height,
            ));
        }
        if job.effect_stack.needs_rebuild {
            info!("Rebuilding Effect Stack");
            if let Some(ref mut compute_pipeline) = self.compute_pipeline {
                compute_pipeline.rebuild_from_stack(
                    &self.device,
                    &job.effect_stack,
                    &self.shader_modules,
                );
            }
        }
        if let Some(ref mut pipe) = self.compute_pipeline {
            let mut encoder = self.device.create_command_encoder(&Default::default());

            for container in &job.effect_stack.effects {
                if !container.enabled() {
                    continue;
                }

                let matching_passes: Vec<&GpuEffectPass> = pipe
                    .passes
                    .iter()
                    .filter(|p| p.effect_id == container.id())
                    .collect();

                container.update_uniforms(&self.queue, &matching_passes);
            }

            let (output_view, maybe_idx) =
                pipe.execute(&self.device, &mut encoder, &job.input_view);

            let texture = maybe_idx.map(|idx| pipe.ping_pong_textures[idx].clone());

            self.queue.submit(std::iter::once(encoder.finish()));

            EffectsResult::Finished {
                texture,
                view: Arc::new(output_view),
                dims: [job.src_width, job.src_height],
            }
        } else {
            EffectsResult::Empty("No Compute Pipeline found".into())
        }
    }
}
