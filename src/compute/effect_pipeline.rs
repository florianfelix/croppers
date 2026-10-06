pub struct GpuEffectPass {
    pub effect_id: usize,
    pub pipeline: wgpu::ComputePipeline,
    pub uniform_buffer: wgpu::Buffer,
}

impl GpuEffectPass {
    pub fn update_uniforms<T: bytemuck::Pod>(&self, queue: &wgpu::Queue, data: &T) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*data]));
    }
}

use std::sync::Arc;

use wgpu::Extent3d;

use crate::app_state::effect_stack::EffectStack;

#[allow(dead_code)]
pub struct GpuEffectPipeline {
    pub device: Arc<wgpu::Device>,
    pub ping_pong_textures: [wgpu::Texture; 2],
    pub ping_pong_views: [wgpu::TextureView; 2],
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub passes: Vec<GpuEffectPass>,
    pub dimensions: (u32, u32),
}

impl GpuEffectPipeline {
    pub fn new(device: Arc<wgpu::Device>, width: u32, height: u32) -> Self {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // Shared bind group layout: (0: src_tex, 1: dst_tex, 2: uniforms)
        let bind_group_layout = Self::create_bind_group_layout(device.clone());

        let create_tex = |label: &str| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };

        let tex_a = create_tex("Ping Pong Texture A");
        let tex_b = create_tex("Ping Pong Texture B");
        let view_a = tex_a.create_view(&Default::default());
        let view_b = tex_b.create_view(&Default::default());

        Self {
            device,
            ping_pong_textures: [tex_a, tex_b],
            ping_pong_views: [view_a, view_b],
            bind_group_layout,
            passes: Vec::new(),
            dimensions: (width, height),
        }
    }
    pub fn create_bind_group_layout(device: Arc<wgpu::Device>) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Effect Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn rebuild_from_stack(
        &mut self,
        device: &wgpu::Device,
        stack: &EffectStack,
        shader_modules: &std::collections::HashMap<String, wgpu::ShaderModule>,
    ) {
        self.passes.clear();

        for container in &stack.effects {
            if !container.enabled() {
                continue;
            }

            container.build_passes(
                device,
                &self.bind_group_layout,
                shader_modules,
                &mut self.passes,
            );
        }
    }

    /// Dispatches all compute passes in order using ping-pong textures
    pub fn execute(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
    ) -> (wgpu::TextureView, Option<usize>) {
        if self.passes.is_empty() {
            return (input_view.clone(), None);
        }
        // info!("Executing");

        let workgroups_x = self.dimensions.0.div_ceil(16);
        let workgroups_y = self.dimensions.1.div_ceil(16);

        let mut current_src_view = input_view.clone();
        let mut dst_idx = 0;

        for pass in &self.passes {
            let write_view = &self.ping_pong_views[dst_idx];

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Ping-Pong Compute Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&current_src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(write_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: pass.uniform_buffer.as_entire_binding(),
                    },
                ],
            });

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Effect Compute Pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&pass.pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
            }

            // Point to output view for next pass input
            current_src_view = write_view.clone();

            // Toggle ping-pong index (0 -> 1 -> 0)
            dst_idx = 1 - dst_idx;
        }
        let last_written_idx = 1 - dst_idx;

        (current_src_view, Some(last_written_idx))
    }
}
