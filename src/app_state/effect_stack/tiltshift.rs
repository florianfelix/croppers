use crate::app_state::effect_stack::EffectContainerObject;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TiltShiftContainer {
    pub id: usize,
    pub enabled: bool,
    pub uniforms: TiltShiftUniforms,
}

impl TiltShiftContainer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            id: 0,
            enabled: true,
            uniforms: TiltShiftUniforms::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TiltShiftUniforms {
    // --- 1. Block (16 Bytes) ---
    pub max_radius: f32,
    pub dir_x: i32,
    pub dir_y: i32,
    pub center: f32,

    // --- 2. Block (16 Bytes) ---
    pub focus_size: f32,
    pub feather: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for TiltShiftUniforms {
    fn default() -> Self {
        Self {
            max_radius: 30.0,
            dir_x: 1, // Standardmäßig auf Horizontal-Werte initialisiert
            dir_y: 0,
            center: 0.5,
            focus_size: 0.1,
            feather: 0.4,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

impl EffectContainerObject for TiltShiftContainer {
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn name(&self) -> &str {
        "Tilt Shift"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.indent("tiltshift_settings", |ui| {
            // Die UI manipuliert nur max_radius, center, focus_size und feather.
            // dir_x und dir_y werden im UI ignoriert, da sie reine Steuerflags für die Pipeline sind.
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.max_radius, 0.0..=200.0).text("Radius"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.center, 0.0..=1.0).text("Center"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.focus_size, 0.0..=1.0).text("focus size"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.uniforms.feather, 0.0..=1.0).text("feather"))
                .changed();
        });
        changed
    }

    fn shader_key(&self) -> &str {
        "tiltshift"
    }

    fn build_passes(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        shaders: &HashMap<String, wgpu::ShaderModule>,
        passes: &mut Vec<crate::compute::effect_pipeline::GpuEffectPass>,
    ) {
        use wgpu::util::DeviceExt;

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TiltShift Pipeline Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline TiltShift"),
            layout: Some(&pipeline_layout),
            module: &shaders[self.shader_key()],
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        // --- PASS 1: Horizontaler Pass ---
        // Wir nehmen die aktuellen Slider-Werte und zwingen dir_x=1, dir_y=0
        let mut h_uniforms = self.uniforms;
        h_uniforms.dir_x = 1;
        h_uniforms.dir_y = 0;

        let h_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TiltShift Horizontal Uniform Buffer"),
            contents: bytemuck::bytes_of(&h_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        passes.push(crate::compute::effect_pipeline::GpuEffectPass {
            effect_id: self.id,
            pipeline: pipeline.clone(),
            uniform_buffer: h_buf,
        });

        // --- PASS 2: Vertikaler Pass ---
        // Wir nehmen dieselben Slider-Werte und zwingen dir_x=0, dir_y=1
        let mut v_uniforms = self.uniforms;
        v_uniforms.dir_x = 0;
        v_uniforms.dir_y = 1;

        let v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TiltShift Vertical Uniform Buffer"),
            contents: bytemuck::bytes_of(&v_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        passes.push(crate::compute::effect_pipeline::GpuEffectPass {
            effect_id: self.id,
            pipeline,
            uniform_buffer: v_buf,
        });
    }

    fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        matching_passes: &[&crate::compute::effect_pipeline::GpuEffectPass],
    ) {
        // ESSENZIELL: Beim Slider-Update müssen wir die Richtungs-Flags zwingend wieder mitsenden,
        // da die GPU sonst nicht mehr weiß, in welche Achse sie weichzeichnen soll!

        // Pass 0 (Horizontal) updaten
        if let Some(h_pass) = matching_passes.first() {
            let mut h_uniforms = self.uniforms;
            h_uniforms.dir_x = 1;
            h_uniforms.dir_y = 0;
            queue.write_buffer(&h_pass.uniform_buffer, 0, bytemuck::bytes_of(&h_uniforms));
        }

        // Pass 1 (Vertikal) updaten
        if let Some(v_pass) = matching_passes.get(1) {
            let mut v_uniforms = self.uniforms;
            v_uniforms.dir_x = 0;
            v_uniforms.dir_y = 1;
            queue.write_buffer(&v_pass.uniform_buffer, 0, bytemuck::bytes_of(&v_uniforms));
        }
    }
}
