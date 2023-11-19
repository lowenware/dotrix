use super::Device;
use ash::vk;
use std::sync::Arc;

pub struct Queue {
    pub(super) vk_queue: vk::Queue,
    pub(super) device: Arc<Device>,
}

impl Queue {
    pub(crate) unsafe fn new(device: &Arc<Device>, queue_family_index: u32) -> Self {
        let vk_queue = device.vk_device.get_device_queue(queue_family_index, 0);
        Self {
            vk_queue,
            device: Arc::clone(device),
        }
    }
}

impl Clone for Queue {
    fn clone(&self) -> Self {
        Self {
            vk_queue: self.vk_queue,
            device: Arc::clone(&self.device),
        }
    }
}
