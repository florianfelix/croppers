use directories::ProjectDirs;
use effects::prelude::*;
use serde::{Deserialize, Serialize};

use super::effects;
use crate::app_state::effect_stack::EffectStack;

impl EffectStack {
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
    pub fn save_to_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let saved = self.to_saved_stack();
        let ron_or_json = toml::to_string_pretty(&saved)?; // or serde_json / ron
        std::fs::write(path, ron_or_json)?;
        self.refresh_presets();
        Ok(())
    }

    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let saved: SavedEffectStack = toml::from_str(&content)?;
        Ok(Self::from_saved_stack(saved))
    }
    pub fn refresh_presets(&mut self) {
        let presets_dir = if let Some(proj_dirs) = ProjectDirs::from("de", "ff", "croppers") {
            let confdir = proj_dirs.config_dir();
            let presets_dir = confdir.join("presets");
            if !presets_dir.exists() {
                if std::fs::create_dir_all(&presets_dir).is_ok() {
                    Some(presets_dir)
                } else {
                    None
                }
            } else {
                Some(presets_dir)
            }
        } else {
            None
        };
        if let Some(presets_dir) = presets_dir {
            if let Ok(entries) = std::fs::read_dir(presets_dir) {
                let presets = entries
                    .flatten()
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                            Some(path)
                        } else {
                            None
                        }
                    })
                    .collect();
                self.presets = presets;
            }
        }
        self.presets = vec![];
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
        uniforms: effects::blur::BlurUniforms,
    },
    Vignette {
        enabled: bool,
        uniforms: effects::vignette::VignetteUniforms,
    },
    Bw {
        enabled: bool,
        uniforms: effects::bw::BwUniforms,
    },
    Hsv {
        enabled: bool,
        uniforms: effects::hsv::HsvUniforms,
    },
    Gamma {
        enabled: bool,
        uniforms: effects::gamma::GammaUniforms,
    },
    TiltShift {
        enabled: bool,
        uniforms: effects::tiltshift::TiltShiftUniforms,
    },
    PixelateTilt {
        enabled: bool,
        uniforms: effects::pixelate_tilt::PixelateTiltUniforms,
    },
    PixelateLinear {
        enabled: bool,
        uniforms: effects::pixelate_linear::PixelateLinearUniforms,
    },
    Dither {
        enabled: bool,
        uniforms: effects::dither::ColorDitherUniforms,
    },
    Crt {
        enabled: bool,
        uniforms: effects::crt::CrtUniforms,
    },
    Ascii {
        enabled: bool,
        uniforms: effects::ascii::AsciiUniforms,
    },
    Vcr {
        enabled: bool,
        uniforms: effects::vcr::VcrUniforms,
    },
    Glitch {
        enabled: bool,
        uniforms: effects::glitch::GlitchUniforms,
    },
}

impl From<EffectConfig> for Box<dyn effects::EffectContainerObject> {
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
