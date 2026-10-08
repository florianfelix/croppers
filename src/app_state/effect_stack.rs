use std::{collections::HashMap, path::PathBuf};

use dyn_clone::DynClone;
use enum_iterator::Sequence;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::app_state::effect_stack::{
    ascii::AsciiContainer, blur::BlurContainer, bw::BwContainer, crt::CrtContainer,
    dither::DitherContainer, gamma::GammaContainer, glitch::GlitchContainer, hsv::HsvContainer,
    pixelate_linear::PixelateLinearContainer, pixelate_tilt::PixelateTiltContainer,
    tiltshift::TiltShiftContainer, vcr::VcrContainer, vignette::VignetteContainer,
};

mod ascii;
mod blur;
mod bw;
mod crt;
mod dither;
mod gamma;
mod glitch;
mod hsv;
mod pixelate_linear;
mod pixelate_tilt;
mod tiltshift;
mod vcr;
pub mod vignette;

pub trait EffectContainerObject: DynClone + Send + Sync {
    // --- Metadaten & Zustand ---
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);

    fn name(&self) -> &str;

    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool;

    fn shader_key(&self) -> &str;

    fn build_passes(
        &self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        shader_modules: &HashMap<String, wgpu::ShaderModule>,
        passes: &mut Vec<crate::compute::effect_pipeline::GpuEffectPass>,
    );

    fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        matching_passes: &[&crate::compute::effect_pipeline::GpuEffectPass],
    );
    fn to_config(&self) -> EffectConfig;
}
dyn_clone::clone_trait_object!(EffectContainerObject);

#[derive(Clone)]
pub struct EffectStack {
    pub effects: Vec<Box<dyn EffectContainerObject>>,
    pub needs_rebuild: bool,
    pub enabled: bool,
    pub presets: Vec<PathBuf>,
}

impl Default for EffectStack {
    fn default() -> Self {
        let effects = vec![EffectSelector::Ascii.into()];
        let mut result = Self {
            effects,
            needs_rebuild: true,
            enabled: true,
            presets: vec![],
        };
        result.validate_ids();
        result.refresh_presets();
        result
    }
}
impl EffectStack {
    pub fn validate_ids(&mut self) {
        for (idx, e) in self.effects.iter_mut().enumerate() {
            e.set_id(idx);
        }
    }
    /// Convert current runtime stack to a serializable DTO
    pub fn to_saved_stack(&self) -> SavedEffectStack {
        SavedEffectStack {
            enabled: self.enabled,
            effects: self.effects.iter().map(|e| e.to_config()).collect(),
        }
    }

    /// Restore the effect stack from a saved configuration
    pub fn from_saved_stack(saved: SavedEffectStack) -> Self {
        let mut stack = Self {
            enabled: saved.enabled,
            effects: saved.effects.into_iter().map(Into::into).collect(),
            needs_rebuild: true,
            presets: vec![],
        };
        stack.validate_ids();
        stack.refresh_presets();
        stack
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Sequence)]
pub enum EffectSelector {
    Blur,
    Vignette,
    Bw,
    Hsv,
    Gamma,
    TiltShift,
    PixelateTilt,
    PixelateLinear,
    Dither,
    Crt,
    Ascii,
    Vcr,
    Glitch,
}
impl From<EffectSelector> for Box<dyn EffectContainerObject> {
    fn from(value: EffectSelector) -> Self {
        match value {
            EffectSelector::Crt => CrtContainer::new(),
            EffectSelector::Blur => BlurContainer::new(),
            EffectSelector::PixelateLinear => PixelateLinearContainer::new(),
            EffectSelector::PixelateTilt => PixelateTiltContainer::new(),
            EffectSelector::Dither => DitherContainer::new(),
            EffectSelector::Vignette => VignetteContainer::new(),
            EffectSelector::Bw => BwContainer::new(),
            EffectSelector::Hsv => HsvContainer::new(),
            EffectSelector::Gamma => GammaContainer::new(),
            EffectSelector::TiltShift => TiltShiftContainer::new(),
            EffectSelector::Ascii => AsciiContainer::new(),
            EffectSelector::Vcr => VcrContainer::new(),
            EffectSelector::Glitch => GlitchContainer::new(),
        }
    }
}

impl EffectSelector {
    pub fn all() -> Vec<Self> {
        enum_iterator::all::<Self>().collect_vec()
    }
    pub fn draw_all(ui: &mut egui::Ui, effect_stack: &mut EffectStack) {
        for selector in Self::all() {
            let name = format!("{:?}", selector);
            if ui.selectable_label(false, &name).clicked() {
                let mut new_effect: Box<dyn EffectContainerObject> = selector.into();
                new_effect.set_id(effect_stack.effects.len());
                effect_stack.effects.push(new_effect);
                effect_stack.needs_rebuild = true;
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedEffectStack {
    pub enabled: bool,
    pub effects: Vec<EffectConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectConfig {
    Blur {
        enabled: bool,
        uniforms: blur::BlurUniforms,
    },
    Vignette {
        enabled: bool,
        uniforms: vignette::VignetteUniforms,
    },
    Bw {
        enabled: bool,
        uniforms: bw::BwUniforms,
    },
    Hsv {
        enabled: bool,
        uniforms: hsv::HsvUniforms,
    },
    Gamma {
        enabled: bool,
        uniforms: gamma::GammaUniforms,
    },
    TiltShift {
        enabled: bool,
        uniforms: tiltshift::TiltShiftUniforms,
    },
    PixelateTilt {
        enabled: bool,
        uniforms: pixelate_tilt::PixelateTiltUniforms,
    },
    PixelateLinear {
        enabled: bool,
        uniforms: pixelate_linear::PixelateLinearUniforms,
    },
    Dither {
        enabled: bool,
        uniforms: dither::ColorDitherUniforms,
    },
    Crt {
        enabled: bool,
        uniforms: crt::CrtUniforms,
    },
    Ascii {
        enabled: bool,
        uniforms: ascii::AsciiUniforms,
    },
    Vcr {
        enabled: bool,
        uniforms: vcr::VcrUniforms,
    },
    Glitch {
        enabled: bool,
        uniforms: glitch::GlitchUniforms,
    },
}

impl From<EffectConfig> for Box<dyn EffectContainerObject> {
    fn from(config: EffectConfig) -> Self {
        match config {
            EffectConfig::Blur { enabled, uniforms } => {
                let mut c = BlurContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Ascii { enabled, uniforms } => {
                let mut c = AsciiContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Crt { enabled, uniforms } => {
                let mut c = CrtContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Dither { enabled, uniforms } => {
                let mut c = DitherContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Vignette { enabled, uniforms } => {
                let mut c = VignetteContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Bw { enabled, uniforms } => {
                let mut c = BwContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Hsv { enabled, uniforms } => {
                let mut c = HsvContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Gamma { enabled, uniforms } => {
                let mut c = GammaContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::TiltShift { enabled, uniforms } => {
                let mut c = TiltShiftContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::PixelateTilt { enabled, uniforms } => {
                let mut c = PixelateTiltContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::PixelateLinear { enabled, uniforms } => {
                let mut c = PixelateLinearContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Vcr { enabled, uniforms } => {
                let mut c = VcrContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
            EffectConfig::Glitch { enabled, uniforms } => {
                let mut c = GlitchContainer::new();
                c.enabled = enabled;
                c.uniforms = uniforms;
                c
            }
        }
    }
}
