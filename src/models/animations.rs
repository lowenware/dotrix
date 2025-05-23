use std::collections::HashMap;
use std::time::Duration;

use super::{Joint, Transform, TransformBuilder};
use crate::loaders::Asset;
use crate::math::{Quat, Vec3};
use crate::utils::Id;

pub struct Animation {
    name: String,
    duration: Duration,
    translation_channels: Vec<Channel<Vec3>>,
    rotation_channels: Vec<Channel<Quat>>,
    scale_channels: Vec<Channel<Vec3>>,
}

impl Animation {
    /// Constructs new asset instance
    pub fn new(name: String) -> Self {
        Self {
            name,
            duration: Duration::from_secs(0),
            translation_channels: Vec::new(),
            rotation_channels: Vec::new(),
            scale_channels: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns [`Duration`] of the animation
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Adds translation transformation channel
    pub fn add_translation_channel(
        &mut self,
        joint_id: Id<Joint>,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        translations: Vec<Vec3>,
    ) {
        self.update_duration(&timestamps);
        self.translation_channels.push(Channel::from(
            joint_id,
            interpolation,
            timestamps,
            translations,
        ));
    }

    /// Adds rotation transformation channel
    pub fn add_rotation_channel(
        &mut self,
        joint_id: Id<Joint>,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        rotations: Vec<Quat>,
    ) {
        self.update_duration(&timestamps);
        self.rotation_channels.push(Channel::from(
            joint_id,
            interpolation,
            timestamps,
            rotations,
        ));
    }

    /// Adds scale transformation channel
    pub fn add_scale_channel(
        &mut self,
        joint_id: Id<Joint>,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        scales: Vec<Vec3>,
    ) {
        self.update_duration(&timestamps);
        self.scale_channels
            .push(Channel::from(joint_id, interpolation, timestamps, scales));
    }

    fn update_duration(&mut self, timestamps: &[f32]) {
        let max_timestamp = timestamps.last().copied().unwrap_or(0.0);
        let duration = Duration::from_secs_f32(max_timestamp);
        if duration > self.duration {
            self.duration = duration;
        }
    }

    /// Samples the animeation at some keyframe (s) and returns a HashMap of
    /// [`crate::assets::Skin`] joint id to [`TransformBuilder`]
    pub fn sample(&self, timestamp: f32) -> HashMap<Id<Joint>, TransformBuilder> {
        let mut result = HashMap::new();
        let duration_secs = self.duration.as_secs_f32();

        let keyframe = if timestamp > duration_secs {
            timestamp % duration_secs
        } else {
            timestamp
        };

        for channel in &self.translation_channels {
            if let Some(transform) = channel.sample(keyframe) {
                result.insert(
                    channel.joint_id,
                    Transform::builder().with_translate(transform),
                );
            }
        }

        for channel in &self.rotation_channels {
            if let Some(transform) = channel.sample(keyframe) {
                if let Some(t) = result.get_mut(&channel.joint_id) {
                    t.rotate = Some(transform);
                } else {
                    result.insert(
                        channel.joint_id,
                        Transform::builder().with_rotate(transform),
                    );
                }
            }
        }
        for channel in &self.scale_channels {
            if let Some(transform) = channel.sample(keyframe) {
                if let Some(t) = result.get_mut(&channel.joint_id) {
                    t.scale = Some(transform);
                } else {
                    result.insert(channel.joint_id, Transform::builder().with_scale(transform));
                }
            }
        }

        result
    }
}

impl Asset for Animation {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Interpolation types
#[derive(Debug)]
pub enum Interpolation {
    /// Linear interpolation
    Linear,
    /// Step Interpolation
    Step,
    /// Cubic Interpolation
    CubicSpline,
}

trait Interpolate: Copy {
    fn linear(self, target: Self, value: f32) -> Self;
}

impl Interpolate for Vec3 {
    fn linear(self, target: Self, value: f32) -> Self {
        self.lerp(target, value)
    }
}

impl Interpolate for Quat {
    fn linear(self, target: Self, value: f32) -> Self {
        // NOTE: try slerp from math if any issue
        self.slerp(target, value)
    }
}

/// Keyframes for the channel transformations
pub(crate) struct KeyFrame<T> {
    transformation: T,
    timestamp: f32,
}

impl<T> KeyFrame<T> {
    fn new(timestamp: f32, transformation: T) -> Self {
        Self {
            transformation,
            timestamp,
        }
    }
}

struct Channel<T: Interpolate + Copy + Clone> {
    keyframes: Vec<KeyFrame<T>>,
    joint_id: Id<Joint>,
    interpolation: Interpolation,
}

impl<T: Interpolate + Copy + Clone> Channel<T> {
    fn from(
        joint_id: Id<Joint>,
        interpolation: Interpolation,
        timestamps: Vec<f32>,
        transforms: Vec<T>,
    ) -> Self {
        let keyframes = timestamps
            .into_iter()
            .zip(transforms)
            .map(|(timestamp, transformation)| KeyFrame::new(timestamp, transformation))
            .collect::<Vec<_>>();

        Channel {
            keyframes,
            joint_id,
            interpolation,
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
                        let value =
                            (keyframe - first.timestamp) / (next.timestamp - first.timestamp);
                        Some(first.transformation.linear(next.transformation, value))
                    }
                    _ => panic!("Unsupported interpolation {:?}", self.interpolation),
                };
            }
        }
        None
    }
}

/// Animation player state
///
/// [`Duration`] contains current time offset from the beginning of the animation
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AnimationState {
    /// Animation is playing
    Play(Duration),
    /// Animation is looped
    Loop(Duration),
    /// Animation is stopped
    Stop,
}

impl AnimationState {
    pub fn play() -> Self {
        AnimationState::Play(Duration::from_secs(0))
    }

    pub fn play_loop() -> Self {
        AnimationState::Loop(Duration::from_secs(0))
    }

    pub fn stop() -> Self {
        AnimationState::Stop
    }
}

/// Component to control model animation
pub struct AnimationPlayer {
    animation: Id<Animation>,
    state: AnimationState,
    /// animation speed
    speed: f32,
}

impl AnimationPlayer {
    /// creates new component instance with specified animation with [`State::Stop`]
    pub fn new(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: AnimationState::stop(),
            speed: 1.0,
        }
    }

    /// creates new component instance with specified animation with [`State::Play`]
    pub fn play(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: AnimationState::play(),
            speed: 1.0,
        }
    }

    /// creates new component instance with specified animation with [`State::Loop`]
    pub fn looped(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: AnimationState::play_loop(),
            speed: 1.0,
        }
    }

    /// Starts current animation
    pub fn start(&mut self) {
        self.state = AnimationState::play();
    }

    /// Starts current animation looped
    pub fn start_loop(&mut self) {
        self.state = AnimationState::play_loop();
    }

    /// Stops current animation
    pub fn stop(&mut self) {
        self.state = AnimationState::stop();
    }

    /// Changes current animation
    pub fn animate(&mut self, animation: Id<Animation>) {
        self.animation = animation;
        self.state = AnimationState::stop();
    }

    /// Returns current animation [`Id`]
    pub fn animation(&self) -> Id<Animation> {
        self.animation
    }

    /// Returns current [`State`]
    pub fn state(&self) -> AnimationState {
        self.state
    }

    pub fn update(&mut self, delta: Duration, duration: Duration) -> Option<Duration> {
        let (state, duration) = match self.state {
            AnimationState::Play(current) => {
                let new_duration = current + delta.mul_f32(self.speed);
                let state = if new_duration < duration {
                    AnimationState::Play(new_duration)
                } else {
                    AnimationState::Stop
                };
                (state, Some(new_duration))
            }
            AnimationState::Loop(current) => {
                let mut new_duration = current + delta.mul_f32(self.speed);
                if new_duration >= duration {
                    new_duration = Duration::from_secs_f32(
                        new_duration.as_secs_f32() % duration.as_secs_f32(),
                    );
                }
                let state = AnimationState::Loop(new_duration);
                (state, Some(new_duration))
            }
            AnimationState::Stop => (AnimationState::Stop, None),
        };
        self.state = state;
        duration
    }
}
