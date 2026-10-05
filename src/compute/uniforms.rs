use crate::app_state::effect_stack::{BwSettings, HsvSettings, VignetteSettings};

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
