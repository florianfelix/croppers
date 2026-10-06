use crate::{
    compute::{
        EffectsJob,
        effect_pipeline::{GpuEffectPass, GpuEffectPipeline},
    },
    utils::DimHelper,
};
use anyhow::{Context, Result};
use image::{DynamicImage, RgbaImage};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
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
            last_processed: None,
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

            if let Some(idx) = maybe_idx {
                self.last_processed = Some((idx, [job.src_width, job.src_height]));
            }

            self.queue.submit(std::iter::once(encoder.finish()));

            EffectsResult::Finished((Arc::new(output_view), [job.src_width, job.src_height]))
        } else {
            EffectsResult::Empty("No Compute Pipeline found".into())
        }
    }
}

impl EffectProcessor {
    pub fn try_save(&self, path: &PathBuf) -> Result<()> {
        let (idx, dims) = self.last_processed.ok_or_else(|| {
            anyhow::anyhow!("No processed texture available to save (stack was empty or never run)")
        })?;

        let pipe = self
            .compute_pipeline
            .as_ref()
            .context("No compute pipeline available")?;

        let texture = &pipe.ping_pong_textures[idx];
        self.save_texture_to_file(texture, dims.width(), dims.height(), path)
    }

    #[tracing::instrument(skip(self, texture, width, height))]
    pub fn save_texture_to_file(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        path: &PathBuf,
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
        self.device.poll(wgpu::PollType::wait_indefinitely())?;

        if rx.recv()?.is_ok() {
            let data = slice.get_mapped_range()?;

            let mut unpadded_data = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
            for chunk in data.chunks_exact(padded_bytes_per_row as usize) {
                unpadded_data.extend_from_slice(&chunk[..unpadded_bytes_per_row as usize]);
            }

            drop(data);
            staging_buffer.unmap();

            let rgba_img = RgbaImage::from_raw(width, height, unpadded_data)
                .context("Failed to construct image from raw buffer")?;

            let dyn_img = DynamicImage::ImageRgba8(rgba_img);

            let is_jpeg = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg"))
                .unwrap_or(false);

            if is_jpeg {
                dyn_img.to_rgb8().save(path)?;
            } else {
                dyn_img.save(path)?;
            }

            Ok(())
        } else {
            panic!("Staging Buffer Mapped incorrectly");
        }
    }
}
