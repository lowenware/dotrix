//! Animation components and systems
use dotrix_math::{Mat4, SquareMatrix};
use std::time::Duration;

use crate::assets::Animation;
use crate::ecs::Const;
use crate::{Assets, Frame, Id, Pose, World};

/// Animation playback state
///
/// [`Duration`] contains current time offset from the beginning of the animation
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    /// Animation is playing
    Play(Duration),
    /// Animation is looped
    Loop(Duration),
    /// Animation is stopped
    Stop,
}

/// Component to control model animation
pub struct Animator {
    animation: Id<Animation>,
    state: State,
    /// animation speed
    pub speed: f32,
}

impl Animator {
    /// creates new component instance with specified animation with [`State::Stop`]
    pub fn new(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: State::Stop,
            speed: 1.0,
        }
    }

    /// creates new component instance with specified animation with [`State::Play`]
    pub fn play(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: State::Play(Duration::from_secs(0)),
            speed: 1.0,
        }
    }

    /// creates new component instance with specified animation with [`State::Loop`]
    pub fn looped(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: State::Loop(Duration::from_secs(0)),
            speed: 1.0,
        }
    }

    /// Starts current animation
    pub fn start(&mut self) {
        self.state = State::Play(Duration::from_secs(0));
    }

    /// Starts current animation looped
    pub fn start_loop(&mut self) {
        self.state = State::Loop(Duration::from_secs(0));
    }

    /// Stops current animation
    pub fn stop(&mut self) {
        self.state = State::Stop;
    }

    /// Changes current animation
    pub fn animate(&mut self, animation: Id<Animation>) {
        self.animation = animation;
        self.state = State::Stop;
    }

    /// Returns current animation [`Id`]
    pub fn animation(&self) -> Id<Animation> {
        self.animation
    }

    /// Returns current [`State`]
    pub fn state(&self) -> State {
        self.state
    }

    fn update(&mut self, delta: Duration, duration: Duration) -> Option<Duration> {
        self.state = match self.state {
            State::Play(current) => {
                let new_duration = current + delta.mul_f32(self.speed);
                if new_duration < duration {
                    State::Play(new_duration)
                } else {
                    State::Stop
                }
            }
            State::Loop(current) => {
                let new_duration = current + delta.mul_f32(self.speed);
                State::Loop(if new_duration < duration {
                    new_duration
                } else {
                    Duration::from_secs_f32(new_duration.as_secs_f32() % duration.as_secs_f32())
                })
            }
            State::Stop => State::Stop,
        };

        match self.state {
            State::Play(current) => Some(current),
            State::Loop(current) => Some(current),
            State::Stop => None,
        }
    }
}

/// System handling skeletal animation
pub fn skeletal(frame: Const<Frame>, world: Const<World>, assets: Const<Assets>) {
    for (animator, pose) in world.query::<(&mut Animator, &mut Pose)>() {
        let global_transform = Mat4::identity(); // model.transform.matrix();
        if let Some(skin) = assets.get(pose.skin) {
            let mut local_transforms = None;

            if let Some(animation) = assets.get::<Animation>(animator.animation) {
                if let Some(duration) = animator.update(frame.delta(), animation.duration()) {
                    local_transforms = Some(animation.sample(duration.as_secs_f32()));
                }
            }

            skin.transform(pose, &global_transform, local_transforms);
        }
    }
}
