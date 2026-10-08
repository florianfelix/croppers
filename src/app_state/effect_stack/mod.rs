use crate::app_state::effect_stack::effects::{EffectContainerObject, prelude::*};
use enum_iterator::Sequence;
use itertools::Itertools;
use std::path::PathBuf;

pub mod effects;
pub mod presets;

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
