/// Buffer Wrapper
pub struct Buffer {
    pub inner: wgpu::Buffer,
}

impl Buffer {}

/// Buffer Builder
pub struct Builder<'a, 'b> {
    pub gpu: &'a crate::Gpu,
    pub descriptor: wgpu::BufferDescriptor<'b>,
}

impl<'a, 'b> Builder<'a, 'b> {
    #[inline(always)]
    pub fn size(mut self, size: wgpu::BufferAddress) -> Self {
        self.descriptor.size = size;
        self
    }

    #[inline(always)]
    pub fn allow_map_read(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::MAP_READ;
        self
    }

    #[inline(always)]
    pub fn allow_map_write(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::MAP_WRITE;
        self
    }

    #[inline(always)]
    pub fn allow_copy_dst(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::COPY_DST;
        self
    }

    #[inline(always)]
    pub fn allow_copy_src(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::COPY_SRC;
        self
    }

    #[inline(always)]
    pub fn use_as_vertex(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::VERTEX;
        self
    }

    #[inline(always)]
    pub fn use_as_index(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::INDEX;
        self
    }

    #[inline(always)]
    pub fn use_as_uniform(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::UNIFORM;
        self
    }

    #[inline(always)]
    pub fn use_as_storage(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::STORAGE;
        self
    }

    #[inline(always)]
    pub fn use_as_indirect(mut self) -> Self {
        self.descriptor.usage |= wgpu::BufferUsages::INDIRECT;
        self
    }

    #[inline(always)]
    pub fn create(self) -> Buffer {
        self.gpu.create_buffer(&self.descriptor)
    }
}
