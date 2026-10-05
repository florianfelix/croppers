use std::collections::HashMap;

use egui::Pos2;
use itertools::Itertools;

mod crt;
mod vignette;

pub trait EffectContainerObject {
    // --- Metadaten & Zustand ---
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);

    fn name(&self) -> &str;

    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);

    // --- UI Rendering ---
    // Zeichnet die Slider und gibt true zurück, wenn .changed() getriggert wurde.
    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool;

    // --- GPU Pipeline & Pass Konstruktion ---
    // Bestimmt, welchen Shader dieser Effekt nutzen will (z.B. "crt" oder "blur")
    fn shader_key(&self) -> &str;

    // Erlaubt es Effekten, sich in die GpuEffectPass-Vektoren einzuschreiben.
    // Mehrpass-Effekte (wie Blur/TiltShift) pushen hier einfach 2 Pässe hinein!
    fn build_passes(
        &self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        shader_modules: &HashMap<String, wgpu::ShaderModule>,
        passes: &mut Vec<crate::compute::effect_pipeline::GpuEffectPass>,
    );

    // --- GPU Frame Updates ---
    // Schreibt die aktuellen CPU-Werte in die Uniform-Buffer der zugehörigen Pässe.
    fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        matching_passes: &[&crate::compute::effect_pipeline::GpuEffectPass],
    );
}

#[derive(Clone)]
pub struct EffectStack {
    pub effects: Vec<EffectContainer>,
    pub needs_rebuild: bool,
}
impl std::fmt::Debug for EffectStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unstable_name_collisions)]
        let effects: String = self
            .effects
            .iter()
            .map(|e| {
                if e.enabled {
                    e.name.clone()
                } else {
                    e.name.chars().map(|c| format!("{}\u{0336}", c)).collect()
                }
            })
            .intersperse(String::from("-->"))
            .collect();

        write!(f, "needs rebuild: {} |{effects}", self.needs_rebuild)
    }
}
impl Default for EffectStack {
    fn default() -> Self {
        // let effects = vec![
        //     EffectSelector::Vignette.into(),
        //     EffectSelector::Blur.into(),
        //     EffectSelector::Bw.into(),
        // ];

        let effects = vec![EffectSelector::Crt.into()];
        let mut result = Self {
            effects,
            needs_rebuild: true,
        };
        result.validate_ids();
        result
    }
}
impl EffectStack {
    pub fn validate_ids(&mut self) {
        for (idx, e) in self.effects.iter_mut().enumerate() {
            e.id = idx;
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EffectSelector {
    Vignette,
    Blur,
    Bw,
    Hsv,
    Gamma,
    TiltShift,
    PixelateTilt,
    PixelateLinear,
    Dither,
    Crt,
}
impl From<EffectSelector> for EffectContainer {
    fn from(value: EffectSelector) -> Self {
        match value {
            EffectSelector::Vignette => EffectContainer {
                id: 0,
                name: "The Vignette".into(),
                enabled: true,
                effect: Effect::Vignette(VignetteSettings::default()),
            },
            EffectSelector::Blur => EffectContainer {
                id: 0,
                name: "The Blur".into(),
                enabled: true,
                effect: Effect::Blur(BlurSettings::default()),
            },
            EffectSelector::Bw => EffectContainer {
                id: 0,
                name: "Grey".into(),
                enabled: true,
                effect: Effect::Bw(BwSettings::default()),
            },
            EffectSelector::Hsv => EffectContainer {
                id: 0,
                name: "HSV".into(),
                enabled: true,
                effect: Effect::Hsv(HsvSettings::default()),
            },
            EffectSelector::Gamma => EffectContainer {
                id: 0,
                name: "Gamma".into(),
                enabled: true,
                effect: Effect::Gamma(GammaSettings::default()),
            },
            EffectSelector::TiltShift => EffectContainer {
                id: 0,
                name: "Tilt Shift".into(),
                enabled: true,
                effect: Effect::TiltShift(TiltShiftSettings::default()),
            },
            EffectSelector::PixelateTilt => EffectContainer {
                id: 0,
                name: "Pixelate Tilt".into(),
                enabled: true,
                effect: Effect::PixelateTilt(PixelateTiltSettings::default()),
            },
            EffectSelector::PixelateLinear => EffectContainer {
                id: 0,
                name: "Pixelate Linear".into(),
                enabled: true,
                effect: Effect::PixelateLinear(PixelateLinearSettings::default()),
            },
            EffectSelector::Dither => EffectContainer {
                id: 0,
                name: "Dither".into(),
                enabled: true,
                effect: Effect::Dither(DitherSettings::default()),
            },
            EffectSelector::Crt => EffectContainer {
                id: 0,
                name: "Crt".into(),
                enabled: true,
                effect: Effect::Crt(CrtSettings::default()),
            },
        }
    }
}

#[derive(Clone)]
pub struct EffectContainer {
    pub id: usize,
    pub name: String,
    pub enabled: bool,
    pub effect: Effect,
}

#[derive(Clone)]
pub enum Effect {
    Blur(BlurSettings),
    Vignette(VignetteSettings),
    Bw(BwSettings),
    Hsv(HsvSettings),
    Gamma(GammaSettings),
    TiltShift(TiltShiftSettings),
    PixelateTilt(PixelateTiltSettings),
    PixelateLinear(PixelateLinearSettings),
    Dither(DitherSettings),
    Crt(CrtSettings),
}

#[derive(Debug, Clone, Copy)]
pub struct CrtSettings {
    pub curvature: f32,
    pub scanline_count: f32,
    pub scanline_strength: f32,
    pub rgb_mask_strength: f32,
}
impl Default for CrtSettings {
    fn default() -> Self {
        Self {
            curvature: 0.1,
            scanline_count: 240.0,
            scanline_strength: 0.5,
            rgb_mask_strength: 0.2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DitherSettings {
    pub color_steps: f32,
    pub spread: f32,
    pub scale: f32,
    pub palette_index: i32,
    pub matrix_type: i32,
}
impl Default for DitherSettings {
    fn default() -> Self {
        Self {
            color_steps: 4.0,
            spread: 0.5,
            scale: 1.0,
            palette_index: 0,
            matrix_type: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PixelateLinearSettings {
    pub max_pixel_size: f32,
    pub angle: f32,
    pub start_x: f32,
    pub start_y: f32,
    pub length: f32,
    pub invert: i32,
}
impl Default for PixelateLinearSettings {
    fn default() -> Self {
        Self {
            max_pixel_size: 64.0,
            angle: 0.0,
            start_x: 0.0,
            start_y: 0.5,
            length: 0.8,
            invert: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PixelateTiltSettings {
    pub max_pixel_size: f32,
    pub angle: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub focus_size: f32,
    pub feather: f32,
}
impl Default for PixelateTiltSettings {
    fn default() -> Self {
        Self {
            max_pixel_size: 64.0,
            angle: 2.6,
            center_x: 0.5,
            center_y: 0.5,
            focus_size: 0.0,
            feather: 0.55,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TiltShiftSettings {
    pub max_radius: f32,
    pub center: f32,
    pub focus_size: f32,
    pub feather: f32,
}
impl Default for TiltShiftSettings {
    fn default() -> Self {
        Self {
            max_radius: 30.,
            center: 0.5,
            focus_size: 0.1,
            feather: 0.4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GammaSettings {
    pub gr: f32,
    pub gg: f32,
    pub gb: f32,
}
impl Default for GammaSettings {
    fn default() -> Self {
        Self {
            gr: 1.0,
            gg: 1.0,
            gb: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HsvSettings {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub factor: f32,
}
impl Default for HsvSettings {
    fn default() -> Self {
        Self {
            hue: 0.5,
            saturation: 1.0,
            value: 1.0,
            factor: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BwSettings {
    pub select: i32,
}
impl Default for BwSettings {
    fn default() -> Self {
        Self { select: 3 }
    }
}

#[derive(Clone)]
pub struct CropSettings {
    pub enabled: bool,
    pub crop_width: u32,
    pub crop_height: u32,
    pub uv_rect: egui::Rect,
}
impl Default for CropSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            crop_width: 3440,
            crop_height: 1440,
            uv_rect: egui::Rect {
                min: Pos2::ZERO,
                max: Pos2 { x: 1.0, y: 1.0 },
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlurSettings {
    pub radius: f32,
}
impl Default for BlurSettings {
    fn default() -> Self {
        Self { radius: 10.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VignetteSettings {
    pub intensity: f32,
    pub radius: f32,
    pub smoothness: f32,
}
impl Default for VignetteSettings {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            radius: 0.75,
            smoothness: 0.65,
        }
    }
}
