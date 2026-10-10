use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EffectContainerObject;
use crate::app_state::effect_stack::presets::EffectConfig;

#[derive(Clone)]
pub struct DitherContainer {
    pub id: usize,
    pub enabled: bool,
    pub ui_collapsed: bool,
    pub uniforms: ColorDitherUniforms,
}

impl DitherContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            ui_collapsed: false,
            uniforms: ColorDitherUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorDitherUniforms {
    pub color_steps: f32,
    pub spread: f32,
    pub pattern_scale: f32,
    pub palette_index: i32,

    pub matrix_type: i32,
    pub _padding4: f32,
    pub _padding5: f32,
    pub _padding6: f32,
}

impl Default for ColorDitherUniforms {
    fn default() -> Self {
        Self {
            color_steps: 4.0,
            spread: 0.5,
            pattern_scale: 1.0,
            palette_index: 0,
            matrix_type: 0,
            _padding4: 0.0,
            _padding5: 0.0,
            _padding6: 0.0,
        }
    }
}

impl EffectContainerObject for DitherContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Dither & Palettes"
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
                egui::Slider::new(&mut self.uniforms.color_steps, 2.0..=20.0)
                    .step_by(1.0)
                    .text("color steps"),
            )
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut self.uniforms.spread, 0.0..=5.0).text("spread"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut self.uniforms.pattern_scale, 1.0..=128.0).text("scale"))
            .changed();
        let pallette = [
            "rgb",
            "Game Boy",
            "CGA",
            "PICO-8",
            "Commodore 64",
            "NES",
            "Catppuccin Mocha",
            "Tokyo Night",
            "Gruvbox Dark",
            "warm muted",
            "Earth Vintage",
            "Everforest Dark",
            "Deep Ocean",
            "Ayu Mirage",
        ];
        let txt = pallette
            .get(self.uniforms.palette_index as usize)
            .copied()
            .unwrap_or("rgb");
        changed |= ui
            .add(egui::Slider::new(&mut self.uniforms.palette_index, 0..=13).text(txt))
            .changed();

        let mut is_8x8 = self.uniforms.matrix_type == 1;
        let mattype = match is_8x8 {
            true => "8x8 matrix",
            false => "4x4 matrix",
        };
        if ui.add(egui::Checkbox::new(&mut is_8x8, mattype)).changed() {
            self.uniforms.matrix_type = is_8x8 as i32;
            changed = true;
        }
    
        changed
    }

    fn shader_key(&self) -> &str {
        "dither"
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
            label: Some("Dither Uniform Buffer"),
            contents: bytemuck::bytes_of(&self.uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Dither Pass Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Dither"),
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
        EffectConfig::Dither {
            enabled: self.enabled,
            uniforms: self.uniforms,
        }
    }
}
