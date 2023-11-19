pub mod formats;
pub mod frame;
pub mod pbr;
pub mod vulkan;

use std::borrow::Cow;
use std::ffi::{c_char, CStr, CString};
use std::sync::Arc;

pub use ash::vk;

use crate::log;
use crate::window::Window;

pub use formats::Extent2D;
pub use frame::{CreateFrame, Frame, RenderPass, SubmitFrame};
pub use pbr::Renderer;
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
