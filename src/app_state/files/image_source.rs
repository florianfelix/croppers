use image::{ImageBuffer, ImageReader, Rgba};
use std::path::PathBuf;
use tracing::{error, info, warn};

use crate::app_state::GpuHandles;

pub type ImgResult<T> = std::result::Result<T, ImgError>;

#[derive(Debug)]
pub enum ImgError {
    NoImage,
    ImageError(image::ImageError),
    FailedToLoad {
        path: PathBuf,
    },
    Io(std::io::Error),
    WrongDimensions {
        path: PathBuf,
        dims: [u32; 2],
        max_dims: u32,
    },
}
impl std::fmt::Display for ImgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImgError::NoImage => write!(f, "No image provided"),
            ImgError::ImageError(err) => write!(f, "Image error: {}", err),
            ImgError::FailedToLoad { path } => {
                write!(f, "Failed to load image from path: {}", path.display())
            }
            ImgError::Io(err) => write!(f, "IO error: {}", err),
            ImgError::WrongDimensions {
                path,
                dims,
                max_dims,
            } => {
                write!(
                    f,
                    "Image at path {} has wrong dimensions: {:?}. Max: {:?}",
                    path.display(),
                    dims,
                    max_dims
                )
            }
        }
    }
}
impl std::error::Error for ImgError {}
impl From<std::io::Error> for ImgError {
    fn from(err: std::io::Error) -> Self {
        ImgError::Io(err)
    }
}
impl From<image::ImageError> for ImgError {
    fn from(err: image::ImageError) -> Self {
        ImgError::ImageError(err)
    }
}
impl std::fmt::Debug for ImageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\n{}", self.path, self.name)
    }
}

#[derive(Clone)]
pub struct ImageSource {
    pub path: PathBuf,
    pub name: String,
    pub rgba: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub image_size: (u32, u32),
}

impl ImageSource {
    #[tracing::instrument(skip(gpu))]
    pub fn from_path(path: &PathBuf, gpu: Option<GpuHandles>) -> ImgResult<Self> {
        let img = ImageReader::open(path)?.decode()?;
        let name = match path.file_name() {
            Some(osstr) => osstr.to_string_lossy().to_string(),
            None => String::new(),
        };
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        info!(
            "Loaded image {:?} with dimensions {}x{}",
            path, width, height
        );
        if let Some(gpu) = gpu {
            if !(width > 0
                && height > 0
                && width <= gpu.dims_max / 100
                && height <= gpu.dims_max / 100)
            {
                error!(
                    "Image dimensions {}x{} exceed GPU max dimensions {}",
                    width,
                    height,
                    gpu.dims_max / 100
                );
                return Err(ImgError::WrongDimensions {
                    path: path.clone(),
                    dims: [width, height],
                    max_dims: gpu.dims_max,
                });
            }
        } else {
            warn!("No GPU set yet")
        }
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
