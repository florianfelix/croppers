use crate::{
    compute::{
        EffectsJob,
        effect_pipeline::{GpuEffectPass, GpuEffectPipeline},
    },
    utils::DimHelper,
};
use anyhow::Result;
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
    pub last_processed: Option<(usize, [u32; 2])>,
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
        Self {
            device,
            queue,
            shader_modules,
            compute_pipeline: None,
            last_processed: None,
        }
    }
    pub fn process_effects(&mut self, job: EffectsJob) -> EffectsResult {
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

            // 2. Das Frame-Update (in `process_effects`) schrumpft auf:
            for container in &job.effect_stack.effects {
                if !container.enabled() {
                    continue;
                }

                let matching_passes: Vec<&GpuEffectPass> = pipe
                    .passes
                    .iter()
                    .filter(|p| p.effect_id == container.id())
                    .collect();

                // Delegiert das Schreiben des Buffers direkt an das Trait-Objekt!
                container.update_uniforms(&self.queue, &matching_passes);
            }

            let (output_view, maybe_idx) =
                pipe.execute(&self.device, &mut encoder, &job.input_view);

            if let Some(idx) = maybe_idx {
                self.last_processed = Some((idx, [job.src_width, job.src_height]));
            }
            // Submit to GPU queue
            self.queue.submit(std::iter::once(encoder.finish()));

            EffectsResult::Finished((Arc::new(output_view), [job.src_width, job.src_height]))
        } else {
            EffectsResult::Empty("No Compute Pipeline found".into())
        }
    }
}

use std::path::Path;

impl EffectProcessor {
    pub fn try_save(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some((idx, dims)) = self.last_processed {
            if let Some(pipe) = &self.compute_pipeline {
                let texture = &pipe.ping_pong_textures[idx];
                self.save_texture_to_file(texture, dims.width(), dims.height(), "output.png")?;
            }
        }
        Ok(())
    }

    /// Reads a texture back to the CPU and writes it to disk at `path`.
    pub fn save_texture_to_file<P: AsRef<Path>>(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        path: P,
    ) -> Result<()> {
        let bytes_per_pixel = 4u32; // Rgba8Unorm
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT; // 256
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) & !(align - 1);
        let buffer_size = (padded_bytes_per_row * height) as u64;

        // 1. Create staging buffer
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Download Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 2. Encode texture -> buffer copy
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Texture Download Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // 3. Map buffer asynchronously and wait
        let slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        // Block worker thread until GPU copy finishes
        self.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()??;

        // 4. Strip row padding into contiguous RGBA buffer
        let mapped_data = slice.get_mapped_range()?;
        let mut unpadded_pixels = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);

        for row in 0..height {
            let start = (row * padded_bytes_per_row) as usize;
            let end = start + unpadded_bytes_per_row as usize;
            unpadded_pixels.extend_from_slice(&mapped_data[start..end]);
        }

        drop(mapped_data);
        staging_buffer.unmap();

        // 5. Save to disk via the image crate
        image::save_buffer(
            path,
            &unpadded_pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )?;

        Ok(())
    }
}
