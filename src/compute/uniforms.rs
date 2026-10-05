use crate::app_state::effect_stack::{
    BwSettings, GammaSettings, HsvSettings, PixelateTiltSettings, TiltShiftSettings,
    VignetteSettings,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // or bytemuck::Pod, bytemuck::Zeroable
pub struct BlurUniforms {
    pub radius: f32,
    pub dir_x: i32,
    pub dir_y: i32,
    pub _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // or bytemuck::Pod, bytemuck::Zeroable
pub struct VignetteUniforms {
    pub intensity: f32,
    pub radius: f32,
    pub smoothness: f32,
    pub _padding: i32,
}
impl From<VignetteSettings> for VignetteUniforms {
    fn from(value: VignetteSettings) -> Self {
        Self {
            intensity: value.intensity,
            radius: value.radius,
            smoothness: value.smoothness,
            _padding: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // or bytemuck::Pod, bytemuck::Zeroable
pub struct BwUniforms {
    pub select: i32,
    pub _padding1: i32,
    pub _padding2: i32,
    pub _padding3: i32,
}
impl From<BwSettings> for BwUniforms {
    fn from(value: BwSettings) -> Self {
        Self {
            select: value.select,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // or bytemuck::Pod, bytemuck::Zeroable
pub struct HsvUniforms {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub factor: f32,
}
impl From<HsvSettings> for HsvUniforms {
    fn from(value: HsvSettings) -> Self {
        Self {
            hue: value.hue,
            saturation: value.saturation,
            value: value.value,
            factor: value.factor,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // or bytemuck::Pod, bytemuck::Zeroable
pub struct GammaUniforms {
    pub gr: f32,
    pub gg: f32,
    pub gb: f32,
    pub _padding1: i32,
}
impl From<GammaSettings> for GammaUniforms {
    fn from(value: GammaSettings) -> Self {
        Self {
            gr: value.gr,
            gg: value.gg,
            gb: value.gb,
            _padding1: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // or bytemuck::Pod, bytemuck::Zeroable
pub struct TiltShiftUniforms {
    pub max_radius: f32,
    pub dir_x: i32,
    pub dir_y: i32,
    pub center: f32,     // 0.0 bis 1.0 (Top-Down Position des scharfen Streifens)
    pub focus_size: f32, // Breite der Zone, die komplett scharf bleibt
    pub feather: f32,    // Radius/Breite der Weichzeichnungskante (Übergangszone)
    pub _padding1: f32,
    pub _padding2: f32,
}
impl TiltShiftUniforms {
    pub fn horizontal(settings: &TiltShiftSettings) -> Self {
        Self {
            max_radius: settings.max_radius,
            dir_x: 1,
            dir_y: 0,
            center: settings.center,
            focus_size: settings.focus_size,
            feather: settings.feather,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
    pub fn vertical(settings: &TiltShiftSettings) -> Self {
        Self {
            max_radius: settings.max_radius,
            dir_x: 0,
            dir_y: 1,
            center: settings.center,
            focus_size: settings.focus_size,
            feather: settings.feather,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PixelateTiltUniforms {
    // --- 1. Block (16 Bytes) ---
    pub max_pixel_size: f32, // Maximale Kachelgröße am Rand (z. B. 16.0 oder 32.0)
    pub angle: f32,          // Rotation des Fokusstreifens in Radians (0.0 bis 2*PI)
    pub center_x: f32,       // Zentrum des Streifens auf X (0.0 bis 1.0)
    pub center_y: f32,       // Zentrum des Streifens auf Y (0.0 bis 1.0)

    // --- 2. Block (16 Bytes) ---
    pub focus_size: f32, // Breite der perfekt scharfen Zone (0.0 bis 1.0)
    pub feather: f32,    // Übergangsbreite bis zur maximalen Verpixelung (0.0 bis 1.0)
    pub _padding1: f32,
    pub _padding2: f32,
}

impl From<PixelateTiltSettings> for PixelateTiltUniforms {
    fn from(value: PixelateTiltSettings) -> Self {
        Self {
            max_pixel_size: value.max_pixel_size,
            angle: value.angle,
            center_x: value.center_x,
            center_y: value.center_y,
            focus_size: value.focus_size,
            feather: value.feather,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}
