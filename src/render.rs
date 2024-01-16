pub mod formats;
pub mod frame;
pub mod pbr;
pub mod vulkan;

pub use ash::vk;

use crate::window::Window;

pub use formats::Extent2D;
pub use frame::{CreateFrame, Frame, RenderPass, SubmitFrame};
pub use pbr::RenderModels;
pub use vulkan::{
    CommandBufferIter, CommandRecorder, CommandRecorderSetup, Device, Display, FramePresenter,
    Framebuffers, Gpu, Semaphore, Surface,
};

/// GPU device type
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceType {
    /// Integrated GPU
    Integrated,
    /// Discrete GPU
    Discrete,
}

pub struct DisplaySetup<'a> {
    /// Window reference
    pub window: Window,
    /// Application name
    pub app_name: &'a str,
    /// Application version
    pub app_version: u32,
    /// Debug flag
    pub debug: bool,
    /// Device type request
    pub device_type_request: Option<DeviceType>,
}

/// Value Format
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Format {
    /// 32 bit float attribute
    Float32,
    /// 2 x 32 bit float attribute
    Float32x2,
    /// 3 x 32 bit float attribute
    Float32x3,
    /// 4 x 32 bit float attribute
    Float32x4,
    /// 2 x 16 bit unsigned integer attribute
    Uint16x2,
    /// 4 x 16 bit unsigned integer attribute
    Uint16x4,
    /// 32 bit unsigned integer attribute
    Uint32,
    /// 2 x 32 bit unsigned integer attribute
    Uint32x2,
    /// 3 x 32 bit unsigned integer attribute
    Uint32x3,
    /// 4 x 32 bit unsigned integer attribute
    Uint32x4,
}

impl Format {
    /// Returns the actual attribute size in bytes
    pub fn size(&self) -> usize {
        match self {
            Format::Float32 => 4,
            Format::Float32x2 => 4 * 2,
            Format::Float32x3 => 4 * 3,
            Format::Float32x4 => 4 * 4,
            Format::Uint16x2 => 2 * 2,
            Format::Uint16x4 => 2 * 4,
            Format::Uint32 => 4,
            Format::Uint32x2 => 4 * 2,
            Format::Uint32x3 => 4 * 3,
            Format::Uint32x4 => 4 * 4,
        }
    }

    /// Returns the actual attribute TypeId
    pub fn type_id(&self) -> std::any::TypeId {
        match self {
            Format::Float32 => std::any::TypeId::of::<f32>(),
            Format::Float32x2 => std::any::TypeId::of::<[f32; 2]>(),
            Format::Float32x3 => std::any::TypeId::of::<[f32; 3]>(),
            Format::Float32x4 => std::any::TypeId::of::<[f32; 4]>(),
            Format::Uint16x2 => std::any::TypeId::of::<[u16; 2]>(),
            Format::Uint16x4 => std::any::TypeId::of::<[u16; 4]>(),
            Format::Uint32 => std::any::TypeId::of::<u32>(),
            Format::Uint32x2 => std::any::TypeId::of::<[u32; 2]>(),
            Format::Uint32x3 => std::any::TypeId::of::<[u32; 3]>(),
            Format::Uint32x4 => std::any::TypeId::of::<[u32; 4]>(),
        }
    }
}
