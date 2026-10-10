use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EffectContainerObject;
use crate::app_state::effect_stack::presets::EffectConfig;

#[derive(Clone)]
pub struct VcrContainer {
    pub id: usize,
    pub enabled: bool,
    pub ui_collapsed: bool,
    pub uniforms: VcrUniforms,
}

impl VcrContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            ui_collapsed: false,
            uniforms: VcrUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[serde(default)]
pub struct VcrUniforms {
    pub time: f32,
    pub jitter: f32,
    pub glitch: f32,
    pub glitch_noise: f32,
    pub head_switch: f32,
    pub head_switch_noise: f32,
    pub tape_roll: f32,
    pub chroma_decay: f32,
}

impl Default for VcrUniforms {
    fn default() -> Self {
        Self {
            time: 20.0,
            jitter: 1.0,
            glitch: 3.0,
            glitch_noise: 1.0,
            head_switch: 5.0,
            head_switch_noise: 0.1,
            tape_roll: 1.0,
            chroma_decay: 0.5,
        }
    }
}

impl EffectContainerObject for VcrContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Vcr"
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
            .add(elegance::Slider::new(&mut self.uniforms.time, 0.0..=200.0).label("time"))
            .changed();
        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.jitter, 0.0..=5.0).label("jitter"))
            .changed();
        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.glitch, 0.0..=10.0).label("glitch"))
            .changed();
        changed |= ui
            .add(
                elegance::Slider::new(&mut self.uniforms.glitch_noise, 0.0..=3.0)
                    .label("glitch noise"),
            )
            .changed();
        changed |= ui
            .add(
                elegance::Slider::new(&mut self.uniforms.head_switch, 0.0..=10.0)
                    .label("head-switch"),
            )
            .changed();
        changed |= ui
            .add(
                elegance::Slider::new(&mut self.uniforms.head_switch_noise, 0.0..=3.0)
                    .label("head-switch noise"),
            )
            .changed();
        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.tape_roll, 0.0..=10.0).label("tape roll"))
            .changed();
        changed |= ui
            .add(
                elegance::Slider::new(&mut self.uniforms.chroma_decay, 0.0..=1.0)
                    .label("chroma decay"),
            )
            .changed();
    
        changed
    }

    fn shader_key(&self) -> &str {
        "vcr"
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
            label: Some("Vcr Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Vcr Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Vcr"),
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
        EffectConfig::Vcr {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
