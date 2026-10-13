use std::{fs, io::Write, path::PathBuf};

use crate::app_state::widgets::CropMode;
use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    Auto,
    Slate,
    Charcoal,
    Frost,
    Paper,
}

impl ThemeMode {
    pub const ALL: [Self; 5] = [
        Self::Auto,
        Self::Slate,
        Self::Charcoal,
        Self::Frost,
        Self::Paper,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Slate => "Slate (Dark Blue)",
            Self::Charcoal => "Charcoal (Dark Grey)",
            Self::Frost => "Frost (Light Blue)",
            Self::Paper => "Paper (Light Warm)",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CroppersConfig {
    #[serde(skip)]
    pub app_id: String,
    #[serde(skip)]
    pub fullscreen: bool,
    #[serde(skip)]
    pub is_new_config: bool,
    #[serde(skip)]
    pub startup_file: Option<PathBuf>,

    pub theme_mode: ThemeMode,
    pub appearance: AppearanceConfig,
    pub startup: Startup,
    pub on_image_load: OnImageLoad,
}
impl Default for CroppersConfig {
    fn default() -> Self {
        Self {
            app_id: "croppers".to_string(),
            fullscreen: false,
            is_new_config: false,
            startup_file: None,
            theme_mode: ThemeMode::default(),
            appearance: AppearanceConfig::default(),
            startup: Startup::default(),
            on_image_load: OnImageLoad::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Startup {
    pub enable_effects: bool,
    pub show_panel: bool,
    pub default_crop_setting: StartupCrop,
}
impl Default for Startup {
    fn default() -> Self {
        Self {
            enable_effects: false,
            show_panel: true,
            default_crop_setting: StartupCrop::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub accent_color: [f32; 4],
    pub canvas_bg_color: [f32; 4],
    pub canvas_shadow_color: [f32; 4],
    pub canvas_radius: f32,
    pub canvas_shadow_offset_x: f32,
    pub canvas_shadow_offset_y: f32,
}
impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            accent_color: [0.125, 0.0, 0.2, 1.0],
            canvas_bg_color: [0.08, 0.08, 0.09, 1.0],
            canvas_shadow_color: [0.04, 0.04, 0.04, 0.6],
            canvas_radius: 100.0,
            canvas_shadow_offset_x: 0.0,
            canvas_shadow_offset_y: 0.0,
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
            crop_enabled: false,
            effects_enabled: false,
            empty_effects: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupCrop {
    pub enabled: bool,
    pub default_crop_mode: CropMode,
    pub fixed_crop_width: u32,
    pub fixed_crop_height: u32,
}
impl Default for StartupCrop {
    fn default() -> Self {
        Self {
            enabled: false,
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
        conf.startup_file = cli.path.clone();

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
