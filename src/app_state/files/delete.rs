use std::fs;
use tracing::info;

use crate::app_state::{
    AppState,
    files::{
        image_source::ImgError,
        navigation::{get_images_in_folder, get_next_image},
    },
};

impl AppState {
    pub fn delete_current_file(&mut self) {
        if let Some(ref folder) = self.folder {
            let current_path = self.current_path();
            if current_path.as_os_str().is_empty() {
                return;
            }
            let images = get_images_in_folder(folder);
            let canonical_current = current_path
                .canonicalize()
                .unwrap_or_else(|_| current_path.clone());
            let next = if images.len() > 1 {
                get_next_image(&canonical_current, &images)
            } else {
                None
            };

            if fs::remove_file(&current_path).is_ok() {
                info!("Deleted file {:?}", current_path);
                self.ui_state.notifications.push("File deleted", false);
                if let Some(next) = next {
                    self.try_open_file(&next);
                    self.set_src_dirty();
                } else {
                    self.source = Err(ImgError::NoImage);
                    self.set_src_dirty();
                }
            }
        }
    }
}
