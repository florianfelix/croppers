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

    pub startup_crop: StartupCrop,
    pub startup_ui: StartupUi,
    pub startup_effects: StartupEffects,
}
impl Default for CroppersConfig {
    fn default() -> Self {
        Self {
            app_id: "croppers".to_string(),
            fullscreen: false,
            startup_crop: StartupCrop::default(),
            startup_ui: StartupUi::default(),
            startup_effects: StartupEffects::default(),
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
        let mut conf = if let Some(proj_dirs) = ProjectDirs::from("de", &cli.app_id, &cli.app_id) {
            let confdir = proj_dirs.config_dir();
            let path = confdir.join("conf.toml");
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                toml::from_str::<Self>(&content)?
            } else {
                info!("No config found. Using defaults");
                Self::default()
            }
        } else {
            info!("No config found. Using defaults");
            Self::default()
        };

        // Merge runtime CLI value
        conf.app_id = cli.app_id.clone();
        conf.fullscreen = cli.fullscreen;

        Ok(conf)
    }

    pub fn save_defaults() -> Result<()> {
        let test = CroppersConfig::default();
        if let Some(proj_dirs) = ProjectDirs::from("de", "croppers", "croppers") {
            let confdir = proj_dirs.config_dir();
            if !confdir.exists() {
                fs::create_dir_all(confdir)?;
            }
            let path = confdir.join("conf.toml");
            let txt = toml::to_string(&test)?;

            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;

            file.write_all(txt.as_bytes())?;
        }

        info!("Saved defaults\n{:#?}", test);
        Ok(())
    }
}
