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
use wgpu::util::DeviceExt;

use crate::{
    app_state::effect_stack::{Effect, EffectStack},
    compute::uniforms::{BlurUniforms, BwUniforms, GammaUniforms, HsvUniforms, VignetteUniforms},
};

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
            if !container.enabled {
                // info!("{:} Disabled", container.name);
                continue;
            }
            // info!("{:} Enabled", container.name);

            // Create uniform buffer based on effect type
            let (uniform_buffer, shader_key) = match &container.effect {
                Effect::Gamma(settings) => {
                    let uniforms: GammaUniforms = (*settings).into();
                    let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gamma Uniform Buffer"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                    (buf, "gamma")
                }
                Effect::Hsv(settings) => {
                    let uniforms: HsvUniforms = (*settings).into();
                    let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Hsv Uniform Buffer"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                    (buf, "hsv")
                }
                Effect::Bw(settings) => {
                    let uniforms: BwUniforms = (*settings).into();
                    let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Bw Uniform Buffer"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                    (buf, "bw")
                }
                Effect::Vignette(settings) => {
                    let uniforms: VignetteUniforms = (*settings).into();
                    let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vignette Uniform Buffer"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                    (buf, "vignette")
                }
                Effect::Blur(settings) => {
                    // 1. Horizontal Pass Uniforms
                    // Cast integers directly via bytemuck or transmute bits:
                    let uniforms = BlurUniforms {
                        radius: settings.radius,
                        dir_x: 1,
                        dir_y: 0,
                        _padding: 0.0,
                    };

                    let h_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Blur Horizontal Uniform Buffer"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

                    let pipeline_layout =
                        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("Blur Pipeline Layout"),
                            bind_group_layouts: &[Some(&self.bind_group_layout)],
                            immediate_size: 0,
                        });

                    let compute_pipeline =
                        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                            label: Some("Compute Pipeline Blur"),
                            layout: Some(&pipeline_layout),
                            module: &shader_modules["blur"],
                            entry_point: Some("main"),
                            compilation_options: Default::default(),
                            cache: None,
                        });

                    // Add Horizontal Pass
                    self.passes.push(GpuEffectPass {
                        effect_id: container.id,
                        pipeline: compute_pipeline.clone(),
                        uniform_buffer: h_buf,
                    });

                    // 2. Vertical Pass Uniforms
                    let uniforms = BlurUniforms {
                        radius: settings.radius,
                        dir_x: 0,
                        dir_y: 1,
                        _padding: 0.0,
                    };
                    let v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Blur Vertical Uniform Buffer"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

                    // Add Vertical Pass
                    self.passes.push(GpuEffectPass {
                        effect_id: container.id,
                        pipeline: compute_pipeline,
                        uniform_buffer: v_buf,
                    });

                    continue;
                }
            };

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pass Layout"),
                bind_group_layouts: &[Some(&self.bind_group_layout)],
                immediate_size: 0, // ..Default::default()
            });

            let compute_pipeline =
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(&format!("Compute Pipeline {}", container.name)),
                    layout: Some(&pipeline_layout),
                    module: &shader_modules[shader_key],
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                });

            // Note: Bind groups are created/updated per ping-pong swap during execution
            self.passes.push(GpuEffectPass {
                effect_id: container.id,
                pipeline: compute_pipeline,
                uniform_buffer,
            });
        }
    }

    /// Dispatches all compute passes in order using ping-pong textures
    pub fn execute(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
    ) -> wgpu::TextureView {
        if self.passes.is_empty() {
            return input_view.clone();
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

        current_src_view
    }
}
