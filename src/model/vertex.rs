use dotrix_math::{Vec2, Vec3};

/// Vertex Attribute Format
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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

pub trait Attribute: 'static {
    type Raw: bytemuck::Pod + bytemuck::Zeroable;
    fn name() -> &'static str;
    fn format() -> AttributeFormat;
}

pub struct Position {
    pub value: Vec3,
}

impl Attribute for Position {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Position"
    }
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x3
    }
}

pub struct Normal {
    pub value: Vec3,
}

impl Attribute for Normal {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Normal"
    }
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x3
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TexUV {
    pub u: f32,
    pub v: f32,
}

// TODO: introduce pack in teh  Attribute trait
impl TexUV {
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
    pub fn pack(&self) -> [f32; 2] {
        [self.u, self.v]
    }
}

impl Attribute for TexUV {
    type Raw = [f32; 2];
    fn name() -> &'static str {
        "TexUV"
    }
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x2
    }
}

impl From<&TexUV> for [f32; 2] {
    fn from(value: &TexUV) -> Self {
        [value.u, value.v]
    }
}

pub struct Tangent {
    pub value: Vec3,
}

impl Attribute for Tangent {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Tangent"
    }
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x3
    }
}

pub struct Bitangent {
    pub value: Vec3,
}

impl Attribute for Bitangent {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Bitangent"
    }
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x3
    }
}

pub struct Weights {
    value: [f32; 4],
}

impl Attribute for Weights {
    type Raw = [f32; 4];
    fn name() -> &'static str {
        "Weights"
    }
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x4
    }
}

pub struct Joints {
    value: [u16; 4],
}

impl Attribute for Joints {
    type Raw = [u16; 4];
    fn name() -> &'static str {
        "Weights"
    }
    fn format() -> AttributeFormat {
        AttributeFormat::Uint16x4
    }
}
