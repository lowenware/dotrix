use std::sync::Arc;
use ash::vk;

impl Device {
    pub unsafe fn new(
        vk_device: ash::Device,
        vk_instance: ash::Instance,
        vk_entry: ash::Entry,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
        queue_family_index: u32,
    ) -> Self {
        Self {
            vk_device,
            vk_instance,
            _vk_entry: vk_entry,
            memory_properties,
            queue_family_index,
        }
    }
}

