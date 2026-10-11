use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

use crate::app_state::{AppState, files::image_source::ImgError};

impl AppState {
    #[tracing::instrument(skip(self))]
    pub fn open_next(&mut self) {
        if self.try_open_next().is_ok() {
            self.source_dirty = true;
        }
        self.set_srcimg_texture_from_source();
    }

    pub fn open_previous(&mut self) {
        if self.try_open_prev().is_ok() {
            self.source_dirty = true;
        }
        self.set_srcimg_texture_from_source();
    }

    pub fn current_path(&self) -> PathBuf {
        if let Some(ref folder) = self.folder {
            match self.source.as_ref() {
                Ok(source) => source.path.clone(),
                Err(imgerror) => match imgerror {
                    ImgError::FailedToLoad { path } => path.to_owned(),
                    ImgError::WrongDimensions { path, .. } => path.to_owned(),
                    _ => {
                        let imgs = get_images_in_folder(folder);
                        if let Some(p) = imgs.last() {
                            p.to_owned()
                        } else {
                            PathBuf::new()
                        }
                    }
                },
            }
        } else {
            PathBuf::new()
        }
    }

    pub fn try_open_next(&mut self) -> Result<()> {
        #[allow(clippy::collapsible_if)]
        if let Some(ref folder) = self.folder {
            let current_path = self.current_path().canonicalize()?;

            // if let Ok(ref source) = self.source {
            let images = get_images_in_folder(folder);

            let next = get_next_image(&current_path, &images);
            if let Some(next) = next {
                self.try_open_file(&next);
            }
            // }
        }
        Ok(())
    }

    pub fn try_open_prev(&mut self) -> Result<()> {
        #[allow(clippy::collapsible_if)]
        if let Some(ref folder) = self.folder {
            // if let Ok(ref source) = self.source {
            let current_path = self.current_path().canonicalize()?;
            let images = get_images_in_folder(folder);
            let next = get_previous_image(&current_path, &images);
            if let Some(next) = next {
                info!("NEXT {:?}", next);
                self.try_open_file(&next);
            }
            // }
        }
        Ok(())
    }
}

pub(crate) fn get_images_in_folder<P: AsRef<Path>>(folder_path: P) -> Vec<PathBuf> {
    let valid_extensions = ["jpg", "jpeg", "png", "webp"];
    let mut images = Vec::new();

    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            #[allow(clippy::collapsible_if)]
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if valid_extensions.contains(&ext.to_lowercase().as_str()) {
                        images.push(path);
                    }
                }
            }
        }
    }

    images.sort();
    images
}

pub(crate) fn get_next_image(current_image: &Path, image_list: &[PathBuf]) -> Option<PathBuf> {
    let current_index = image_list.iter().position(|p| p == current_image)?;

    let next_index = (current_index + 1) % image_list.len();

    Some(image_list[next_index].clone())
}

pub(crate) fn get_previous_image(current_image: &Path, image_list: &[PathBuf]) -> Option<PathBuf> {
    let current_index = image_list.iter().position(|p| p == current_image)?;
    let prev_index = (current_index + image_list.len() - 1) % image_list.len();
    Some(image_list[prev_index].clone())
}
