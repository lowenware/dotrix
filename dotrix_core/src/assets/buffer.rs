//! Buffer asset
//!

use crate::{
    providers::BufferProvider,
    reloadable::*,
    renderer::{Buffer as GpuBuffer, Renderer},
};

/// Buffer asset
pub struct Buffer {
    /// The length of the data that the buffer was created with
    pub data_len: usize,
    /// The underlying gpu buffer
    pub buffer: GpuBuffer,
    /// Flagged on change
    pub reload_state: ReloadState,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new("Buffer")
    }
}

impl Buffer {
    /// Construct a new Buffer asset
    pub fn new(label: &str) -> Self {
        Self {
            data_len: 0,
            buffer: GpuBuffer::new(label),
            reload_state: Default::default(),
        }
    }

    /// Construct with a Vertex Buffer
    pub fn vertex(label: &str) -> Self {
        Self::new(label).use_as_vertex()
    }

    /// Construct with a Index Buffer
    pub fn index(label: &str) -> Self {
        Self::new(label).use_as_index()
    }

    /// Construct with a Storage buffer
    pub fn storage(label: &str) -> Self {
        Self::new(label).use_as_storage()
    }

    /// Construct with a Uniform buffer
    pub fn uniform(label: &str) -> Self {
        Self::new(label).use_as_uniform()
    }

    /// Construct with a Indirect buffer
    pub fn indirect(label: &str) -> Self {
        Self::new(label).use_as_indirect()
    }

    /// Construct with a Map Read buffer
    pub fn map_read(label: &str) -> Self {
        Self::new(label).use_as_map_read()
    }

    /// Construct with a Map Write buffer
    pub fn map_write(label: &str) -> Self {
        Self::new(label).use_as_map_write()
    }

    /// Allow to use as Vertex Buffer
    #[must_use]
    pub fn use_as_vertex(mut self) -> Self {
        self.buffer = self.buffer.use_as_vertex();
        self
    }

    /// Allow to use as Index Buffer
    #[must_use]
    pub fn use_as_index(mut self) -> Self {
        self.buffer = self.buffer.use_as_index();
        self
    }

    /// Allow to use as Storage Buffer
    #[must_use]
    pub fn use_as_storage(mut self) -> Self {
        self.buffer = self.buffer.use_as_storage();
        self
    }

    /// Allow to use as Uniform Buffer
    #[must_use]
    pub fn use_as_uniform(mut self) -> Self {
        self.buffer = self.buffer.use_as_uniform();
        self
    }

    /// Allow to use as Indirect Buffer
    #[must_use]
    pub fn use_as_indirect(mut self) -> Self {
        self.buffer = self.buffer.use_as_indirect();
        self
    }

    /// Allow to use as Map Read Buffer
    #[must_use]
    pub fn use_as_map_read(mut self) -> Self {
        self.buffer = self.buffer.use_as_map_read();
        self
    }

    /// Allow to use as Map Write Buffer
    #[must_use]
    pub fn use_as_map_write(mut self) -> Self {
        self.buffer = self.buffer.use_as_map_write();
        self
    }

    /// Allow reading from buffer
    #[must_use]
    pub fn allow_read(mut self) -> Self {
        self.buffer = self.buffer.allow_read();
        self
    }

    /// Allow writing to buffer
    #[must_use]
    pub fn allow_write(mut self) -> Self {
        self.buffer = self.buffer.allow_write();
        self
    }

    /// Return true if buffer is writable
    pub fn can_write(&self) -> bool {
        self.buffer.can_write()
    }

    /// Loads the [`Buffer`] data to a buffer
    pub fn load(&mut self, renderer: &Renderer, data: &[u8]) {
        if data.len() != self.data_len {
            self.unload();
        } else if data.len() == self.data_len {
            self.flag_update();
        }
        renderer.load_buffer(&mut self.buffer, data);
        self.data_len = data.len();
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.flag_reload();
        self.data_len = 0;
        self.buffer.unload();
    }

    /// Check if underlying buffer is loaded
    pub fn loaded(&self) -> bool {
        self.buffer.loaded()
    }
}

impl Reloadable for Buffer {
    fn get_reload_state(&self) -> &ReloadState {
        &self.reload_state
    }
}

impl ReloadableMut for Buffer {
    fn get_reload_state_mut(&mut self) -> &mut ReloadState {
        &mut self.reload_state
    }
}

impl Reloadable for &mut Buffer {
    fn get_reload_state(&self) -> &ReloadState {
        &self.reload_state
    }
}

impl ReloadableMut for &mut Buffer {
    fn get_reload_state_mut(&mut self) -> &mut ReloadState {
        &mut self.reload_state
    }
}

impl Reloadable for &Buffer {
    fn get_reload_state(&self) -> &ReloadState {
        &self.reload_state
    }
}

impl BufferProvider for Buffer {
    fn get_buffer(&self) -> &GpuBuffer {
        &self.buffer
    }
}

impl BufferProvider for &Buffer {
    fn get_buffer(&self) -> &GpuBuffer {
        &self.buffer
    }
}
