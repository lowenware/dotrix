use std::time::{Duration};
use dotrix_math::{SquareMatrix, Mat4};

use crate::{
    assets::{Animation, Id},
    components::Model,
    ecs::{Const},
    services::{Assets, Frame, World},
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    Play(Duration),
    Loop(Duration),
    Stop,
}

pub struct Animator {
    animation: Id<Animation>,
    state: State,
    pub speed: f32,
}

impl Animator {
    pub fn new(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: State::Stop,
            speed: 1.0,
        }
    }

    pub fn play(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: State::Play(Duration::from_secs(0)),
            speed: 1.0,
        }
    }

    pub fn looped(animation: Id<Animation>) -> Self {
        Self {
            animation,
            state: State::Loop(Duration::from_secs(0)),
            speed: 1.0,
        }
    }

    pub fn start(&mut self) {
        self.state = State::Play(Duration::from_secs(0));
    }

    pub fn start_loop(&mut self) {
        self.state = State::Loop(Duration::from_secs(0));
    }

    pub fn stop(&mut self) {
        self.state = State::Stop;
    }

    pub fn animate(&mut self, animation: Id<Animation>) {
        self.animation = animation;
        self.state = State::Stop;
    }

    pub fn animation(&self) -> Id<Animation> {
        self.animation
    }

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
            },
            State::Loop(current) => {
                let new_duration = current + delta.mul_f32(self.speed);
                State::Loop(
                    if new_duration < duration {
                        new_duration
                    } else {
                        Duration::from_secs_f32(new_duration.as_secs_f32() % duration.as_secs_f32())
                    }
                )
            },
            State::Stop => State::Stop
        };

        match self.state {
            State::Play(current) => Some(current),
            State::Loop(current) => Some(current),
            State::Stop => None,
        }
    }

}

pub fn skeletal_animation(frame: Const<Frame>, world: Const<World>, assets: Const<Assets>) {
    for (model, animator) in world.query::<(&mut Model, &mut Animator)>() {
        let global_transform = Mat4::identity(); // model.transform.matrix();
        if let Some(skin) = assets.get(model.skin) {

            let mut local_transforms = None;

            if let Some(animation) = assets.get::<Animation>(animator.animation) {
                if let Some(duration) = animator.update(frame.delta(), animation.duration()) {
                    local_transforms = Some(animation.sample(duration.as_secs_f32()));
                }
            }

            if let Some(pose) = model.pose.as_mut() {
                skin.transform(pose, &global_transform, local_transforms);
            }
        }
    }
}
