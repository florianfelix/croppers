use egui::Pos2;
use itertools::Itertools;

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
            .map(|e| e.name.clone())
            .intersperse(String::from("\n"))
            .collect();

        // effects.extend(self.effects.iter().map(|e| e.name.push_str("\n")));

        write!(f, "needs rebuild: {}\n{effects}", self.needs_rebuild)
    }
}
impl Default for EffectStack {
    fn default() -> Self {
        let vignette = EffectContainer {
            id: 0,
            name: String::from("The Vignette"),
            enabled: true,
            effect: Effect::Vignette(VignetteSettings {
                intensity: 1.0,
                radius: 0.75,
                smoothness: 0.65,
            }),
        };
        let blur = EffectContainer {
            id: 1,
            name: String::from("Blur"),
            enabled: true,
            effect: Effect::Blur(BlurSettings {
                radius: 10.0,
                sigma: 1.0,
            }),
        };
        let _vignette2 = EffectContainer {
            id: 3,
            name: String::from("The Vignette"),
            enabled: true,
            effect: Effect::Vignette(VignetteSettings {
                intensity: 1.0,
                radius: 0.75,
                smoothness: 0.65,
            }),
        };
        Self {
            effects: vec![vignette, blur],
            needs_rebuild: true,
        }
    }
}
impl EffectStack {
    pub fn validate_ids(&mut self) {
        for (idx, e) in self.effects.iter_mut().enumerate() {
            e.id = idx;
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
    Crop(CropSettings),
    Blur(BlurSettings),
    Vignette(VignetteSettings),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EffectSelector {
    Vignette,
    Blur,
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
                effect: Effect::Blur(BlurSettings {
                    radius: 10.0,
                    sigma: 1.0,
                }),
            },
        }
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
    pub sigma: f32,
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
