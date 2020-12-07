use std::time::{Duration};
use std::collections::HashMap;
use super::{
    skin::JointId,
    transform::Transform,
};

#[derive(Debug)]
pub enum Interpolation {
    Linear,
    Step,
    CubicSpline,
}

impl Interpolation {
    pub fn from(interpolation: gltf::animation::Interpolation) -> Self {
        match interpolation {
            gltf::animation::Interpolation::Linear => Interpolation::Linear,
            gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
            gltf::animation::Interpolation::Step => Interpolation::Step,
        }
    }
}

// TODO: move to math
/// nalgebra and cgmath lerp methods produce some weird artifacts, see link bellow
/// https://github.com/rustgd/cgmath/issues/300
pub fn slerp(
    left: cgmath::Quaternion<f32>,
    right: cgmath::Quaternion<f32>,
    amount: f32
) -> cgmath::Quaternion<f32> {
    let num2;
    let num3;
    let num = amount;
    let mut num4 = (((left.v.x * right.v.x) + (left.v.y * right.v.y)) + (left.v.z * right.v.z))
        + (left.s * right.s);
    let mut flag = false;
    if num4 < 0.0 {
        flag = true;
        num4 = -num4;
    }
    if num4 > 0.999_999 {
        num3 = 1.0 - num;
        num2 = if flag { -num } else { num };
    } else {
        let num5 = num4.acos();
        let num6 = 1.0 / num5.sin();
        num3 = ((1.0 - num) * num5).sin() * num6;
        num2 = if flag {
            -(num * num5).sin() * num6
        } else {
            (num * num5).sin() * num6
        };
    }
    cgmath::Quaternion::new(
        (num3 * left.s) + (num2 * right.s),
        (num3 * left.v.x) + (num2 * right.v.x),
        (num3 * left.v.y) + (num2 * right.v.y),
        (num3 * left.v.z) + (num2 * right.v.z),
    )
}

trait Interpolate: Copy {
    fn linear(self, target: Self, value: f32) -> Self;
}

impl Interpolate for cgmath::Vector3<f32> {
    fn linear(self, target: Self, value: f32) -> Self {
        use cgmath::VectorSpace;
        self.lerp(target, value)
    }
}

impl Interpolate for cgmath::Quaternion<f32> {
    fn linear(self, target: Self, value: f32) -> Self {
        slerp(self, target, value)
    }
}

/// Keyframes for the channel transformations of type T
pub struct KeyFrame<T> {
    transformation: T,
    timestamp: f32,
}

impl<T> KeyFrame<T> {
    fn new(timestamp: f32, transformation: T) -> Self {
        Self {
            timestamp,
            transformation
        }
    }
}

struct Channel<T: Interpolate + Copy + Clone> {
    keyframes: Vec<KeyFrame<T>>,
    joint_id: JointId,
    interpolation: Interpolation,
}

impl<T: Interpolate + Copy + Clone> Channel<T> {
    fn from(
        joint_id: JointId,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        transforms: Vec<T>
    ) -> Self {
        let keyframes = timestamps.into_iter().zip(transforms.into_iter()).map(
            |(timestamp, transformation)| KeyFrame::new(timestamp, transformation)
        ).collect::<Vec<_>>();

        Channel {
            interpolation,
            keyframes,
            joint_id,
        }
    }

    fn sample(&self, keyframe: f32) -> Option<T> {
        for i in 0..self.keyframes.len() - 1 {
            let first = &self.keyframes[i];
            let next = &self.keyframes[i + 1];
            if keyframe >= first.timestamp && keyframe < next.timestamp {
                return match self.interpolation {
                    Interpolation::Step => Some(first.transformation),
                    Interpolation::Linear => {
                        let value = (keyframe - first.timestamp) /
                            (next.timestamp - first.timestamp);
                        Some(first.transformation.linear(next.transformation, value))
                    },
                    _ => panic!("Unsupported interpolation {:?}", self.interpolation),
                };
            }
        }
        None
    }
}

pub struct Animation {
    duration: Duration,
    translation_channels: Vec<Channel<cgmath::Vector3<f32>>>,
    rotation_channels: Vec<Channel<cgmath::Quaternion<f32>>>,
    scale_channels: Vec<Channel<cgmath::Vector3<f32>>>,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            duration: Duration::from_secs(0),
            translation_channels: Vec::new(),
            rotation_channels: Vec::new(),
            scale_channels: Vec::new(),
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn add_translation_channel(
        &mut self,
        joint_id: JointId,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        translations: Vec<cgmath::Vector3<f32>>,
    ) {
        self.update_duration(&timestamps);
        self.translation_channels.push(Channel::from(joint_id, interpolation, timestamps, translations));
    }

    pub fn add_rotation_channel(
        &mut self,
        joint_id: JointId,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        rotations: Vec<cgmath::Quaternion<f32>>,
    ) {
        self.update_duration(&timestamps);
        self.rotation_channels.push(Channel::from(joint_id, interpolation, timestamps, rotations));
    }

    pub fn add_scale_channel(
        &mut self,
        joint_id: JointId,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        scales: Vec<cgmath::Vector3<f32>>,
    ) {
        self.update_duration(&timestamps);
        self.scale_channels.push(Channel::from(joint_id, interpolation, timestamps, scales));
    }

    fn update_duration(&mut self, timestamps: &[f32]) {
        let max_timestamp = timestamps.last().copied().unwrap_or(0.0);
        let duration = Duration::from_secs_f32(max_timestamp);
        if duration > self.duration {
            self.duration = duration;
        }
    }

    pub fn sample(&self, keyframe: f32) -> HashMap<JointId, Transform> {
        let mut result = HashMap::new();

        for channel in &self.translation_channels {
            if let Some(transform) = channel.sample(keyframe) {
                result.insert(channel.joint_id, Transform::from_translation(transform));
            }
        }

        for channel in &self.rotation_channels {
            if let Some(transform) = channel.sample(keyframe) {
                if let Some(t) = result.get_mut(&channel.joint_id) {
                    t.rotate = Some(transform);
                } else {
                    result.insert(channel.joint_id, Transform::from_rotation(transform));
                }
            }
        }
        for channel in &self.scale_channels {
            if let Some(transform) = channel.sample(keyframe) {
                if let Some(t) = result.get_mut(&channel.joint_id) {
                    t.scale = Some(transform);
                } else {
                    result.insert(channel.joint_id, Transform::from_scale(transform));
                }
            }
        }

        result
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self::new()
    }
}
