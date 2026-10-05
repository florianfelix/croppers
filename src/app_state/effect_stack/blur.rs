use std::collections::HashMap;

use crate::app_state::effect_stack::EffectContainerObject;

#[derive(Clone)]
pub struct BlurContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: BlurUniforms,
}
impl BlurContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: BlurUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // or bytemuck::Pod, bytemuck::Zeroable
pub struct BlurUniforms {
    pub radius: f32,
    pub dir_x: i32,
    pub dir_y: i32,
    pub _padding: f32,
}
impl Default for BlurUniforms {
    fn default() -> Self {
        Self {
            radius: 10.0,
            dir_x: 1,
            dir_y: 0,
            _padding: 0.0,
        }
    }
}

impl EffectContainerObject for BlurContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "The Blur"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("blur_settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.radius, 0.0..=200.0).text("Radius"))
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "blur"
    }

    fn build_passes(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        shaders: &HashMap<String, wgpu::ShaderModule>,
        passes: &mut Vec<crate::compute::effect_pipeline::GpuEffectPass>,
    ) {
        use wgpu::util::DeviceExt;
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Blur"),
            layout: Some(&pipeline_layout),
            module: &shaders[self.shader_key()],
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        // Pass 1: Horizontal Puffer erstellen und pushen
        let h_uniforms = BlurUniforms {
            radius: self.uniforms.radius,
            dir_x: 1,
            dir_y: 0,
            _padding: 0.0,
        };
        let h_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Blur Horizontal Buffer"),
            contents: bytemuck::bytes_of(&h_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        passes.push(crate::compute::effect_pipeline::GpuEffectPass {
            effect_id: self.id,
            pipeline: pipeline.clone(),
            uniform_buffer: h_buf,
        });

        // Pass 2: Vertical Puffer erstellen und pushen
        let v_uniforms = BlurUniforms {
            radius: self.uniforms.radius,
            dir_x: 0,
            dir_y: 1,
            _padding: 0.0,
        };
        let v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Blur Vertical Buffer"),
            contents: bytemuck::bytes_of(&v_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        passes.push(crate::compute::effect_pipeline::GpuEffectPass {
            effect_id: self.id,
            pipeline,
            uniform_buffer: v_buf,
        });
    }

    fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        matching_passes: &[&crate::compute::effect_pipeline::GpuEffectPass],
    ) {
        if let Some(h_pass) = matching_passes.first() {
            let h_uniforms = BlurUniforms {
                radius: self.uniforms.radius,
                dir_x: 1,
                dir_y: 0,
                _padding: 0.0,
            };
            queue.write_buffer(&h_pass.uniform_buffer, 0, bytemuck::bytes_of(&h_uniforms));
        }
        if let Some(v_pass) = matching_passes.get(1) {
            let v_uniforms = BlurUniforms {
                radius: self.uniforms.radius,
                dir_x: 0,
                dir_y: 1,
                _padding: 0.0,
            };
            queue.write_buffer(&v_pass.uniform_buffer, 0, bytemuck::bytes_of(&v_uniforms));
        }
    }
}
