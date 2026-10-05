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
        let effects = vec![
            EffectSelector::Vignette.into(),
            EffectSelector::Blur.into(),
            EffectSelector::Bw.into(),
        ];

        // let effects = vec![];
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
