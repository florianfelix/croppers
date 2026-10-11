use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EffectContainerObject;
use crate::app_state::effect_stack::presets::EffectConfig;

#[derive(Clone)]
pub struct PixelateTiltContainer {
    pub id: usize,
    pub enabled: bool,
    pub ui_collapsed: bool,
    pub uniforms: PixelateTiltUniforms,
}

impl PixelateTiltContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            ui_collapsed: false,
            uniforms: PixelateTiltUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[serde(default)]
pub struct PixelateTiltUniforms {
    pub max_pixel_size: f32,
    pub angle: f32,
    pub center_x: f32,
    pub center_y: f32,

    pub focus_size: f32,
    pub feather: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for PixelateTiltUniforms {
    fn default() -> Self {
        Self {
            max_pixel_size: 64.0,
            angle: 0.0,
            center_x: 0.5,
            center_y: 0.5,
            focus_size: 0.1,
            feather: 0.4,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

impl EffectContainerObject for PixelateTiltContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Pixelate Tilt Shift"
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
                elegance::Slider::new(&mut self.uniforms.max_pixel_size, 1.0..=128.0)
                    .label("pixel size"),
            )
            .changed();
        let mut angle_deg = self.uniforms.angle.to_degrees();
        if ui
            .add(
                elegance::Slider::new(&mut angle_deg, 0.0..=360.0)
                    .suffix("°")
                    .label("angle"),
            )
            .changed()
        {
            self.uniforms.angle = angle_deg.to_radians();
            changed = true;
        }
        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.center_x, 0.0..=1.0).label("center x"))
            .changed();
        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.center_y, 0.0..=1.0).label("center y"))
            .changed();
        changed |= ui
            .add(
                elegance::Slider::new(&mut self.uniforms.focus_size, 0.0..=1.0).label("focus size"),
            )
            .changed();
        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.feather, 0.0..=1.0).label("feather"))
            .changed();

        changed
    }

    fn shader_key(&self) -> &str {
        "pixelate"
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
            label: Some("Pixelate Tilt Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pixelate Tilt Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Pixelate Tilt"),
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
        EffectConfig::PixelateTilt {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
