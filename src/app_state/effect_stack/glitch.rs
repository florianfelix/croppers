use crate::app_state::effect_stack::EffectContainerObject;
use std::collections::HashMap;

#[derive(Clone)]
pub struct GlitchContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: GlitchUniforms,
}

impl GlitchContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: GlitchUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlitchUniforms {
    pub time: f32,
    pub line_dropouts: f32,
    pub block_displace: f32,
    pub color_burst: f32,
    pub burst_grid_size: f32,
    pub tearing_micro: f32,
    pub tearing_macro: f32,
    pub tearing_blocks: f32,
}

impl Default for GlitchUniforms {
    fn default() -> Self {
        Self {
            time: 20.0,
            line_dropouts: 0.1,
            block_displace: 0.15,
            color_burst: 0.1,
            burst_grid_size: 8.0,
            tearing_micro: 1.0,
            tearing_macro: 0.5,
            tearing_blocks: 32.0,
        }
    }
}

impl EffectContainerObject for GlitchContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Glitch"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("Glitch_settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.time, 0.0..=200.0).text("time"))
                .changed();
            ui.separator();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.line_dropouts, 0.0..=1.0)
                        .text("Signal dropouts"),
                )
                .changed();
            ui.separator();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.block_displace, 0.0..=1.0)
                        .text("Sync Tearing"),
                )
                .changed();

            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.tearing_micro, 0.0..=20.0)
                        .text("micro tearing"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.tearing_macro, 0.0..=20.0)
                        .text("macro tearing"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.tearing_blocks, 0.0..=100.0)
                        .step_by(1.0)
                        .text("tearing blocks"),
                )
                .changed();
            ui.separator();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.color_burst, 0.0..=1.0)
                        .text("Chroma Frying"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.burst_grid_size, 0.0..=100.0)
                        .text("Grid size"),
                )
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "glitch"
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
            label: Some("Glitch Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Glitch Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Glitch"),
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
