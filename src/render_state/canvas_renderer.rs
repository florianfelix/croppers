use std::sync::Arc;

use crate::utils::Vertex;
use egui::Rect;
use wgpu::{RenderPass, util::DeviceExt};
use winit::window::Window;

pub struct CanvasRenderer {
    pub device: Arc<wgpu::Device>,
    pub window: Arc<Window>,
    pub queue: Arc<wgpu::Queue>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CanvasRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        window: Arc<Window>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        initial_view: &wgpu::TextureView,
    ) -> Self {
        let vertices = [
            Vertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Bind Group Layout"),
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
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(initial_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/canvas_shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Canvas Render Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            ..Default::default()
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Canvas Render Pipeline"),
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
                    format,
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
            window,
            queue,
            render_pipeline,
            vertex_buffer,
            sampler,
            bind_group_layout,
            bind_group,
        }
    }

    /// Rebinds the texture view outputting from compute passes to the render pipeline.
    pub fn update_texture_binding(&mut self, processed_view: &Arc<wgpu::TextureView>) {
        self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Processed Image Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(processed_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    /// Scales the quad mesh aspect ratio to fit within the target UI viewport, applying zoom and pan.
    pub fn update_quad_aspect(
        &mut self,
        img_width: f32,
        img_height: f32,
        viewport_w: f32,
        viewport_h: f32,
        zoom: f32,
        pan: egui::Vec2,
    ) {
        let canvas_aspect = viewport_w / viewport_h;
        let image_aspect = img_width / img_height;

        let (mut scale_x, mut scale_y) = if image_aspect > canvas_aspect {
            (1.0, canvas_aspect / image_aspect)
        } else {
            (image_aspect / canvas_aspect, 1.0)
        };

        // Apply zoom
        scale_x *= zoom;
        scale_y *= zoom;

        // Apply pan translation relative to normalized device coordinates (-1.0 to 1.0)
        // egui pan is in pixels. Convert pan pixels to NDC offset:
        let offset_x = (pan.x / viewport_w) * 2.0;
        let offset_y = -(pan.y / viewport_h) * 2.0; // NDC Y is up, Egui Y is down

        let vertices = [
            Vertex {
                position: [-scale_x + offset_x, -scale_y + offset_y],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [scale_x + offset_x, -scale_y + offset_y],
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [-scale_x + offset_x, scale_y + offset_y],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [scale_x + offset_x, scale_y + offset_y],
                uv: [1.0, 0.0],
            },
        ];

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render<'a>(
        &mut self,
        render_pass: &mut RenderPass<'a>,
        target_rect: &Option<Rect>,
        image_size: Option<(f32, f32)>,
        zoom: f32,
        pan: egui::Vec2,
    ) {
        if let Some(rect) = target_rect {
            let scale = self.window.scale_factor() as f32;
            let viewport_w = rect.width() * scale;
            let viewport_h = rect.height() * scale;

            if let Some((img_width, img_height)) = image_size {
                self.update_quad_aspect(img_width, img_height, viewport_w, viewport_h, zoom, pan);
            }

            let x = rect.min.x * scale;
            let y = rect.min.y * scale;

            render_pass.set_viewport(x, y, viewport_w, viewport_h, 0.0, 1.0);
        }
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1);
    }
}
