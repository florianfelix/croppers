use anyhow::Result;
use image::{ImageBuffer, ImageReader, Rgba};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

use crate::app_state::AppState;

#[derive(Clone)]
pub struct ImageSource {
    pub path: PathBuf,
    pub name: String,
    pub rgba: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub image_size: (u32, u32),
}
impl std::fmt::Debug for ImageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\n{}", self.path, self.name)
    }
}

impl ImageSource {
    pub fn from_path(path: &PathBuf) -> Result<Self> {
        let img = ImageReader::open(path)?.decode()?;
        let name = match path.file_name() {
            Some(osstr) => osstr.to_string_lossy().to_string(),
            None => String::new(),
        };
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let image_size = (width, height);
        Ok(Self {
            path: path.to_owned(),
            name,
            rgba,
            image_size,
        })
    }
    pub fn from_bytes(name: &str, bytes: Vec<u8>) -> Self {
        let img = image::load_from_memory(&bytes).unwrap();
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Self {
            path: PathBuf::new(),
            name: String::from(name),
            rgba,
            image_size: (width, height),
        }
    }
}

impl AppState {
    #[tracing::instrument(skip(self))]
    pub fn open_next(&mut self) {
        if self.try_open_next().is_ok() {
            self.source_dirty = true;
        }
        self.set_srcimg_texture_from_source();
        // self.set_srcimg_to_display();
    }

    pub fn open_previous(&mut self) {
        if self.try_open_prev().is_ok() {
            self.source_dirty = true;
        }
        self.set_srcimg_texture_from_source();
        // self.set_srcimg_to_display();
    }
    pub fn set_src_dirty(&mut self) {
        self.source_dirty = true;
        self.set_srcimg_texture_from_source();
    }

    pub fn try_open_file(&mut self, path: &PathBuf) {
        if let Ok(abspath) = fs::canonicalize(path)
            && path.is_file()
        {
            let abs_folder = abspath.parent().map(|p| p.to_owned());
            self.folder = abs_folder
        }
        if let Ok(source) = ImageSource::from_path(path) {
            info!("Setting new Source Image {:?}", path);
            self.source = Some(source);
        }
    }

    pub fn try_open_next(&mut self) -> Result<()> {
        #[allow(clippy::collapsible_if)]
        if let Some(ref folder) = self.folder {
            if let Some(ref source) = self.source {
                let images = get_images_in_folder(folder);
                let next = get_next_image(&source.path.canonicalize().unwrap(), &images);
                if let Some(next) = next {
                    // info!("NEXT {:?}", next);
                    self.try_open_file(&next);
                }
            }
        }
        Ok(())
    }
    pub fn try_open_prev(&mut self) -> Result<()> {
        #[allow(clippy::collapsible_if)]
        if let Some(ref folder) = self.folder {
            if let Some(ref source) = self.source {
                let images = get_images_in_folder(folder);
                let next = get_previous_image(&source.path.canonicalize().unwrap(), &images);
                if let Some(next) = next {
                    info!("NEXT {:?}", next);
                    self.try_open_file(&next);
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn save_effect_texture(&mut self, increment: bool) {
        if self.texture_vault.effect_view.is_some() {
            if let Some(ref source) = self.source {
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
        if let Some(ref source) = self.source {
            let is_overwrite = source.path.canonicalize().unwrap() == path;
            info!("is_overwrite {is_overwrite}");
            if is_overwrite {
                self.try_open_file(&path);
                self.effect_stack.enabled = false;
            }
        }
    }
    pub fn delete_current_file(&mut self) {
        if let Some(ref folder) = self.folder {
            if let Some(ref source) = self.source {
                let images = get_images_in_folder(folder);
                let next = get_next_image(&source.path.canonicalize().unwrap(), &images);

                if fs::remove_file(&source.path).is_ok() {
                    info!("Deleted file");
                    if let Some(next) = next {
                        self.try_open_file(&next);
                        self.set_src_dirty();
                    }
                }
            }
        }
    }
}

fn get_images_in_folder<P: AsRef<Path>>(folder_path: P) -> Vec<PathBuf> {
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

fn get_next_image(current_image: &Path, image_list: &[PathBuf]) -> Option<PathBuf> {
    let current_index = image_list.iter().position(|p| p == current_image)?;

    let next_index = (current_index + 1) % image_list.len();

    Some(image_list[next_index].clone())
}

fn get_previous_image(current_image: &Path, image_list: &[PathBuf]) -> Option<PathBuf> {
    let current_index = image_list.iter().position(|p| p == current_image)?;
    let prev_index = (current_index + image_list.len() - 1) % image_list.len();
    Some(image_list[prev_index].clone())
}

fn increment_filename(path: &Path) -> PathBuf {
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
