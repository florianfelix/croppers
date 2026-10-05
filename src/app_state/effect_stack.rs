use std::{collections::HashMap, sync::Arc};

use dyn_clone::DynClone;
use egui::Pos2;
use itertools::Itertools;

use crate::app_state::effect_stack::{
    blur::BlurContainer, bw::BwContainer, crt::CrtContainer, dither::DitherContainer,
    gamma::GammaContainer, hsv::HsvContainer, pixelate_linear::PixelateLinearContainer,
    pixelate_tilt::PixelateTiltContainer, tiltshift::TiltShiftContainer,
    vignette::VignetteContainer,
};

mod blur;
mod bw;
mod crt;
mod dither;
mod gamma;
mod hsv;
mod pixelate_linear;
mod pixelate_tilt;
mod tiltshift;
pub mod vignette;

pub trait EffectContainerObject: DynClone + Send + Sync {
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
dyn_clone::clone_trait_object!(EffectContainerObject);

#[derive(Clone)]
pub struct EffectStack {
    pub effects: Vec<Box<dyn EffectContainerObject>>,
    pub needs_rebuild: bool,
}

impl Default for EffectStack {
    fn default() -> Self {
        // let effects = vec![
        //     EffectSelector::Vignette.into(),
        //     EffectSelector::Blur.into(),
        //     EffectSelector::Bw.into(),
        // ];

        let effects = vec![EffectSelector::Vignette.into()];
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
        for (idx, mut e) in self.effects.iter_mut().enumerate() {
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
        }
    }
}
