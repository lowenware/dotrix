use crate::{
    assets::{
        skin::{JointIndex, JointTransform, Skin, MAX_JOINTS},
        Assets,
    },
    id::Id,
    reloadable::*,
    renderer::{Buffer, Renderer},
};

use dotrix_derive::*;
use dotrix_math::{Mat4, SquareMatrix};

/// Transformed [`Skin`] state
#[derive(Reloadable, BufferProvider)]
#[buffer_provider(field = "uniform")]
pub struct Pose {
    /// Id of a Skin asset
    pub skin: Id<Skin>,
    /// Transformations of the [`Skin`] joints
    pub joints: Vec<JointTransform>,
    /// Joints transformations buffer
    pub uniform: Buffer,
    /// The reload state
    pub reload_state: ReloadState,
}

impl Pose {
    /// Loads the [`Pose`] into GPU buffers
    pub fn load(&mut self, renderer: &Renderer, assets: &Assets) -> bool {
        if let Some(skin) = assets.get(self.skin) {
            let joints_matrices = self.matrices(&skin.index);
            renderer.load_buffer(
                &mut self.uniform,
                bytemuck::cast_slice(joints_matrices.as_slice()),
            );
            self.flag_reload();
            return true;
        }
        false
    }

    /// Returns transformation matrices in proper order and packed to be used in shaders
    fn matrices(&self, index: &[JointIndex]) -> Vec<[[f32; 4]; 4]> {
        let mut result = index
            .iter()
            .map(|i| {
                let joint_transform = self.joints.iter().find(|j| j.id == i.id).unwrap();
                let global_transform = &joint_transform.global_transform;
                let inverse_bind_matrix = i.inverse_bind_matrix;
                inverse_bind_matrix
                    .as_ref()
                    .map(|ibmx| global_transform * ibmx)
                    .unwrap_or(*global_transform)
                    .into()
            })
            .collect::<Vec<_>>();

        while result.len() < MAX_JOINTS {
            result.push(Mat4::identity().into());
        }
        result
    }
}

impl From<Id<Skin>> for Pose {
    /// Constructs new [`Pose`]
    fn from(skin: Id<Skin>) -> Self {
        Self {
            skin,
            joints: vec![JointTransform::default(); MAX_JOINTS], // 32 -> MAX_JOINTS
            uniform: Buffer::uniform("Pose Buffer"),
            reload_state: Default::default(),
        }
    }
}
