use std::collections::HashMap;
use super::super::renderer::transform::{ Transform, TransformBuilder };
use dotrix_math::{Mat4, SquareMatrix};

/// Joint identificator (unsigned integer)
pub type JointId = usize;

/// Joint data structure
pub struct Joint {
    /// Local bind transformation of the joint
    pub local_bind_transform: Transform,
    /// name of the joint
    pub name: Option<String>,
    /// Id of the joint (defined by model)
    pub id: JointId,
    /// Parent joint Id
    pub parent_id: Option<JointId>,
}

impl Joint {
    /// Constructs new [`Joint`]
    pub fn new(
        id: JointId,
        parent_id: Option<JointId>,
        name: Option<String>,
        local_bind_transform: Transform,
    ) -> Self {
        Self { local_bind_transform, name, id, parent_id }
    }

    fn transform(
        &self,
        parent_transform: &Mat4,
        local_transform: Option<&TransformBuilder>,
    ) -> JointTransform {
        let local_transform = local_transform
            .map(|l| self.local_bind_transform.merge(l))
            .as_ref()
            .unwrap_or(&self.local_bind_transform)
            .matrix();

        JointTransform {
            id: self.id,
            global_transform: parent_transform * local_transform
        }
    }
}

/// Joints inverse bind matrix index
pub struct JointIndex {
    /// Id of the [`Joint`]
    pub id: JointId,
    /// Inverse bind matrix of the [`Joint`]
    pub inverse_bind_matrix: Option<Mat4>,
}

/// Model skin attribute
#[derive(Default)]
pub struct Skin {
    /// List of all skin joints (the order does matter)
    pub joints: Vec<Joint>,
    /// Joints inverse bind matrix index
    pub index: Vec<JointIndex>,
}

impl Skin {
    /// Constructs new [`Skin`]
    pub fn new(
        joints: Vec<Joint>,
        mut index: Vec<JointIndex>,
        inverse_bind_matrices: Option<Vec<Mat4>>,
    ) -> Self {

        if let Some(inverse_bind_matrices) = inverse_bind_matrices {
            for (mut joint_index, matrix) in index.iter_mut().zip(inverse_bind_matrices.iter()) {
                joint_index.inverse_bind_matrix = Some(*matrix);
            }
        }

        Self {
            joints,
            index,
        }
    }

    fn index(&self, joint_id: JointId) -> usize {
        self.joints.iter().position(|j| j.id == joint_id).unwrap()
    }

    /// Applies `local_transforms` to the [`Skin`] and stores output in the [`Pose`]
    pub fn transform(
        &self,
        skin_transform: &mut Pose,
        model_transform: &Mat4,
        local_transforms: Option<HashMap<JointId, TransformBuilder>>,
    ) {

        for (i, joint) in self.joints.iter().enumerate() {
            let parent_transform = joint.parent_id
                .map(|parent_id| skin_transform.joints[self.index(parent_id)].global_transform)
                .or(Some(*model_transform))
                .unwrap();

            let local_transform = local_transforms
                .as_ref()
                .map(|l| l.get(&joint.id))
                .unwrap_or(None);

            skin_transform.joints[i] = joint.transform(&parent_transform, local_transform);
        }
    }
}

/// Transformation of the joint
#[derive(Debug, Clone)]
pub struct JointTransform {
    id: JointId,
    global_transform: Mat4,
}

impl Default for JointTransform {
    fn default() -> Self {
        Self {
            id: 0,
            global_transform: Mat4::identity(),
        }
    }
}

/// Transformed [`Skin`] state
pub struct Pose {
    /// Transformations of the [`Skin`] joints
    pub joints: Vec<JointTransform>,
    /// Pipeline buffer
    pub buffer: wgpu::Buffer,
}

impl Pose {
    /// Constructs new [`Pose`]
    pub fn new(device: &wgpu::Device) -> Self {
        use wgpu::util::DeviceExt;
        let joints_matrices: [[[f32; 4]; 4]; 32] = [Mat4::identity().into(); 32];
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Pose Buffer"),
                contents: bytemuck::cast_slice(&joints_matrices),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );
        Self {
            joints: vec![JointTransform::default(); 32], // 32 -> MAX_JOINTS
            buffer,
        }
    }

    /// Loads buffers of the [`Pose`]
    pub fn load(&self, index: &[JointIndex], queue: &wgpu::Queue) {
        let joints_matrices = self.matrices(index);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(joints_matrices.as_slice()));
    }

    /// Returns transformation matrices in proper order and packed to be used in shaders
    pub fn matrices(&self, index: &[JointIndex]) -> Vec<[[f32; 4]; 4]> {
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

        while result.len() < 32 {
            result.push(Mat4::identity().into());
        }
        result
    }
}

