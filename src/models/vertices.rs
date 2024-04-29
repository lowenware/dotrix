use crate::graphics::Format;
use crate::math::Vec3;

pub trait VertexAttribute: 'static {
    type Raw: bytemuck::Pod + bytemuck::Zeroable;
    fn name() -> &'static str;
    fn format() -> Format;
    fn pack(&self) -> Self::Raw;
}

pub struct VertexPosition {
    pub value: Vec3,
}

impl VertexAttribute for VertexPosition {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Position"
    }
    fn format() -> Format {
        Format::Float32x3
    }
    fn pack(&self) -> Self::Raw {
        self.value.into()
    }
}

pub struct VertexNormal {
    pub value: Vec3,
}

impl VertexAttribute for VertexNormal {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Normal"
    }
    fn format() -> Format {
        Format::Float32x3
    }
    fn pack(&self) -> Self::Raw {
        self.value.into()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct VertexTexture {
    pub u: f32,
    pub v: f32,
}

impl VertexTexture {
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}

impl VertexAttribute for VertexTexture {
    type Raw = [f32; 2];
    fn name() -> &'static str {
        "TexUV"
    }
    fn format() -> Format {
        Format::Float32x2
    }
    fn pack(&self) -> [f32; 2] {
        [self.u, self.v]
    }
}

pub struct VertexTangent {
    pub value: Vec3,
}

impl VertexAttribute for VertexTangent {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Tangent"
    }
    fn format() -> Format {
        Format::Float32x3
    }
    fn pack(&self) -> Self::Raw {
        self.value.into()
    }
}

pub struct VertexBitangent {
    pub value: Vec3,
}

impl VertexAttribute for VertexBitangent {
    type Raw = [f32; 3];
    fn name() -> &'static str {
        "Bitangent"
    }
    fn format() -> Format {
        Format::Float32x3
    }
    fn pack(&self) -> Self::Raw {
        self.value.into()
    }
}

pub struct VertexWeights {
    value: [f32; 4],
}

impl VertexAttribute for VertexWeights {
    type Raw = [f32; 4];
    fn name() -> &'static str {
        "Weights"
    }
    fn format() -> Format {
        Format::Float32x4
    }
    fn pack(&self) -> Self::Raw {
        self.value
    }
}

pub struct VertexJoints {
    value: [u16; 4],
}

impl VertexAttribute for VertexJoints {
    type Raw = [u16; 4];
    fn name() -> &'static str {
        "Weights"
    }
    fn format() -> Format {
        Format::Uint16x4
    }
    fn pack(&self) -> Self::Raw {
        self.value
    }
}
