//! Animation components and systems
//!
//! ## Enabling animation
//!
//! To add skeletal animation support to your game you only need to add a [`skeletal_animation`]
//! system using [`crate::Dotrix`] builder.
//!
//! ```no_run
//! use dotrix_core::{
//!     Dotrix,
//!     ecs::System,
//!     systems::skeletal_animation,
//! };
//!
//! // in your main function
//! Dotrix::application("My Game")
//!     .with_system(System::from(skeletal_animation))
//!     .run()
//! 
//! ```
//!
//! ## Spawning animated models
//!
//! Animated entities can be added to the [`World`] using [`crate::components::Model`] component 
//! with configured [`crate::assets::Skin`] and [`Animator`] component constructed with
//! [`Animation`] asset.
//!
//! ```no_run
//! use dotrix_core::{
//!     ecs::Mut,
//!     components::{ Model, Animator },
//!     services::{ Assets, World }
//! };
//!
//! fn my_system(mut world: Mut<World>, mut assets: Mut<Assets>) {
//!
//!     assets.import("assets/MyFile.gltf");
//!
//!     let mesh = assets.register("MyFile::model::mesh");
//!     let skin = assets.register("MyFile::model::skin");
//!     let texture = assets.register("MyFile::model::texture");
//!     let animation = assets.register("MyFile::animation");
//!
//!     world.spawn(Some((
//!         Model { mesh, texture, skin, ..Default::default() },
//!         Animator::looped(animation),
//!     )));
//! }
//! ```
//!

use std::time::{ Duration };
use dotrix_math::{ SquareMatrix, Mat4 };

use crate::{
    assets::{ Animation, Id },
    components::Model,
    ecs::{ Const },
    services::{ Assets, Frame, World },
};

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

/// System handling skeletal animation
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
