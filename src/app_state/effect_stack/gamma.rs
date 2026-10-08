use serde::{Deserialize, Serialize};

use crate::app_state::effect_stack::EffectContainerObject;
use std::collections::HashMap;

#[derive(Clone)]
pub struct GammaContainer {
    pub id: usize,
    pub enabled: bool,
    pub linked_channels: bool,
    pub uniforms: GammaUniforms,
}

impl GammaContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            linked_channels: true,
            uniforms: GammaUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct GammaUniforms {
    pub gr: f32,
    pub gg: f32,
    pub gb: f32,
    pub _padding: f32,
}

impl Default for GammaUniforms {
    fn default() -> Self {
        Self {
            gr: 1.0,
            gg: 1.0,
            gb: 1.0,
            _padding: 0.0,
        }
    }
}

impl EffectContainerObject for GammaContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Gamma"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("gamma_settings", |ui| {
            // Toggle between unified single-channel mode and individual RGB channels
            if ui
                .checkbox(&mut self.linked_channels, "Link RGB channels")
                .changed()
            {
                if self.linked_channels {
                    // Synchronize green and blue to red upon linking
                    self.uniforms.gg = self.uniforms.gr;
                    self.uniforms.gb = self.uniforms.gr;
                    changed = true;
                }
            }

            if self.linked_channels {
                // Unified single slider controlling all channels
                let mut unified_gamma = self.uniforms.gr;
                if ui
                    .add(egui::Slider::new(&mut unified_gamma, 0.01..=10.0).text("gamma"))
                    .changed()
                {
                    self.uniforms.gr = unified_gamma;
                    self.uniforms.gg = unified_gamma;
                    self.uniforms.gb = unified_gamma;
                    changed = true;
                }
            } else {
                // Individual sliders per channel
                changed |= ui
                    .add(egui::Slider::new(&mut self.uniforms.gr, 0.01..=10.0).text("gamma red"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.uniforms.gg, 0.01..=10.0).text("gamma green"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.uniforms.gb, 0.01..=10.0).text("gamma blue"))
                    .changed();
            }
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "gamma"
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
            label: Some("Gamma Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gamma Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Gamma"),
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
    fn to_config(&self) -> super::EffectConfig {
        super::EffectConfig::Gamma {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
