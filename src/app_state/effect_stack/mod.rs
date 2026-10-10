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
    pub selected_preset: Option<PathBuf>,
    pub preset_name_input: String,
    pub presets_collapsed: bool,
}

impl Default for EffectStack {
    fn default() -> Self {
        let effects = vec![effects::EffectSelector::Vignette.into()];
        let mut result = Self {
            effects,
            needs_rebuild: true,
            enabled: true,
            presets: vec![],
            selected_preset: None,
            preset_name_input: String::new(),
            presets_collapsed: false,
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
