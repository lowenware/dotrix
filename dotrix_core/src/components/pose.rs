use crate::{
    assets::{
        Assets,
        skin::{ JointId, JointTransform, JointIndex, MAX_JOINTS, Skin },
    },
    generics::Id,
    renderer::{ Renderer, UniformBuffer },
};

use dotrix_math::{ Mat4, SquareMatrix };

/// Transformed [`Skin`] state
pub struct Pose {
    pub skin: Id<Skin>,
    /// Transformations of the [`Skin`] joints
    pub joints: Vec<JointTransform>,
    /// Joints transformations buffer
    pub buffer: UniformBuffer,
}

impl Pose {
    /// Constructs new [`Pose`]
    pub fn new(skin: Id<Skin>) -> Self {
        Self {
            skin,
            joints: vec![JointTransform::default(); MAX_JOINTS], // 32 -> MAX_JOINTS
            buffer: UniformBuffer::default(),
        }
    }

    /// Loads the [`Pose`] into GPU buffers
    pub fn load(&mut self, renderer: &Renderer, assets: &Assets) {
        if let Some(skin) = assets.get(self.skin) {
            let joints_matrices = self.matrices(&skin.index);
            renderer.load_uniform_buffer(
                &mut self.buffer,
                bytemuck::cast_slice(joints_matrices.as_slice())
            );
        }
    }

    /// Returns transformation matrices in proper order and packed to be used in shaders
    fn matrices(&self, index: &[JointIndex]) -> Vec<[[f32; 4]; 4]> {
        let mut result = index.iter().map(|i| {
            let joint_transform = self.joints.iter().find(|j| j.id == i.id).unwrap();
            let global_transform = &joint_transform.global_transform;
            let inverse_bind_matrix = i.inverse_bind_matrix;
            inverse_bind_matrix
                .as_ref()
                .map(|ibmx| global_transform * ibmx)
                .unwrap_or(*global_transform)
                .into()
        }).collect::<Vec<_>>();

        while result.len() < MAX_JOINTS {
            result.push(Mat4::identity().into());
        }
        result
    }
}

