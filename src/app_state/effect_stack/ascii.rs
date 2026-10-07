use crate::app_state::effect_stack::EffectContainerObject;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AsciiContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: AsciiUniforms,
}

impl AsciiContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: AsciiUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AsciiUniforms {
    pub scale: f32,
    pub charset: u32,
    pub inset: f32,
    pub mult: f32,
    pub glyph_selection: i32,
    pub scale_x: f32,
    pub rain_factor: f32,
    pub rain_length: f32,
}

impl Default for AsciiUniforms {
    fn default() -> Self {
        Self {
            scale: 20.0,
            charset: 0,
            inset: 0.1,
            mult: 0.0,
            glyph_selection: 0,
            scale_x: 1.0,
            rain_factor: 0.0,
            rain_length: 20.0,
        }
    }
}

impl EffectContainerObject for AsciiContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Ascii"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("Ascii_settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.scale, 0.0..=200.0).text("Scale"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.inset, 0.0..=0.45).text("inset"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.charset, 0..=2).text("atlas"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.mult, 0.0..=1.0).text("original bg"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.glyph_selection, 0..=3).text("glyph set"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.scale_x, 0.0..=2.0).text("scale x"))
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.rain_factor, 0.0..=1.0)
                        .text("rain factor"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.rain_length, 1.0..=200.0)
                        .text("rain length"),
                )
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "ascii"
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
            label: Some("Ascii Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ascii Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Ascii"),
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
