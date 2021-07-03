use crate::{
    renderer::{ Renderer, UniformBuffer },
};

use dotrix_math::{ Vec3, Quat, Rad, Rotation3 };

/// Transformation component
pub struct Transform {
    /// Translation vector
    pub translate: Vec3,
    /// Rotation quaternion
    pub rotate: Quat,
    /// Scale vector
    pub scale: Vec3,
    /// Transformation Uniform buffer
    pub buffer: UniformBuffer,
}

impl Transform {
    pub fn set(&mut self, transform: &crate::generics::Transform) {
        self.translate = transform.translate;
        self.rotate = transform.rotate;
        self.scale = transform.scale;
    }

    pub fn get(&self) -> crate::generics::Transform {
        crate::generics::Transform {
            translate: self.translate,
            rotate: self.rotate,
            scale: self.scale,
        }
    }

    pub fn load(&mut self, renderer: &Renderer) {
        let transform_matrix = self.get().matrix();
        let transform_raw = AsRef::<[f32; 16]>::as_ref(&transform_matrix);
        renderer.load_uniform_buffer(&mut self.buffer, bytemuck::cast_slice(transform_raw));
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate: Vec3::new(0.0, 0.0, 0.0),
            rotate: Quat::from_angle_y(Rad(0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
            buffer: UniformBuffer::default(),
        }
    }
}
