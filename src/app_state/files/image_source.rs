use anyhow::Result;
use image::{ImageBuffer, ImageReader, Rgba};
use std::path::PathBuf;

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
    #[tracing::instrument]
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
