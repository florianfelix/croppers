use serde::{Deserialize, Serialize};

use crate::app_state::effect_stack::presets::EffectConfig;

use super::EffectContainerObject;
use std::collections::HashMap;

#[derive(Clone)]
pub struct BwContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: BwUniforms,
}

impl BwContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: BwUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct BwUniforms {
    pub select: i32,
    pub _padding1: i32,
    pub _padding2: i32,
    pub _padding3: i32,
}

impl Default for BwUniforms {
    fn default() -> Self {
        Self {
            select: 3, // Fallback auf Luminanz
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}

impl EffectContainerObject for BwContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Black and White"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("bw_settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.select, 0..=3).text("mode (rgbl)"))
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "bw"
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
            label: Some("Bw Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bw Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Bw"),
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
    fn to_config(&self) -> EffectConfig {
        EffectConfig::Bw {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
