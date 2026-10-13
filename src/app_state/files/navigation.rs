use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

use crate::app_state::AppState;

impl AppState {
    pub fn invalidate_folder_cache(&mut self) {
        self.folder_cache = None;
    }

    pub fn get_folder_images(&mut self) -> &[PathBuf] {
        let Some(ref folder) = self.folder else {
            return &[];
        };
        if self.folder_cache.is_none() {
            self.folder_cache = Some(get_images_in_folder(folder));
        }
        self.folder_cache.as_deref().unwrap_or(&[])
    }

    #[tracing::instrument(skip(self))]
    pub fn open_next(&mut self) {
        let _ = self.try_open_next();
    }

    pub fn open_previous(&mut self) {
        let _ = self.try_open_prev();
    }

    pub fn current_path(&self) -> PathBuf {
        if let Some(ref target) = self.target_path {
            return target.clone();
        }
        if let Some(ref folder) = self.folder {
            match self.source.as_ref() {
                Ok(source) => source.path.clone(),
                Err(imgerror) => {
                    if let Some(path) = imgerror.path() {
                        path.to_owned()
                    } else if let Some(ref imgs) = self.folder_cache {
                        imgs.last().cloned().unwrap_or_default()
                    } else {
                        let imgs = get_images_in_folder(folder);
                        if let Some(p) = imgs.last() {
                            p.to_owned()
                        } else {
                            PathBuf::new()
                        }
                    }
                }
            }
        } else {
            PathBuf::new()
        }
    }

    pub fn try_open_next(&mut self) -> Result<()> {
        let current = self.current_path();
        let current_path = current.canonicalize().unwrap_or(current);
        let images = self.get_folder_images();

        let next = get_next_image(&current_path, images);
        if let Some(next) = next {
            self.try_open_file(&next);
        }
        Ok(())
    }

    pub fn try_open_prev(&mut self) -> Result<()> {
        let current = self.current_path();
        let current_path = current.canonicalize().unwrap_or(current);
        let images = self.get_folder_images();
        let next = get_previous_image(&current_path, images);
        if let Some(next) = next {
            info!("NEXT {:?}", next);
            self.try_open_file(&next);
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
                        images.push(path.canonicalize().unwrap_or(path));
                    }
                }
            }
        }
    }

    images.sort();
    images
}

pub(crate) fn get_next_image(current_image: &Path, image_list: &[PathBuf]) -> Option<PathBuf> {
    if image_list.is_empty() {
        return None;
    }
    let current_index = image_list.iter().position(|p| p == current_image);
    let next_index = match current_index {
        Some(idx) => (idx + 1) % image_list.len(),
        None => 0,
    };

    Some(image_list[next_index].clone())
}

pub(crate) fn get_previous_image(current_image: &Path, image_list: &[PathBuf]) -> Option<PathBuf> {
    if image_list.is_empty() {
        return None;
    }
    let current_index = image_list.iter().position(|p| p == current_image);
    let prev_index = match current_index {
        Some(idx) => (idx + image_list.len() - 1) % image_list.len(),
        None => image_list.len() - 1,
    };
    Some(image_list[prev_index].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_and_previous_image() {
        let p1 = PathBuf::from("/a/1.png");
        let p2 = PathBuf::from("/a/2.png");
        let p3 = PathBuf::from("/a/3.png");
        let list = vec![p1.clone(), p2.clone(), p3.clone()];

        assert_eq!(get_next_image(&p1, &list), Some(p2.clone()));
        assert_eq!(get_next_image(&p2, &list), Some(p3.clone()));
        assert_eq!(get_next_image(&p3, &list), Some(p1.clone()));

        assert_eq!(get_previous_image(&p1, &list), Some(p3.clone()));
        assert_eq!(get_previous_image(&p2, &list), Some(p1.clone()));
        assert_eq!(get_previous_image(&p3, &list), Some(p2.clone()));
    }

    #[test]
    fn test_get_next_and_previous_image_unlisted() {
        let p1 = PathBuf::from("/a/1.png");
        let p2 = PathBuf::from("/a/2.png");
        let list = vec![p1.clone(), p2.clone()];
        let unknown = PathBuf::from("/a/unknown.png");

        assert_eq!(get_next_image(&unknown, &list), Some(p1.clone()));
        assert_eq!(get_previous_image(&unknown, &list), Some(p2.clone()));
    }

    #[test]
    fn test_get_next_and_previous_image_empty() {
        let list = vec![];
        let p1 = PathBuf::from("/a/1.png");
        assert_eq!(get_next_image(&p1, &list), None);
        assert_eq!(get_previous_image(&p1, &list), None);
    }

    #[test]
    fn test_folder_cache_and_invalidation() {
        let temp_dir = std::env::temp_dir().join("croppers_cache_test");
        let _ = fs::create_dir_all(&temp_dir);
        let img1 = temp_dir.join("img1.png");
        let img2 = temp_dir.join("img2.png");
        fs::write(&img1, b"fake").unwrap();
        fs::write(&img2, b"fake").unwrap();

        let mut state = AppState::new(crate::utils::CroppersConfig::default());
        assert!(state.folder_cache.is_none());

        state.folder = Some(temp_dir.clone());
        let images = state.get_folder_images();
        assert_eq!(images.len(), 2);
        assert!(state.folder_cache.is_some());

        // Cache hit (cached images match)
        assert_eq!(state.get_folder_images().len(), 2);

        // Invalidate
        state.invalidate_folder_cache();
        assert!(state.folder_cache.is_none());

        // Re-populates on demand
        assert_eq!(state.get_folder_images().len(), 2);

        // Clean up
        let _ = fs::remove_file(img1);
        let _ = fs::remove_file(img2);
        let _ = fs::remove_dir(temp_dir);
    }
}
