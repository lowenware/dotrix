use super::memory_map::{DeviceMemory, HasDeviceMemory, MemoryMap};
use super::{Device, Gpu};
use ash::vk;
use std::sync::Arc;

/// GPU buffer
pub struct Buffer {
    device: Arc<Device>,
    vk_buffer: vk::Buffer,
    vk_device_memory: vk::DeviceMemory,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .free_memory(self.vk_device_memory, None);
            self.device.vk_device.destroy_buffer(self.vk_buffer, None);
        }
    }
}

impl Buffer {
    pub fn setup() -> Constructor {
        Constructor::default()
    }

    pub fn size(&self) -> usize {
        unsafe { Self::memory_requirements(&self.device, self.vk_buffer).size as usize }
    }

    pub fn map_memory<A, D: Copy>(&mut self, offset: usize, data: &[D]) -> MemoryMap<A, Self> {
        let size = Self::calculate_aligned_size::<A, _>(data);
        unsafe { MemoryMap::<A, Self>::new(self, offset, size) }
    }

    pub fn write_from_slice<A, D: Copy>(&mut self, offset: usize, data: &[D]) {
        let map = self.map_memory::<A, D>(offset, data);
        map.write_from_slice(data);
    }

    pub fn calculate_aligned_size<A, D: Copy>(data: &[D]) -> usize {
        let align = std::mem::align_of::<A>();
        let data_size = std::mem::size_of::<D>();
        let entry_size = data_size + (align - data_size % align) % align;
        entry_size * data.len()
    }

    unsafe fn memory_requirements(
        device: &Device,
        vk_buffer: vk::Buffer,
    ) -> vk::MemoryRequirements {
        device.vk_device.get_buffer_memory_requirements(vk_buffer)
    }

    unsafe fn allocate_device_memory(device: &Device, vk_buffer: vk::Buffer) -> vk::DeviceMemory {
        let memory_requirements = Self::memory_requirements(device, vk_buffer);
        let memory_type_index = Buffer::find_memory_type_index(
            &memory_requirements,
            &device.memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memorytype for the Vulkan buffer.");

        let memory_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index: memory_type_index,
            ..Default::default()
        };

        device
            .vk_device
            .allocate_memory(&memory_allocate_info, None)
            .expect("Failed to allocate Vulkan buffer memory")
    }

    fn find_memory_type_index(
        memory_req: &vk::MemoryRequirements,
        memory_prop: &vk::PhysicalDeviceMemoryProperties,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        memory_prop.memory_types[..memory_prop.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (1 << index) & memory_req.memory_type_bits != 0
                    && memory_type.property_flags & flags == flags
            })
            .map(|(index, _memory_type)| index as _)
    }
}

impl HasDeviceMemory for Buffer {
    fn device_memory(&self) -> DeviceMemory {
        DeviceMemory {
            device: &self.device,
            vk_device_memory: &self.vk_device_memory,
        }
    }
}

/// Buffer descriptor
#[derive(Default)]
pub struct Constructor {
    // TODO: sparse binding
    // flags: vk::BufferCreateFlags,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    // TODO: concurent mode
    // sharing_mode: vk::SharingMode,
}

impl Constructor {
    pub fn create(self, gpu: &Gpu) -> Buffer {
        let buffer_create_info = vk::BufferCreateInfo {
            // flags: self.flags,
            size: self.size,
            usage: self.usage,
            // sharing_mode: self.sharing_mode,
            ..Default::default()
        };
        let device = Arc::clone(&gpu.device);
        let vk_buffer = unsafe {
            device
                .vk_device
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to create a Vulkan buffer")
        };
        let vk_device_memory = unsafe { Buffer::allocate_device_memory(&device, vk_buffer) };

        Buffer {
            device,
            vk_buffer,
            vk_device_memory,
        }
    }

    pub fn use_as_src(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::TRANSFER_SRC;
        self
    }

    pub fn use_as_dst(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::TRANSFER_DST;
        self
    }

    pub fn use_as_uniform_texel(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER;
        self
    }

    pub fn use_as_storage_texel(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER;
        self
    }

    pub fn use_as_uniform(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        self
    }

    pub fn use_as_storage(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::STORAGE_BUFFER;
        self
    }

    pub fn use_as_index(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::INDEX_BUFFER;
        self
    }

    pub fn use_as_vertex(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::VERTEX_BUFFER;
        self
    }

    pub fn use_as_indirect(mut self) -> Self {
        self.usage |= vk::BufferUsageFlags::INDIRECT_BUFFER;
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = size as u64;
        self
    }
}
