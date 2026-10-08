use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EffectContainerObject;
use crate::app_state::effect_stack::presets::EffectConfig;

#[derive(Clone)]
pub struct HsvContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: HsvUniforms,
}

impl HsvContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: HsvUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[serde(default)]
pub struct HsvUniforms {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub factor: f32,
}

impl Default for HsvUniforms {
    fn default() -> Self {
        Self {
            hue: 0.5, // Neutral in Blender
            saturation: 1.0,
            value: 1.0,
            factor: 1.0,
        }
    }
}

impl EffectContainerObject for HsvContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "HSV"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("hsv_settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.hue, 0.0..=1.0).text("hue"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.saturation, 0.0..=2.0).text("saturation"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.value, 0.0..=2.0).text("value"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.factor, 0.0..=1.0).text("factor"))
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "hsv"
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
            label: Some("Hsv Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hsv Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Hsv"),
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
        EffectConfig::Hsv {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
