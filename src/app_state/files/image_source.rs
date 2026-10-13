use image::{ImageBuffer, ImageReader, Rgba};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::app_state::GpuHandles;

pub type ImgResult<T> = std::result::Result<T, ImgError>;

#[derive(Debug, Clone)]
pub enum ImgError {
    NoImage,
    Io {
        path: PathBuf,
        err: Arc<std::io::Error>,
    },
    Decode {
        path: PathBuf,
        err: Arc<image::ImageError>,
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

#[derive(Clone, Debug)]
pub struct LoadedImage {
    pub source: ImageSource,
    pub texture: Arc<wgpu::Texture>,
    pub texture_view: Arc<wgpu::TextureView>,
    pub dims: [u32; 2],
}

impl ImageSource {
    #[tracing::instrument(skip(gpu))]
    pub fn from_path(path: &PathBuf, gpu: Option<GpuHandles>) -> ImgResult<Self> {
        let reader = ImageReader::open(path).map_err(|err| ImgError::Io {
            path: path.clone(),
            err: Arc::new(err),
        })?;
        let img = reader.decode().map_err(|err| ImgError::Decode {
            path: path.clone(),
            err: Arc::new(err),
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

    #[tracing::instrument(skip(device, queue))]
    pub fn load_and_upload(
        path: &Path,
        dims_max: u32,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> ImgResult<LoadedImage> {
        let reader = ImageReader::open(path).map_err(|err| ImgError::Io {
            path: path.to_path_buf(),
            err: Arc::new(err),
        })?;
        let img = reader.decode().map_err(|err| ImgError::Decode {
            path: path.to_path_buf(),
            err: Arc::new(err),
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

        if !(width > 0 && height > 0 && width <= dims_max && height <= dims_max) {
            error!(
                "Image dimensions {}x{} exceed GPU max dimensions {}",
                width, height, dims_max
            );
            return Err(ImgError::WrongDimensions {
                path: path.to_path_buf(),
                dims: [width, height],
                max_dims: dims_max,
            });
        }

        let source = Self {
            path: path.to_path_buf(),
            name,
            rgba,
            image_size: (width, height),
        };

        let (texture, texture_view, dims) =
            crate::app_state::textures::texture_and_view_from_source_img(&source, device, queue);

        Ok(LoadedImage {
            source,
            texture,
            texture_view,
            dims,
        })
    }
}
