use super::Device;
use ash::vk;
use std::ffi::c_void;

/// Device memory map
pub struct MemoryMap<'a, A, T: HasDeviceMemory> {
    buffer: &'a mut T,
    mapped_memory: *mut c_void,
    offset: usize,
    size: usize,
    _align_by: std::marker::PhantomData<A>,
}

impl<'a, A, T: HasDeviceMemory> Drop for MemoryMap<'a, A, T> {
    fn drop(&mut self) {
        let device_memory = self.buffer.device_memory();
        unsafe {
            device_memory
                .device
                .vk_device
                .unmap_memory(*device_memory.vk_device_memory);
        }
    }
}

pub struct DeviceMemory<'a> {
    pub(super) device: &'a Device,
    pub(super) vk_device_memory: &'a vk::DeviceMemory,
}

/// Device memory abstraction
pub trait HasDeviceMemory {
    /// get device memory
    fn device_memory<'a>(&'a self) -> DeviceMemory<'a>;
}

impl<'a, A, T: HasDeviceMemory> MemoryMap<'a, A, T> {
    pub(super) unsafe fn new(buffer: &'a mut T, offset: usize, size: usize) -> Self {
        let device_memory = buffer.device_memory();
        let mapped_memory = device_memory
            .device
            .vk_device
            .map_memory(
                *device_memory.vk_device_memory,
                offset as vk::DeviceSize,
                size as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to map Vulkan device memory");

        Self {
            buffer,
            mapped_memory,
            offset,
            size,
            _align_by: std::marker::PhantomData::default(),
        }
    }

    pub fn write_from_slice<D: Copy>(&self, data: &[D]) {
        let mut aligned_memory = unsafe {
            ash::util::Align::new(
                self.mapped_memory,
                std::mem::align_of::<A>() as u64,
                self.size as vk::DeviceSize,
            )
        };
        aligned_memory.copy_from_slice(&data);
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}
