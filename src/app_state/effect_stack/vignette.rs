use crate::app_state::effect_stack::EffectContainerObject;
use std::collections::HashMap;

#[derive(Clone)]
pub struct VignetteContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: VignetteUniforms,
}

impl VignetteContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: VignetteUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VignetteUniforms {
    pub intensity: f32,
    pub radius: f32,
    pub smoothness: f32,
    pub _padding: f32,
}

impl Default for VignetteUniforms {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            radius: 0.75,
            smoothness: 0.65,
            _padding: 0.0,
        }
    }
}

impl EffectContainerObject for VignetteContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Vignette"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("vignette_settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.intensity, 0.0..=1.0).text("Intensity"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.radius, 0.0..=2.0).text("Radius"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.smoothness, 0.0..=1.0).text("Smoothness"))
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "vignette"
    }

    fn build_passes(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        shaders: &HashMap<String, wgpu::ShaderModule>,
        passes: &mut Vec<crate::compute::effect_pipeline::GpuEffectPass>,
    ) {
        use wgpu::util::DeviceExt;
        let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vignette Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Vignette Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Vignette"),
            layout: Some(&pipeline_layout),
            module: &shaders[self.shader_key()],
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        passes.push(crate::compute::effect_pipeline::GpuEffectPass {
            effect_id: self.id,
            pipeline,
            uniform_buffer: buf,
        });
    }

    fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        matching_passes: &[&crate::compute::effect_pipeline::GpuEffectPass],
    ) {
        if let Some(pass) = matching_passes.first() {
            queue.write_buffer(&pass.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
        }
    }
}
