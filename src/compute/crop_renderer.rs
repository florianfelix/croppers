#![allow(unused)]
use std::sync::Arc;

use tracing::info;
use wgpu::util::DeviceExt;

use crate::utils::Vertex;

#[derive(Debug, Clone)]
pub struct CropJob {
    pub src_view: Arc<wgpu::TextureView>,
    pub src_dimensions: [u32; 2],
    pub target_dimensions: [u32; 2],
    pub uv_rect: egui::Rect,
}
#[derive(Debug, Clone)]
pub enum CropResult {
    Empty(String),
    Finished {
        texture: Arc<wgpu::Texture>,
        view: Arc<wgpu::TextureView>,
    },
}

pub struct CropRender {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub shader: wgpu::ShaderModule,
    target_texture: Option<Arc<wgpu::Texture>>,
    target_texture_view: Option<Arc<wgpu::TextureView>>,
}

impl CropRender {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/crop.wgsl"));

        let bind_group_layout = Self::bind_group_layout(device.clone());

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crop Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            ..Default::default()
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Crop Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::layout())],
                compilation_options: Default::default(),
            },
            multiview_mask: None,
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
        });
        Self {
            device,
            queue,
            render_pipeline,
            // window,
            shader,
            target_texture: None,
            target_texture_view: None,
        }
    }
    pub fn clear(&mut self) {
        self.target_texture = None;
        self.target_texture_view = None;
    }

    pub fn create_target_texture(&mut self, target_dimensions: [u32; 2]) {
        let size = wgpu::Extent3d {
            width: target_dimensions[0],
            height: target_dimensions[1],
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.target_texture = Some(Arc::new(texture));
        self.target_texture_view = Some(Arc::new(texture_view));
    }

    pub fn process(&mut self, job: CropJob) -> CropResult {
        info!("CropJob {:#?}", job);

        self.create_target_texture(job.target_dimensions);

        let vertices = [
            Vertex {
                position: [-1.0, 1.0],
                uv: [job.uv_rect.min.x, job.uv_rect.min.y],
            }, // Top Left
            Vertex {
                position: [-1.0, -1.0],
                uv: [job.uv_rect.min.x, job.uv_rect.max.y],
            }, // Bottom Left
            Vertex {
                position: [1.0, 1.0],
                uv: [job.uv_rect.max.x, job.uv_rect.min.y],
            }, // Top Right
            Vertex {
                position: [1.0, -1.0],
                uv: [job.uv_rect.max.x, job.uv_rect.max.y],
            }, // Bottom Right
        ];
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Crop Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let bind_group = Self::bind_group(self.device.clone(), job.src_view);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Crop Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Crop Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.target_texture_view.as_ref().unwrap(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                ..Default::default()
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..4, 0..1); // Draw 4 vertices for TriangleStrip quad
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        if let (Some(ref target_texture), Some(ref target_texture_view)) = (
            self.target_texture.clone(),
            self.target_texture_view.clone(),
        ) {
            info!(
                "Finished Cropping {:?}",
                [target_texture.width(), target_texture.height()]
            );
            CropResult::Finished {
                texture: target_texture.clone(),
                view: target_texture_view.clone(),
            }
        } else {
            CropResult::Empty("No Crop texture found".into())
        }
    }

    fn bind_group_layout(device: Arc<wgpu::Device>) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Crop Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
    fn bind_group(
        device: Arc<wgpu::Device>,
        input_view: Arc<wgpu::TextureView>,
    ) -> wgpu::BindGroup {
        let bind_group_layout = Self::bind_group_layout(device.clone());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }
}
