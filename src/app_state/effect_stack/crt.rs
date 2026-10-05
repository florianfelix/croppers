use std::collections::HashMap;

use crate::app_state::effect_stack::EffectContainerObject;

pub struct CrtContainer {
    pub id: usize,
    pub enabled: bool,
    pub settings: crate::app_state::effect_stack::CrtSettings, // Ihre bestehende Struct
}

impl EffectContainerObject for CrtContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Crt"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("crt_settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.settings.curvature, 0.0..=1.0).text("curvature"))
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.settings.scanline_count, 0.0..=1000.0)
                        .text("scanlines"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.settings.scanline_strength, 0.0..=1.0)
                        .text("scanline strength"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.settings.rgb_mask_strength, 0.0..=1.0)
                        .text("mask strength"),
                )
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "crt"
    }

    fn build_passes(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        shaders: &HashMap<String, wgpu::ShaderModule>,
        passes: &mut Vec<crate::compute::effect_pipeline::GpuEffectPass>,
    ) {
        use wgpu::util::DeviceExt;
        let uniforms: crate::compute::uniforms::CrtUniforms = self.settings.into();

        let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crt Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crt Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Crt"),
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
            let uniforms: crate::compute::uniforms::CrtUniforms = self.settings.into();
            queue.write_buffer(&pass.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        }
    }
}
