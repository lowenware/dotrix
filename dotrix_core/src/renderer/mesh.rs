/// Vertex Attribute Format
#[derive(Debug)]
pub enum AttributeFormat {
    /// 32 bit float attribute
    Float32,
    /// 2 x 32 bit float attribute
    Float32x2,
    /// 3 x 32 bit float attribute
    Float32x3,
    /// 4 x 32 bit float attribute
    Float32x4,
    /// 2 x 16 bit unsigned integer attribute
    Uint16x2,
    /// 4 x 16 bit unsigned integer attribute
    Uint16x4,
    /// 32 bit unsigned integer attribute
    Uint32,
    /// 2 x 32 bit unsigned integer attribute
    Uint32x2,
    /// 3 x 32 bit unsigned integer attribute
    Uint32x3,
    /// 4 x 32 bit unsigned integer attribute
    Uint32x4,
}

impl AttributeFormat {
    /// Returns the actual attribute size in bytes
    pub fn size(&self) -> usize {
        match self {
            AttributeFormat::Float32 => 4,
            AttributeFormat::Float32x2 => 4 * 2,
            AttributeFormat::Float32x3 => 4 * 3,
            AttributeFormat::Float32x4 => 4 * 4,
            AttributeFormat::Uint16x2 => 2 * 2,
            AttributeFormat::Uint16x4 => 2 * 4,
            AttributeFormat::Uint32 => 4,
            AttributeFormat::Uint32x2 => 4 * 2,
            AttributeFormat::Uint32x3 => 4 * 3,
            AttributeFormat::Uint32x4 => 4 * 4,
        }
    }

    /// Returns the actual attribute TypeId
    pub fn type_id(&self) -> std::any::TypeId {
        match self {
            AttributeFormat::Float32 => std::any::TypeId::of::<f32>(),
            AttributeFormat::Float32x2 => std::any::TypeId::of::<[f32; 2]>(),
            AttributeFormat::Float32x3 => std::any::TypeId::of::<[f32; 3]>(),
            AttributeFormat::Float32x4 => std::any::TypeId::of::<[f32; 4]>(),
            AttributeFormat::Uint16x2 => std::any::TypeId::of::<[u16; 2]>(),
            AttributeFormat::Uint16x4 => std::any::TypeId::of::<[u16; 4]>(),
            AttributeFormat::Uint32 => std::any::TypeId::of::<u32>(),
            AttributeFormat::Uint32x2 => std::any::TypeId::of::<[u32; 2]>(),
            AttributeFormat::Uint32x3 => std::any::TypeId::of::<[u32; 3]>(),
            AttributeFormat::Uint32x4 => std::any::TypeId::of::<[u32; 4]>(),
        }
    }
}

impl From<&AttributeFormat> for wgpu::VertexFormat {
    fn from(obj: &AttributeFormat) -> Self {
        match obj {
            AttributeFormat::Float32 => wgpu::VertexFormat::Float32,
            AttributeFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
            AttributeFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
            AttributeFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
            AttributeFormat::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            AttributeFormat::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            AttributeFormat::Uint32 => wgpu::VertexFormat::Uint32,
            AttributeFormat::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            AttributeFormat::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            AttributeFormat::Uint32x4 => wgpu::VertexFormat::Uint32x4,
        }
    }
}
