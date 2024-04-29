use std::collections::HashMap;
use std::hash::Hash;

/// Buffer layout and safety checker
pub struct BufferLayout<I, T>
where
    I: Hash,
    T: Clone,
{
    registry: HashMap<I, T>,
    total_size: u64,
    used_size: u64,
}

impl<I, T> BufferLayout<I, T>
where
    I: Hash + PartialEq + Eq,
    T: Clone,
{
    pub fn new(total_size: u64) -> Self {
        Self {
            registry: HashMap::new(),
            total_size,
            used_size: 0,
        }
    }

    pub fn get(&self, index: I) -> Option<&T> {
        self.registry.get(&index)
    }

    /// Returns Ok(offset_in_buffer) if there is enough space in buffer, or Err(overflow_size)
    /// otherwise
    pub fn store(&mut self, index: I, entry: T, entry_size: u64) -> Result<u64, u64> {
        let offset_in_buffer = self.used_size;
        let total_size = self.total_size;
        let used_size_after = offset_in_buffer + entry_size;
        if used_size_after > total_size {
            return Err(used_size_after - total_size);
        }
        self.registry.insert(index, entry);
        self.used_size = used_size_after;
        Ok(offset_in_buffer)
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn used_size(&self) -> u64 {
        self.used_size
    }
}

/// Layout of data in a buffer
#[derive(Clone, Copy)]
pub struct LayoutInBuffer {
    pub offset: u64,
    pub size: u64,
}

/// Layout of a single mesh verticies
#[derive(Clone, Copy)]
pub struct MeshVerticesLayout {
    /// Offset of the first model vertex in vertex buffer (vertex number, not bytes)
    pub base_vertex: u32,
    /// Number of vertices of the model
    pub vertex_count: u32,
}

/// Layout of a single mesh in buffers
#[derive(Clone, Copy)]
pub struct MeshLayout {
    pub in_vertex_buffer: LayoutInBuffer,
    pub in_index_buffer: Option<LayoutInBuffer>,
    pub vertices: MeshVerticesLayout,
}
