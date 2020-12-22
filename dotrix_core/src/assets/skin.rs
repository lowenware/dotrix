use std::collections::HashMap;
use crate::math::{ Transform, TransformBuilder };

pub type JointId = usize;

pub struct Joint {
    pub local_bind_transform: Transform,
    pub name: Option<String>,
    pub id: JointId,
    pub parent_id: Option<JointId>,
}

impl Joint {

    pub fn new(
        id: JointId,
        parent_id: Option<JointId>,
        name: Option<String>,
        local_bind_transform: Transform,
    ) -> Self {

        Self {
            id,
            parent_id,
            name,
            local_bind_transform,
        }
    }

    fn transform(
        &self,
        parent_transform: &cgmath::Matrix4<f32>,
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

pub struct JointIndex {
    pub id: JointId,
    pub inverse_bind_matrix: Option<cgmath::Matrix4<f32>>,
}

#[derive(Default)]
pub struct Skin {
    /// Joints
    pub joints: Vec<Joint>, // the order does matter
    pub index: Vec<JointIndex>,
}

impl Skin {
    pub fn new(
        joints: Vec<Joint>,
        mut index: Vec<JointIndex>,
        inverse_bind_matrices: Option<Vec<cgmath::Matrix4<f32>>>,
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

    pub fn transform(
        &self,
        skin_transform: &mut Pose,
        model_transform: &cgmath::Matrix4<f32>,
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

#[derive(Debug, Clone)]
pub struct JointTransform {
    id: JointId,
    /// global joint transformation
    global_transform: cgmath::Matrix4<f32>,
}

impl Default for JointTransform {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            id: 0,
            global_transform: cgmath::Matrix4::<f32>::identity(),
        }
    }
}

pub struct Pose {
    /// Joints
    pub joints: Vec<JointTransform>, // the order does matter
    pub buffer: wgpu::Buffer,
}

impl Pose {
    pub fn new(device: &wgpu::Device) -> Self {
        use cgmath::SquareMatrix;
        use wgpu::util::DeviceExt;
        let joints_matrices: [[[f32; 4]; 4]; 32] = [cgmath::Matrix4::<f32>::identity().into(); 32];
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

    pub fn load(&self, index: &[JointIndex], queue: &wgpu::Queue) {
        let joints_matrices = self.matrices(index);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(joints_matrices.as_slice()));
    }

    pub fn matrices(&self, index: &[JointIndex]) -> Vec<[[f32; 4]; 4]> {
        use cgmath::SquareMatrix;
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
            result.push(cgmath::Matrix4::<f32>::identity().into());
        }
        result
    }
}

