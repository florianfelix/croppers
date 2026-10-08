use dyn_clone::DynClone;
use enum_iterator::Sequence;
use itertools::Itertools;
use std::collections::HashMap;

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
mod vignette;
pub mod prelude {
    pub use super::ascii::*;
    pub use super::blur::*;
    pub use super::bw::*;
    pub use super::crt::*;
    pub use super::dither::*;
    pub use super::gamma::*;
    pub use super::glitch::*;
    pub use super::hsv::*;
    pub use super::pixelate_linear::*;
    pub use super::pixelate_tilt::*;
    pub use super::tiltshift::*;
    pub use super::vcr::*;
    pub use super::vignette::*;
}
use self::prelude::*;

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
    fn to_config(&self) -> super::presets::EffectConfig;
}
dyn_clone::clone_trait_object!(EffectContainerObject);

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
    pub fn draw_all(ui: &mut egui::Ui, effect_stack: &mut super::EffectStack) {
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
