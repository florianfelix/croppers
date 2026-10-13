use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EffectContainerObject;
use crate::app_state::effect_stack::presets::EffectConfig;

#[derive(Clone)]
pub struct BrightnessContrastContainer {
    pub id: usize,
    pub enabled: bool,
    pub ui_collapsed: bool,
    pub uniforms: BrightnessContrastUniforms,
}

impl BrightnessContrastContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            ui_collapsed: false,
            uniforms: BrightnessContrastUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[serde(default)]
pub struct BrightnessContrastUniforms {
    pub brightness: f32,
    pub contrast: f32,
    pub _padding: [f32; 2],
}

impl Default for BrightnessContrastUniforms {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            _padding: [0.0; 2],
        }
    }
}

impl EffectContainerObject for BrightnessContrastContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Brightness / Contrast"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn ui_collapsed(&self) -> bool {
        self.ui_collapsed
    }
    fn set_ui_collapsed(&mut self, collapsed: bool) {
        self.ui_collapsed = collapsed;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        changed |= ui
            .add(
                elegance::Slider::new(&mut self.uniforms.brightness, -1.0..=1.0)
                    .label("Brightness"),
            )
            .changed();
        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.contrast, 0.0..=4.0).label("Contrast"))
            .changed();
        changed
    }

    fn shader_key(&self) -> &str {
        "brightness_contrast"
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
            label: Some("Brightness / Contrast Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Brightness / Contrast Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Brightness / Contrast"),
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
        EffectConfig::BrightnessContrast {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
