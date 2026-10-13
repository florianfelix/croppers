use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EffectContainerObject;
use crate::app_state::effect_stack::presets::EffectConfig;

#[derive(Clone)]
pub struct CutoffContainer {
    pub id: usize,
    pub enabled: bool,
    pub ui_collapsed: bool,
    pub uniforms: CutoffUniforms,
}

impl CutoffContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            ui_collapsed: false,
            uniforms: CutoffUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[serde(default)]
pub struct CutoffUniforms {
    pub cutoff_below: f32,
    pub cutoff_above: f32,
    pub falloff: f32,
    pub reintroduce: f32,
    pub angle: f32,
    pub below_enabled: u32,
    pub above_enabled: u32,
    pub mode: u32,
}

impl Default for CutoffUniforms {
    fn default() -> Self {
        Self {
            cutoff_below: 0.1,
            cutoff_above: 0.9,
            falloff: 0.05,
            reintroduce: 0.0,
            angle: std::f32::consts::FRAC_PI_2, // 90°: vertical (bottom to top)
            below_enabled: 1,
            above_enabled: 1,
            mode: 0,
        }
    }
}

impl EffectContainerObject for CutoffContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Cutoff"
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

        ui.horizontal(|ui| {
            ui.label("Mode:");
            if ui
                .selectable_label(self.uniforms.mode == 0, "Luminance")
                .clicked()
            {
                if self.uniforms.mode != 0 {
                    self.uniforms.mode = 0;
                    changed = true;
                }
            }
            if ui
                .selectable_label(self.uniforms.mode == 1, "Linear")
                .clicked()
            {
                if self.uniforms.mode != 1 {
                    self.uniforms.mode = 1;
                    changed = true;
                }
            }
        });

        if self.uniforms.mode == 1 {
            let mut angle_deg = self.uniforms.angle.to_degrees().rem_euclid(360.0);
            if ui
                .add(
                    elegance::Slider::new(&mut angle_deg, 0.0..=360.0)
                        .suffix("°")
                        .label("Angle"),
                )
                .changed()
            {
                self.uniforms.angle = angle_deg.to_radians();
                changed = true;
            }
        }

        ui.separator();

        let mut below_on = self.uniforms.below_enabled == 1;
        if ui.checkbox(&mut below_on, "Cutoff Below").changed() {
            self.uniforms.below_enabled = if below_on { 1 } else { 0 };
            changed = true;
        }
        ui.add_enabled_ui(below_on, |ui| {
            if ui
                .add(
                    elegance::Slider::new(&mut self.uniforms.cutoff_below, 0.0..=1.0)
                        .label("Threshold Below"),
                )
                .changed()
            {
                if self.uniforms.cutoff_below > self.uniforms.cutoff_above {
                    self.uniforms.cutoff_above = self.uniforms.cutoff_below;
                }
                changed = true;
            }
        });

        ui.add_space(2.0);

        let mut above_on = self.uniforms.above_enabled == 1;
        if ui.checkbox(&mut above_on, "Cutoff Above").changed() {
            self.uniforms.above_enabled = if above_on { 1 } else { 0 };
            changed = true;
        }
        ui.add_enabled_ui(above_on, |ui| {
            if ui
                .add(
                    elegance::Slider::new(&mut self.uniforms.cutoff_above, 0.0..=1.0)
                        .label("Threshold Above"),
                )
                .changed()
            {
                if self.uniforms.cutoff_above < self.uniforms.cutoff_below {
                    self.uniforms.cutoff_below = self.uniforms.cutoff_above;
                }
                changed = true;
            }
        });

        ui.separator();

        changed |= ui
            .add(elegance::Slider::new(&mut self.uniforms.falloff, 0.0..=0.5).label("Falloff"))
            .changed();

        changed |= ui
            .add(
                elegance::Slider::new(&mut self.uniforms.reintroduce, 0.0..=1.0)
                    .label("Reintroduce Original"),
            )
            .changed();

        changed
    }

    fn shader_key(&self) -> &str {
        "cutoff"
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
            label: Some("Cutoff Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cutoff Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Cutoff"),
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
        EffectConfig::Cutoff {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
