use std::sync::Arc;

use super::Device;
use crate::log;
use ash::vk;

#[derive(Clone)]
pub struct Semaphore {
    inner: Arc<SemaphoreInner>,
}

impl Semaphore {
    pub fn vk_semaphore(&self) -> &vk::Semaphore {
        self.inner.vk_semaphore
    }
}

struct SemaphoreInner {
    vk_semaphore: vk::Semaphore,
    device: Arc<Device>,
}

impl Drop for SemaphoreInner {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .destroy_semaphore(self.vk_semaphore, None);
        };
    }
}
