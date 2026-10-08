use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EffectContainerObject;
use crate::app_state::effect_stack::presets::EffectConfig;

#[derive(Clone)]
pub struct PixelateLinearContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: PixelateLinearUniforms,
}

impl PixelateLinearContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: PixelateLinearUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[serde(default)]
pub struct PixelateLinearUniforms {
    pub max_pixel_size: f32,
    pub angle: f32,
    pub start_x: f32,
    pub start_y: f32,

    pub length: f32,
    pub invert: i32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for PixelateLinearUniforms {
    fn default() -> Self {
        Self {
            max_pixel_size: 64.0,
            angle: 0.0,
            start_x: 0.0,
            start_y: 0.5,
            length: 0.8,
            invert: 0,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

impl EffectContainerObject for PixelateLinearContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Pixelate Linear"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("pixelate_linear_settings", |ui| {
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.uniforms.max_pixel_size, 1.0..=128.0)
                        .text("pixel size"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::from_get_set(0.0..=360.0, |new_deg| {
                        if let Some(deg) = new_deg {
                            self.uniforms.angle = (deg as f32).to_radians();
                        }
                        self.uniforms.angle.to_degrees() as f64
                    })
                    .suffix("°")
                    .text("angle"),
                )
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.length, 0.0..=1.0).text("length"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.start_x, 0.0..=1.0).text("start x"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.start_y, 0.0..=1.0).text("start y"))
                .changed();

            let mut is_inverted = self.uniforms.invert == 1;
            if ui
                .add(egui::Checkbox::new(&mut is_inverted, "invert"))
                .changed()
            {
                self.uniforms.invert = is_inverted as i32;
                changed = true;
            }
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "pixelate_linear"
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
            label: Some("Pixelate Linear Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pixelate Linear Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Pixelate Linear"),
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
        EffectConfig::PixelateLinear {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
