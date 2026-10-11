use image::{ImageBuffer, ImageReader, Rgba};
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

use crate::app_state::GpuHandles;

pub type ImgResult<T> = std::result::Result<T, ImgError>;

#[derive(Debug)]
pub enum ImgError {
    NoImage,
    Io {
        path: PathBuf,
        err: std::io::Error,
    },
    Decode {
        path: PathBuf,
        err: image::ImageError,
    },
    WrongDimensions {
        path: PathBuf,
        dims: [u32; 2],
        max_dims: u32,
    },
}

impl ImgError {
    pub fn path(&self) -> Option<&Path> {
        match self {
            ImgError::NoImage => None,
            ImgError::Io { path, .. } => Some(path),
            ImgError::Decode { path, .. } => Some(path),
            ImgError::WrongDimensions { path, .. } => Some(path),
        }
    }
}

impl std::fmt::Display for ImgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImgError::NoImage => write!(f, "No image loaded"),
            ImgError::Io { path, err } => {
                write!(f, "Failed to read {}: {}", path.display(), err)
            }
            ImgError::Decode { path, err } => {
                write!(f, "Failed to decode {}: {}", path.display(), err)
            }
            ImgError::WrongDimensions {
                path,
                dims,
                max_dims,
            } => {
                write!(
                    f,
                    "Image {} has dimensions {}x{}, exceeding GPU maximum ({}x{})",
                    path.display(),
                    dims[0],
                    dims[1],
                    max_dims,
                    max_dims
                )
            }
        }
    }
}

impl std::error::Error for ImgError {}

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
        let reader = ImageReader::open(path).map_err(|err| ImgError::Io {
            path: path.clone(),
            err,
        })?;
        let img = reader.decode().map_err(|err| ImgError::Decode {
            path: path.clone(),
            err,
        })?;
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
            if !(width > 0 && height > 0 && width <= gpu.dims_max && height <= gpu.dims_max) {
                error!(
                    "Image dimensions {}x{} exceed GPU max dimensions {}",
                    width, height, gpu.dims_max
                );
                return Err(ImgError::WrongDimensions {
                    path: path.clone(),
                    dims: [width, height],
                    max_dims: gpu.dims_max,
                });
            }
        } else {
            warn!("No GPU set yet");
        }
        let image_size = (width, height);
        Ok(Self {
            path: path.to_owned(),
            name,
            rgba,
            image_size,
        })
    }
}
