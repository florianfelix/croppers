use std::{fs, path::PathBuf};
use tracing::info;

use crate::app_state::AppState;

impl AppState {
    #[tracing::instrument(skip(self))]
    pub fn try_open_file(&mut self, path: &PathBuf) {
        if let Ok(abspath) = fs::canonicalize(path)
            && path.is_file()
        {
            let abs_folder = abspath.parent().map(|p| p.to_owned());
            if self.folder != abs_folder {
                self.folder = abs_folder;
                self.invalidate_folder_cache();
            } else if let Some(ref mut cache) = self.folder_cache {
                if !cache.contains(&abspath) {
                    cache.push(abspath);
                    cache.sort();
                }
            }
        } else {
            self.ui_state
                .notifications
                .push(&format!("{:?} not found", path), true);
            self.invalidate_folder_cache();
            return;
        }

        self.load_id += 1;
        let id = self.load_id;
        self.target_path = Some(path.clone());

        let dims_max = self.gpu.as_ref().map(|g| g.dims_max).unwrap_or(8192);

        let _ = self.tx_job.send(crate::compute::ComputeRequest::LoadImage {
            id,
            path: path.clone(),
            dims_max,
        });
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
