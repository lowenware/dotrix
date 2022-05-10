use dotrix_core::reloadable::*;
use dotrix_core::renderer::Buffer;
use dotrix_core::{Color, Renderer};
use dotrix_derive::*;

pub const MAX_LAYERS: usize = 16;

/// Terrain layer
pub struct Layer {
    /// Terrain layer color
    pub color: Color,
    /// Terrain layer height base 0.0..1.0
    pub height: f32,
    /// Terrain layer blend
    pub blend: f32,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            color: Color::rgb(0.18, 0.62, 0.24),
            height: -1.0,
            blend: 0.1,
        }
    }
}

/// Terrain layers container
#[derive(Reloadable, BufferProvider)]
#[buffer_provider(field = "uniform")]
pub struct Layers {
    /// List of terrain layers
    pub list: Vec<Layer>,
    /// Layers uniform buffer
    pub uniform: Buffer,
    /// The reload state
    pub reload_state: ReloadState,
}

impl Layers {
    /// Loads layers uniform into GPU
    pub fn load(&mut self, renderer: &Renderer) {
        renderer.load_buffer(
            &mut self.uniform,
            bytemuck::cast_slice(&[Uniform::from(self.list.as_slice())]),
        );
        self.flag_update();
    }
}

impl Default for Layers {
    fn default() -> Self {
        Self {
            list: vec![],
            uniform: Buffer::uniform("Terrain Layers Buffer"),
            reload_state: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct LayerUniform {
    color: [f32; 4],
    height: f32,
    blend: f32,
    unused: [u32; 2],
}

unsafe impl bytemuck::Zeroable for LayerUniform {}
unsafe impl bytemuck::Pod for LayerUniform {}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Uniform {
    count: u32,
    unused: [u32; 3],
    layers: [LayerUniform; MAX_LAYERS],
}

impl From<&[Layer]> for Uniform {
    fn from(layers: &[Layer]) -> Self {
        use std::convert::TryInto;

        let count = layers.len() as u32;
        let mut layers = layers
            .iter()
            .map(|layer| LayerUniform {
                color: layer.color.into(),
                height: layer.height,
                blend: layer.blend,
                unused: [0; 2],
            })
            .collect::<Vec<_>>();

        layers.resize(MAX_LAYERS, LayerUniform::default());

        Uniform {
            count,
            unused: [0; 3],
            layers: layers.try_into().unwrap(),
        }
    }
}

unsafe impl bytemuck::Zeroable for Uniform {}
unsafe impl bytemuck::Pod for Uniform {}
