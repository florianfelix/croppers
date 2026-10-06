use std::{collections::HashMap, ops::DerefMut, sync::Arc};

use crate::app_state::effect_stack::EffectContainerObject;

#[derive(Clone)]
pub struct CrtContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: CrtUniforms, // Ihre bestehende Struct
}
impl CrtContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: CrtUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CrtUniforms {
    // --- 1. Block (16 Bytes) ---
    pub curvature: f32,         // Stärke der Röhrenwölbung (0.0 = flach)
    pub scanline_count: f32,    // Anzahl der horizontalen Zeilen (z. B. 240.0)
    pub scanline_strength: f32, // Dunkelheit der Zeilen (0.0 bis 1.0)
    pub rgb_mask_strength: f32, // Intensität des vertikalen RGB-Pixelgitters (0.0 bis 1.0)

    // --- 2. Block (16 Bytes) ---
    pub overscan: f32,
    pub _padding2: f32,
    pub _padding3: f32,
    pub _padding4: f32,
}
impl Default for CrtUniforms {
    fn default() -> Self {
        Self {
            curvature: 0.1,
            scanline_count: 120.0,
            scanline_strength: 0.5,
            rgb_mask_strength: 0.5,
            overscan: 1.0,
            _padding2: 0.0,
            _padding3: 0.0,
            _padding4: 0.0,
        }
    }
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
                .add(egui::Slider::new(&mut self.uniforms.curvature, 0.0..=1.0).text("curvature"))
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.scanline_count, 0.0..=1000.0)
                        .text("scanlines"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.scanline_strength, 0.0..=1.0)
                        .text("scanline strength"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.rgb_mask_strength, 0.0..=1.0)
                        .text("mask strength"),
                )
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.overscan, 0.0..=1.0).text("overscan"))
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

        let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crt Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
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
            queue.write_buffer(&pass.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
        }
    }
}
