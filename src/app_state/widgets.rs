use serde::{Deserialize, Serialize};

pub mod drag_target;
pub mod rect_helper;
pub mod crop_widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropMode {
    Freeform,
    FixedDimensions,
    OriginalRatio,
    Ratio1x1,
    Ratio16x9,
    Ratio4x3,
    Ratio3x2,
    CustomRatio,
}

impl CropMode {
    pub fn label(&self) -> &'static str {
        match self {
            CropMode::Freeform => "Freeform",
            CropMode::FixedDimensions => "Fixed Target Dimensions",
            CropMode::OriginalRatio => "Original Ratio",
            CropMode::Ratio1x1 => "1:1 (Square)",
            CropMode::Ratio16x9 => "16:9",
            CropMode::Ratio4x3 => "4:3",
            CropMode::Ratio3x2 => "3:2",
            CropMode::CustomRatio => "Custom Ratio",
        }
    }

    pub const ALL: [CropMode; 8] = [
        CropMode::Freeform,
        CropMode::FixedDimensions,
        CropMode::OriginalRatio,
        CropMode::Ratio1x1,
        CropMode::Ratio16x9,
        CropMode::Ratio4x3,
        CropMode::Ratio3x2,
        CropMode::CustomRatio,
    ];
}

#[derive(Clone)]
pub struct CropSettings {
    pub enabled: bool,
    pub mode: CropMode,
    pub crop_width: u32,
    pub crop_height: u32,
    pub custom_ratio: [u32; 2],
    pub uv_rect: egui::Rect,
}

impl Default for CropSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: CropMode::Freeform,
            crop_width: 3440,
            crop_height: 1440,
            custom_ratio: [16, 9],
            uv_rect: egui::Rect {
                min: egui::Pos2::ZERO,
                max: egui::Pos2 { x: 1.0, y: 1.0 },
            },
        }
    }
}

impl CropSettings {
    pub fn aspect_ratio(&self, image_size: [u32; 2]) -> Option<f32> {
        match self.mode {
            CropMode::Freeform => None,
            CropMode::FixedDimensions => {
                if self.crop_height > 0 {
                    Some(self.crop_width as f32 / self.crop_height as f32)
                } else {
                    None
                }
            }
            CropMode::OriginalRatio => {
                if image_size[1] > 0 {
                    Some(image_size[0] as f32 / image_size[1] as f32)
                } else {
                    None
                }
            }
            CropMode::Ratio1x1 => Some(1.0),
            CropMode::Ratio16x9 => Some(16.0 / 9.0),
            CropMode::Ratio4x3 => Some(4.0 / 3.0),
            CropMode::Ratio3x2 => Some(3.0 / 2.0),
            CropMode::CustomRatio => {
                if self.custom_ratio[1] > 0 {
                    Some(self.custom_ratio[0] as f32 / self.custom_ratio[1] as f32)
                } else {
                    None
                }
            }
        }
    }

    pub fn target_dimensions(&self, src_dimensions: [u32; 2]) -> [u32; 2] {
        match self.mode {
            CropMode::FixedDimensions => [self.crop_width.max(1), self.crop_height.max(1)],
            _ => {
                let w = ((src_dimensions[0] as f32) * self.uv_rect.width())
                    .round()
                    .max(1.0) as u32;
                let h = ((src_dimensions[1] as f32) * self.uv_rect.height())
                    .round()
                    .max(1.0) as u32;
                [w, h]
            }
        }
    }
}
