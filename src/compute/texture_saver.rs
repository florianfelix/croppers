use anyhow::{Context, Result};
use image::{DynamicImage, RgbaImage};
use std::{fs::File, io::BufWriter, path::Path, sync::Arc};

use crate::app_state::textures::TextureSet;

pub struct TextureSaver {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl TextureSaver {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self { device, queue }
    }

    #[tracing::instrument(skip(self, texture_set))]
    pub fn save(&self, texture_set: &TextureSet, path: &Path) -> Result<()> {
        let texture = texture_set
            .texture
            .as_ref()
            .context("TextureSet has no texture to save")?;
        let width = texture_set.dims[0];
        let height = texture_set.dims[1];
        if width == 0 || height == 0 {
            anyhow::bail!("TextureSet dimensions are invalid: {}x{}", width, height);
        }

        self.save_texture_to_file(texture, width, height, path)
    }

    #[tracing::instrument(skip(self, texture, width, height))]
    pub fn save_texture_to_file(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        path: &Path,
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
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 90);
                encoder.encode_image(&dyn_img.to_rgb8())?;
            } else {
                dyn_img.save(path)?;
            }

            Ok(())
        } else {
            anyhow::bail!("Staging Buffer Mapped incorrectly");
        }
    }
}
