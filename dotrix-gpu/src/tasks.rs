use crate::{Commands, Renderer};
use dotrix_core::{Color};
use dotrix_os::{Task, Any, All, Ro, Rw};

pub struct Frame {
    inner: wgpu::Frame,
    delta: std::time::Duration,
}


