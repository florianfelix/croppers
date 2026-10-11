use std::path::{Path, PathBuf};
use tracing::info;

use crate::app_state::AppState;

impl AppState {
    pub fn save_keyboard_shortcut(&mut self, increment: bool) {
        if self.ui_state.crop.enabled {
            self.ui_state
                .notifications
                .push("Cannot save while crop mode is active.", true);
        } else {
            self.save_effect_texture(increment);
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn save_effect_texture(&mut self, increment: bool) {
        if self.texture_vault.effect.texture_view.is_some() {
            if let Ok(ref source) = self.source {
                if let Ok(ref current) = source.path.canonicalize() {
                    let path_for_saving = if increment {
                        increment_filename(current)
                    } else {
                        current.clone()
                    };
                    if self
                        .tx_job
                        .send(crate::compute::ComputeRequest::SaveLastEffect(
                            path_for_saving.clone(),
                        ))
                        .is_ok()
                    {
                        info!("Send SaveJob {:?}", path_for_saving);
                    }
                };
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn action_on_save(&mut self, path: PathBuf) {
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        self.ui_state
            .notifications
            .push(&format!("Saved: {}", filename), false);

        if let Ok(ref source) = self.source {
            let is_overwrite = source.path.canonicalize().ok().as_ref() == Some(&path);
            info!("is_overwrite {is_overwrite}");
            if is_overwrite {
                self.try_open_file(&path);
                self.effect_stack.enabled = false;
            }
        }
    }
}

pub(crate) fn increment_filename(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    // Extract the parent directory, file stem (name without extension), and the extension
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|e| e.to_str());

    let mut counter = 1;

    loop {
        let new_filename = format!("{}_{:03}", file_stem, counter);

        let mut new_path = parent.join(new_filename);

        if let Some(ext) = extension {
            new_path.set_extension(ext);
        }

        if !new_path.exists() {
            return new_path;
        }

        counter += 1;
    }
}
