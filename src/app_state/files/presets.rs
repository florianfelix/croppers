use directories::ProjectDirs;

use crate::app_state::effect_stack::{EffectStack, SavedEffectStack};

impl EffectStack {
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let saved = self.to_saved_stack();
        let ron_or_json = toml::to_string_pretty(&saved)?; // or serde_json / ron
        std::fs::write(path, ron_or_json)?;
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
