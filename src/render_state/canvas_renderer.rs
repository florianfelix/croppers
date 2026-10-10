use crate::utils::Vertex;
use std::sync::Arc;
use wgpu::{RenderPass, util::DeviceExt};
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CanvasUniforms {
    pub image_bounds: [f32; 4],
    pub bg_color: [f32; 4],
    pub shadow_color: [f32; 4],
    pub radius: f32,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,
    pub _padding3: f32,
}

pub struct CanvasRenderer {
    pub device: Arc<wgpu::Device>,
    pub window: Arc<Window>,
    pub queue: Arc<wgpu::Queue>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub canvas_uniforms: CanvasUniforms,
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
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas Uniform Buffer"),
            size: std::mem::size_of::<CanvasUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
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

        let canvas_uniforms = CanvasUniforms {
            image_bounds: [0.0, 0.0, 0.0, 0.0],
            bg_color: [0.08, 0.08, 0.09, 1.0],
            shadow_color: [0.04, 0.04, 0.04, 0.6],
            radius: 100.0,
            shadow_offset_x: 10.0,
            shadow_offset_y: 10.0,
            _padding3: 0.0,
        };
        Self {
            device,
            window,
            queue,
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            sampler,
            bind_group_layout,
            bind_group,
            canvas_uniforms,
        }
    }

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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render<'a>(
        &mut self,
        render_pass: &mut RenderPass<'a>,
        app_state: &crate::app_state::AppState,
        image_size: Option<(f32, f32)>,
    ) {
        let target_rect = &app_state.ui_state.canvas_rect();
        let zoom = app_state.ui_state.zoom;
        let pan = app_state.ui_state.pan;

        self.canvas_uniforms.bg_color = app_state.config.appearance.canvas_bg_color;
        self.canvas_uniforms.shadow_color = app_state.config.appearance.canvas_shadow_color;
        self.canvas_uniforms.radius = app_state.config.appearance.canvas_radius;
        self.canvas_uniforms.shadow_offset_x = app_state.config.appearance.canvas_shadow_offset_x;
        self.canvas_uniforms.shadow_offset_y = app_state.config.appearance.canvas_shadow_offset_y;

        if let Some(rect) = target_rect {
            let scale = self.window.scale_factor() as f32;
            let viewport_w = rect.width() * scale;
            let viewport_h = rect.height() * scale;
            let viewport_x = rect.min.x * scale;
            let viewport_y = rect.min.y * scale;

            if let Some((img_width, img_height)) = image_size {
                let canvas_aspect = viewport_w / viewport_h;
                let image_aspect = img_width / img_height;

                let (mut scale_x, mut scale_y) = if image_aspect > canvas_aspect {
                    (1.0, canvas_aspect / image_aspect)
                } else {
                    (image_aspect / canvas_aspect, 1.0)
                };

                scale_x *= zoom;
                scale_y *= zoom;

                let half_w = scale_x * (viewport_w / 2.0);
                let half_h = scale_y * (viewport_h / 2.0);

                let center_x = viewport_x + (viewport_w / 2.0) + pan.x * scale;
                let center_y = viewport_y + (viewport_h / 2.0) + pan.y * scale;

                self.canvas_uniforms.image_bounds = [
                    center_x - half_w,
                    center_y - half_h,
                    center_x + half_w,
                    center_y + half_h,
                ];

                self.queue.write_buffer(
                    &self.uniform_buffer,
                    0,
                    bytemuck::bytes_of(&self.canvas_uniforms),
                );
            }

            render_pass.set_viewport(viewport_x, viewport_y, viewport_w, viewport_h, 0.0, 1.0);
        }
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1);
    }
}
