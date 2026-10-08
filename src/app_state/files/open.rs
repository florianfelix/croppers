use std::{fs, path::PathBuf};
use tracing::{error, info};

use crate::app_state::{AppState, files::ImageSource};

impl AppState {
    #[tracing::instrument(skip(self))]
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
                self.set_src_dirty();
            }
            Err(e) => {
                let msg = format!("Failed to load image {e}");
                error!(msg);
                self.ui_state.notifications.push(&msg, true);
            }
        }
    }

    pub fn open_file_dialog(&self) {
        if let Some(proxy) = &self.proxy {
            let proxy = proxy.clone();
            let mut dialog =
                rfd::FileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "webp"]);

            if let Some(folder) = &self.folder {
                dialog = dialog.set_directory(folder);
            }

            std::thread::spawn(move || {
                if let Some(path) = dialog.pick_file() {
                    let _ = proxy.send_event(crate::Message::FileOpened(path));
                }
            });
        }
    }

    pub fn set_src_dirty(&mut self) {
        info!("Setting source dirty");
        self.source_dirty = true;
        self.set_srcimg_texture_from_source();
    }
}
