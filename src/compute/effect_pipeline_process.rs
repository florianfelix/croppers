use crate::{
    app_state::effect_stack::Effect,
    compute::{
        EffectsJob,
        effect_pipeline::{GpuEffectPass, GpuEffectPipeline},
        uniforms::{
            BlurUniforms, BwUniforms, GammaUniforms, HsvUniforms, PixelateTiltUniforms,
            TiltShiftUniforms, VignetteUniforms,
        },
    },
};
use std::{collections::HashMap, sync::Arc};
use tracing::info;
use wgpu::Device;

#[derive(Debug, Clone)]
pub enum EffectsResult {
    Empty(String),
    Finished((Arc<wgpu::TextureView>, [u32; 2])),
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
        Self {
            device,
            queue,
            shader_modules,
            compute_pipeline: None,
        }
    }
    pub fn process_effects(&mut self, job: EffectsJob) -> EffectsResult {
        info!("{:?}", job.effect_stack);
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
                if !container.enabled {
                    continue;
                }
                // Find ALL passes matching this effect ID (Blur will match 2 passes)
                let matching_passes: Vec<&GpuEffectPass> = pipe
                    .passes
                    .iter()
                    .filter(|p| p.effect_id == container.id)
                    .collect();

                match &container.effect {
                    Effect::PixelateTilt(settings) => {
                        if let Some(pass) = matching_passes.first() {
                            let uniforms: PixelateTiltUniforms = (*settings).into();
                            self.queue.write_buffer(
                                &pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                    }
                    Effect::Gamma(settings) => {
                        if let Some(pass) = matching_passes.first() {
                            let uniforms: GammaUniforms = (*settings).into();
                            self.queue.write_buffer(
                                &pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                    }
                    Effect::Hsv(settings) => {
                        if let Some(pass) = matching_passes.first() {
                            let uniforms: HsvUniforms = (*settings).into();
                            self.queue.write_buffer(
                                &pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                    }
                    Effect::Bw(settings) => {
                        if let Some(pass) = matching_passes.first() {
                            let uniforms: BwUniforms = (*settings).into();
                            self.queue.write_buffer(
                                &pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                    }
                    Effect::Vignette(settings) => {
                        if let Some(pass) = matching_passes.first() {
                            let uniforms: VignetteUniforms = (*settings).into();
                            self.queue.write_buffer(
                                &pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                            // pass.update_uniforms(&self.queue, &uniforms);
                        }
                    }
                    Effect::Blur(settings) => {
                        // Pass 0: Horizontal Pass [radius (f32), 1 (i32), 0 (i32), 0.0 (f32)]
                        if let Some(h_pass) = matching_passes.first() {
                            let uniforms = BlurUniforms {
                                radius: settings.radius,
                                dir_x: 1,
                                dir_y: 0,
                                _padding: 0.0,
                            };
                            self.queue.write_buffer(
                                &h_pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                        // Pass 1: Vertical Pass [radius (f32), 0 (i32), 1 (i32), 0.0 (f32)]
                        if let Some(v_pass) = matching_passes.get(1) {
                            let uniforms = BlurUniforms {
                                radius: settings.radius,
                                dir_x: 0,
                                dir_y: 1,
                                _padding: 0.0,
                            };
                            self.queue.write_buffer(
                                &v_pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                    }
                    Effect::TiltShift(settings) => {
                        // Pass 0: Horizontal Pass [radius (f32), 1 (i32), 0 (i32), 0.0 (f32)]
                        if let Some(h_pass) = matching_passes.first() {
                            // let uniforms = BlurUniforms {
                            //     radius: settings.radius,
                            //     dir_x: 1,
                            //     dir_y: 0,
                            //     _padding: 0.0,
                            // };
                            let uniforms = TiltShiftUniforms::horizontal(settings);
                            self.queue.write_buffer(
                                &h_pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                        // Pass 1: Vertical Pass [radius (f32), 0 (i32), 1 (i32), 0.0 (f32)]
                        if let Some(v_pass) = matching_passes.get(1) {
                            // let uniforms = BlurUniforms {
                            //     radius: settings.radius,
                            //     dir_x: 0,
                            //     dir_y: 1,
                            //     _padding: 0.0,
                            // };
                            let uniforms = TiltShiftUniforms::vertical(settings);
                            self.queue.write_buffer(
                                &v_pass.uniform_buffer,
                                0,
                                bytemuck::bytes_of(&uniforms),
                            );
                        }
                    }
                }
            }

            // Execute compute dispatches on worker thread
            let output_view = pipe.execute(&self.device, &mut encoder, &job.input_view);

            // Submit to GPU queue
            self.queue.submit(std::iter::once(encoder.finish()));

            EffectsResult::Finished((Arc::new(output_view), [job.src_width, job.src_height]))
        } else {
            EffectsResult::Empty("No Compute Pipeline found".into())
        }
    }
}

//
// pub fn process_effects(
//     device: Arc<Device>,
//     queue: Arc<wgpu::Queue>,
//     window: Arc<Window>,
//     job: EffectsJob,
//     shader_modules: &HashMap<String, wgpu::ShaderModule>,
//     compute_pipeline: &mut Option<GpuEffectPipeline>,
//     tx_result: crossbeam_channel::Sender<ComputeResult>,
// ) {
//     info!("{:?}", job.effect_stack);
//     if job.src_width == 0 || job.src_height == 0 {
//         return;
//     }
//
//     // Rebuild or initialize compute pipeline if dimensions changed
//     if compute_pipeline
//         .as_ref()
//         .is_none_or(|p| p.dimensions != (job.src_width, job.src_height))
//     {
//         *compute_pipeline = Some(GpuEffectPipeline::new(
//             device.clone(),
//             job.src_width,
//             job.src_height,
//         ));
//     }
//
//     if job.effect_stack.needs_rebuild {
//         info!("Rebuilding Effect Stack");
//         if let Some(compute_pipeline) = compute_pipeline {
//             compute_pipeline.rebuild_from_stack(&device, &job.effect_stack, shader_modules);
//         }
//     }
//
//     if let Some(pipe) = compute_pipeline {
//         let mut encoder = device.create_command_encoder(&Default::default());
//
//         for container in &job.effect_stack.effects {
//             if !container.enabled {
//                 continue;
//             }
//             // Find ALL passes matching this effect ID (Blur will match 2 passes)
//             let matching_passes: Vec<&GpuEffectPass> = pipe
//                 .passes
//                 .iter()
//                 .filter(|p| p.effect_id == container.id)
//                 .collect();
//
//             match &container.effect {
//                 Effect::Vignette(settings) => {
//                     if let Some(pass) = matching_passes.first() {
//                         let pod_data = [
//                             settings.intensity,
//                             settings.radius,
//                             settings.smoothness,
//                             0.0f32,
//                         ];
//                         pass.update_uniforms(&queue, &pod_data);
//                     }
//                 }
//                 Effect::Blur(settings) => {
//                     // Pass 0: Horizontal Pass [radius (f32), 1 (i32), 0 (i32), 0.0 (f32)]
//                     if let Some(h_pass) = matching_passes.first() {
//                         let uniforms = BlurUniforms {
//                             radius: settings.radius,
//                             dir_x: 1,
//                             dir_y: 0,
//                             _padding: 0.0,
//                         };
//                         queue.write_buffer(
//                             &h_pass.uniform_buffer,
//                             0,
//                             bytemuck::bytes_of(&uniforms),
//                         );
//                     }
//                     // Pass 1: Vertical Pass [radius (f32), 0 (i32), 1 (i32), 0.0 (f32)]
//                     if let Some(v_pass) = matching_passes.get(1) {
//                         let uniforms = BlurUniforms {
//                             radius: settings.radius,
//                             dir_x: 0,
//                             dir_y: 1,
//                             _padding: 0.0,
//                         };
//                         queue.write_buffer(
//                             &v_pass.uniform_buffer,
//                             0,
//                             bytemuck::bytes_of(&uniforms),
//                         );
//                     }
//                 }
//                 Effect::Crop(_) => {}
//             }
//         }
//
//         // Execute compute dispatches on worker thread
//         let output_view = pipe.execute(&device, &mut encoder, &job.input_view);
//
//         // Submit to GPU queue
//         queue.submit(std::iter::once(encoder.finish()));
//
//         // Return result containing output view handle
//         if tx_result
//             .send(ComputeResult::FinishedEffects(EffectsResult::Finished(
//                 Arc::new(output_view),
//             )))
//             .is_ok()
//         {
//             window.request_redraw();
//         }
//     }
// }
