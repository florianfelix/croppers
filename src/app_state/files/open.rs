use std::{fs, path::PathBuf};
use tracing::{error, info};

use crate::app_state::{AppState, files::ImageSource};

impl AppState {
    pub fn try_open_file(&mut self, path: &PathBuf) {
        if let Ok(abspath) = fs::canonicalize(path)
            && path.is_file()
        {
            let abs_folder = abspath.parent().map(|p| p.to_owned());
            self.folder = abs_folder;
        } else {
            self.ui_state
                .notifications
                .push(&format!("{:?} not found", path), true);
        }
        match ImageSource::from_path(path) {
            Ok(source) => {
                info!("Setting new Source Image {:?}", path);
                self.source = Some(source);
            }
            Err(e) => {
                let msg = format!("Failed to load image {e}");
                error!(msg);
                self.ui_state.notifications.push(&msg, true);
            }
        }
    }

    pub fn set_src_dirty(&mut self) {
        self.source_dirty = true;
        self.set_srcimg_texture_from_source();
    }
}
