use crate::app_state::effect_stack::effects::EffectContainerObject;
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
        let effects = vec![effects::EffectSelector::Ascii.into()];
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
