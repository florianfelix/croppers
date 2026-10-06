use std::collections::HashMap;

use dyn_clone::DynClone;

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
}
dyn_clone::clone_trait_object!(EffectContainerObject);

#[derive(Clone)]
pub struct EffectStack {
    pub effects: Vec<Box<dyn EffectContainerObject>>,
    pub needs_rebuild: bool,
    pub enabled: bool,
}

impl Default for EffectStack {
    fn default() -> Self {
        let effects = vec![EffectSelector::Glitch.into()];
        let mut result = Self {
            effects,
            needs_rebuild: true,
            enabled: true,
        };
        result.validate_ids();
        result
    }
}
impl EffectStack {
    pub fn validate_ids(&mut self) {
        for (idx, e) in self.effects.iter_mut().enumerate() {
            e.set_id(idx);
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
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
