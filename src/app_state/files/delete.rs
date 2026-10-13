use std::fs;
use tracing::info;

use crate::app_state::{
    AppState,
    files::{image_source::ImgError, navigation::get_next_image},
};

impl AppState {
    pub fn delete_current_file(&mut self) {
        if self.folder.is_some() {
            let current_path = self.current_path();
            if current_path.as_os_str().is_empty() {
                return;
            }
            let canonical_current = current_path
                .canonicalize()
                .unwrap_or_else(|_| current_path.clone());

            let images = self.get_folder_images();
            let next = if images.len() > 1 {
                get_next_image(&canonical_current, images)
            } else {
                None
            };

            if fs::remove_file(&current_path).is_ok() {
                info!("Deleted file {:?}", current_path);
                self.ui_state.notifications.push("File deleted", false);
                if let Some(ref mut cache) = self.folder_cache {
                    cache.retain(|p| p != &canonical_current && p != &current_path);
                }
                if let Some(next) = next {
                    self.try_open_file(&next);
                    self.set_src_dirty();
                } else {
                    self.source = Err(ImgError::NoImage);
                    self.set_src_dirty();
                }
            } else {
                self.ui_state
                    .notifications
                    .push("Failed to delete file", true);
            }
        }
    }
}
