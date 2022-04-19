use super::Context;
use wgpu;
use wgpu::util::DeviceExt;

/// GPU Buffer
pub struct Buffer {
    /// Buffer Label
    pub label: String,
    /// WGPU buffer instance
    pub wgpu_buffer: Option<wgpu::Buffer>,
    /// Buffer Usage
    pub usage: wgpu::BufferUsages,
}

impl Buffer {
    /// Construct new Buffer Instance
    pub fn new(label: &str) -> Self {
        Self {
            label: label.into(),
            wgpu_buffer: None,
            usage: wgpu::BufferUsages::COPY_DST,
        }
    }

    /// Construct new Vertex Buffer
    pub fn vertex(label: &str) -> Self {
        Self::new(label).use_as_vertex()
    }

    /// Construct new Index Buffer
    pub fn index(label: &str) -> Self {
        Self::new(label).use_as_index()
    }

    /// Construct new Storage buffer
    pub fn storage(label: &str) -> Self {
        Self::new(label).use_as_storage()
    }

    /// Construct new Uniform buffer
    pub fn uniform(label: &str) -> Self {
        Self::new(label).use_as_uniform()
    }

    /// Construct new Indirect buffer
    pub fn indirect(label: &str) -> Self {
        Self::new(label).use_as_indirect()
    }

    /// Construct new Map Read buffer
    pub fn map_read(label: &str) -> Self {
        Self::new(label).use_as_map_read()
    }

    /// Construct new Map Write buffer
    pub fn map_write(label: &str) -> Self {
        Self::new(label).use_as_map_write()
    }

    /// Allow to use as Vertex Buffer
    #[must_use]
    pub fn use_as_vertex(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::VERTEX;
        self
    }

    /// Allow to use as Index Buffer
    #[must_use]
    pub fn use_as_index(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::INDEX;
        self
    }

    /// Allow to use as Storage Buffer
    #[must_use]
    pub fn use_as_storage(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::STORAGE;
        self
    }

    /// Allow to use as Uniform Buffer
    #[must_use]
    pub fn use_as_uniform(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::UNIFORM;
        self
    }

    /// Allow to use as Indirect Buffer
    #[must_use]
    pub fn use_as_indirect(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::INDIRECT;
        self
    }

    /// Allow to use as Map Read Buffer
    #[must_use]
    pub fn use_as_map_read(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::MAP_READ;
        self
    }

    /// Allow to use as Map Write Buffer
    #[must_use]
    pub fn use_as_map_write(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::MAP_WRITE;
        self
    }

    /// Allow reading from buffer
    #[must_use]
    pub fn allow_read(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::COPY_DST;
        self
    }

    /// Allow writing to buffer
    #[must_use]
    pub fn allow_write(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::COPY_SRC;
        self
    }

    /// Return true if buffer is writable
    pub fn can_write(&self) -> bool {
        self.usage.contains(wgpu::BufferUsages::COPY_SRC)
    }

    /// Load data into the buffer
    pub fn load<'a>(&mut self, ctx: &Context, data: &'a [u8]) {
        if let Some(buffer) = self.wgpu_buffer.as_ref() {
            ctx.queue.write_buffer(buffer, 0, data);
        } else {
            self.wgpu_buffer = Some(ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(self.label.as_str()),
                    contents: data,
                    usage: self.usage,
                },
            ));
        }
    }

    /// Create buffer of size without data
    ///
    /// Typically used for staging buffers
    pub fn create(&mut self, ctx: &Context, size: u32, mapped: bool) {
        self.wgpu_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(self.label.as_str()),
            size: size as wgpu::BufferAddress,
            usage: self.usage,
            mapped_at_creation: mapped,
        }));
    }

    /// Check if buffer isloaded
    pub fn loaded(&self) -> bool {
        self.wgpu_buffer.is_some()
    }

    /// Release all resources used by the buffer
    pub fn unload(&mut self) {
        self.wgpu_buffer.take();
    }

    /// Get unwrapped reference to WGPU buffer
    pub fn get(&self) -> &wgpu::Buffer {
        self.wgpu_buffer.as_ref().expect("Buffer must be loaded")
    }

    /// Get optional reference to WGPU buffer
    pub fn as_ref(&self) -> Option<&wgpu::Buffer> {
        self.wgpu_buffer.as_ref()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new("Noname Buffer")
    }
}
