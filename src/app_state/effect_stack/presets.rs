use directories::ProjectDirs;
use effects::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
            selected_preset: None,
            preset_name_input: String::new(),
        };
        stack.validate_ids();
        stack.refresh_presets();
        stack
    }

    pub fn presets_directory() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("de", "ff", "croppers")?;
        let presets_dir = proj_dirs.config_dir().join("presets");
        if !presets_dir.exists() {
            std::fs::create_dir_all(&presets_dir).ok()?;
        }
        Some(presets_dir)
    }

    pub fn save_to_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let saved = self.to_saved_stack();
        let pretty = ron::ser::PrettyConfig::default();
        let ron_str = ron::ser::to_string_pretty(&saved, pretty)?;
        std::fs::write(path, ron_str)?;
        self.refresh_presets();
        Ok(())
    }

    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let saved: SavedEffectStack = ron::from_str(&content)?;
        Ok(Self::from_saved_stack(saved))
    }

    pub fn load_preset_by_path(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let saved: SavedEffectStack = ron::from_str(&content)?;
        self.enabled = saved.enabled;
        self.effects = saved.effects.into_iter().map(Into::into).collect();
        self.needs_rebuild = true;
        self.validate_ids();
        Ok(())
    }

    pub fn refresh_presets(&mut self) {
        if let Some(presets_dir) = Self::presets_directory() {
            if let Ok(entries) = std::fs::read_dir(presets_dir) {
                let mut presets: Vec<PathBuf> = entries
                    .flatten()
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "ron") {
                            Some(path)
                        } else {
                            None
                        }
                    })
                    .collect();
                presets.sort();
                self.presets = presets;
                return;
            }
        }
        self.presets = vec![];
    }

    /// Renders UI controls for selecting, loading, saving, and deleting effect stack presets.
    /// Returns `true` if a preset was loaded and needs a GPU rebuild/recompute.
    pub fn draw_presets_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.collapsing("Presets", |ui| {
            let selected_label = self
                .selected_preset
                .as_ref()
                .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
                .unwrap_or("Select preset...")
                .to_string();

            let mut preset_to_select = None;
            let mut do_refresh = false;

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("effect_presets_combo")
                    .selected_text(&selected_label)
                    .width(140.0)
                    .show_ui(ui, |ui| {
                        for path in &self.presets {
                            let stem = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown");
                            let is_selected = self.selected_preset.as_ref() == Some(path);
                            if ui.selectable_label(is_selected, stem).clicked() {
                                preset_to_select = Some(path.clone());
                            }
                        }
                    });

                if ui
                    .small_button("⟳")
                    .on_hover_text("Refresh presets")
                    .clicked()
                {
                    do_refresh = true;
                }
            });

            if let Some(path) = preset_to_select {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    self.preset_name_input = stem.to_string();
                }
                self.selected_preset = Some(path);
            }

            if do_refresh {
                self.refresh_presets();
            }

            let mut do_load = false;
            let mut do_delete = false;

            ui.horizontal(|ui| {
                let has_selection = self.selected_preset.is_some();
                if ui
                    .add_enabled(has_selection, egui::Button::new("Load"))
                    .on_hover_text("Load selected preset")
                    .clicked()
                {
                    do_load = true;
                }

                if ui
                    .add_enabled(has_selection, egui::Button::new("Delete"))
                    .on_hover_text("Delete selected preset file")
                    .clicked()
                {
                    do_delete = true;
                }
            });

            if do_load {
                if let Some(ref path) = self.selected_preset.clone() {
                    if let Err(err) = self.load_preset_by_path(path) {
                        tracing::error!("Failed to load preset: {err}");
                    } else {
                        changed = true;
                    }
                }
            }

            if do_delete {
                if let Some(ref path) = self.selected_preset.take() {
                    let _ = std::fs::remove_file(path);
                    self.refresh_presets();
                }
            }

            ui.separator();

            let mut do_save = false;

            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.preset_name_input)
                        .hint_text("Preset name")
                        .desired_width(120.0),
                );

                let trimmed = self.preset_name_input.trim();
                let can_save = !trimmed.is_empty();

                if ui
                    .add_enabled(can_save, egui::Button::new("Save"))
                    .on_hover_text("Save current effect stack as preset")
                    .clicked()
                {
                    do_save = true;
                }
            });

            if do_save {
                let trimmed = self.preset_name_input.trim().to_string();
                if !trimmed.is_empty() {
                    if let Some(presets_dir) = Self::presets_directory() {
                        let filename = format!("{trimmed}.ron");
                        let path = presets_dir.join(filename);
                        if let Err(err) = self.save_to_file(&path) {
                            tracing::error!("Failed to save preset: {err}");
                        } else {
                            self.selected_preset = Some(path);
                        }
                    }
                }
            }
        });

        changed
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
        uniforms: BlurUniforms,
    },
    Vignette {
        enabled: bool,
        uniforms: VignetteUniforms,
    },
    Bw {
        enabled: bool,
        uniforms: BwUniforms,
    },
    Hsv {
        enabled: bool,
        uniforms: HsvUniforms,
    },
    Gamma {
        enabled: bool,
        uniforms: GammaUniforms,
    },
    TiltShift {
        enabled: bool,
        uniforms: TiltShiftUniforms,
    },
    PixelateTilt {
        enabled: bool,
        uniforms: PixelateTiltUniforms,
    },
    PixelateLinear {
        enabled: bool,
        uniforms: PixelateLinearUniforms,
    },
    Dither {
        enabled: bool,
        uniforms: ColorDitherUniforms,
    },
    Crt {
        enabled: bool,
        uniforms: CrtUniforms,
    },
    Ascii {
        enabled: bool,
        uniforms: AsciiUniforms,
    },
    Vcr {
        enabled: bool,
        uniforms: VcrUniforms,
    },
    Glitch {
        enabled: bool,
        uniforms: GlitchUniforms,
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
