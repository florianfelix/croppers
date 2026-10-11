use std::fs;
use tracing::info;

use crate::app_state::{
    AppState,
    files::navigation::{get_images_in_folder, get_next_image},
};

impl AppState {
    pub fn delete_current_file(&mut self) {
        if let Some(ref folder) = self.folder {
            if let Ok(ref source) = self.source {
                let images = get_images_in_folder(folder);
                let next = get_next_image(&source.path.canonicalize().unwrap(), &images);

                if fs::remove_file(&source.path).is_ok() {
                    info!("Deleted file");
                    self.ui_state.notifications.push("File deleted", false);
                    if let Some(next) = next {
                        self.try_open_file(&next);
                        self.set_src_dirty();
                    }
                }
            }
        }
    }
}
