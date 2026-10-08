use std::{fs, io::Write};

use crate::app_state::widgets::CropMode;
use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CroppersConfig {
    #[serde(skip)]
    pub app_id: String,
    #[serde(skip)]
    pub fullscreen: bool,
    #[serde(skip)]
    pub is_new_config: bool,

    pub startup_crop: StartupCrop,
    pub startup_ui: StartupUi,
    pub startup_effects: StartupEffects,
    pub on_image_load: OnImageLoad,
}
impl Default for CroppersConfig {
    fn default() -> Self {
        Self {
            app_id: "croppers".to_string(),
            fullscreen: false,
            is_new_config: false,
            startup_crop: StartupCrop::default(),
            startup_ui: StartupUi::default(),
            startup_effects: StartupEffects::default(),
            on_image_load: OnImageLoad::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct OnImageLoad {
    pub crop_enabled: bool,
    pub effects_enabled: bool,
    pub empty_effects: bool,
}
impl Default for OnImageLoad {
    fn default() -> Self {
        Self {
            crop_enabled: true,
            effects_enabled: true,
            empty_effects: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StartupEffects {
    pub enabled: bool,
}
impl Default for StartupEffects {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StartupUi {
    pub show_panel: bool,
}
impl Default for StartupUi {
    fn default() -> Self {
        Self { show_panel: true }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StartupCrop {
    pub enabled: bool,
    pub default_crop_mode: CropMode,
    pub fixed_crop_width: u32,
    pub fixed_crop_height: u32,
}
impl Default for StartupCrop {
    fn default() -> Self {
        Self {
            enabled: true,
            default_crop_mode: CropMode::FixedDimensions,
            fixed_crop_width: 1920,
            fixed_crop_height: 1080,
        }
    }
}

impl CroppersConfig {
    pub fn load(cli: &super::Cli) -> Result<Self> {
        let mut conf = if let Some(proj_dirs) = ProjectDirs::from("de", "ff", "croppers") {
            let confdir = proj_dirs.config_dir();
            let path = confdir.join("conf.toml");
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                toml::from_str::<Self>(&content)?
            } else {
                info!("No config found. Using defaults");
                let mut conf = Self::default();
                conf.is_new_config = true;
                conf
            }
        } else {
            info!("No config found. Using defaults");
            let mut conf = Self::default();
            conf.is_new_config = true;
            conf
        };

        // Merge runtime CLI value
        conf.app_id = cli.app_id.clone();
        conf.fullscreen = cli.fullscreen;

        Ok(conf)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(proj_dirs) = ProjectDirs::from("de", "ff", "croppers") {
            let confdir = proj_dirs.config_dir();
            if !confdir.exists() {
                fs::create_dir_all(confdir)?;
            }
            let path = confdir.join("conf.toml");
            let txt = toml::to_string(&self)?;

            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)?;

            file.write_all(txt.as_bytes())?;
            info!("Saved config to {:?}", path);
        }
        Ok(())
    }

    pub fn save_defaults() -> Result<()> {
        let test = CroppersConfig::default();
        test.save()
    }
}
