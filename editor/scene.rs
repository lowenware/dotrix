use dotrix::prelude::*;
use dotrix::{ Assets, Camera, Input, Frame, Color, World };
use dotrix::assets::Texture;
use dotrix::input::{ State as InputState, Button };
use dotrix::pbr::Light;
use dotrix::math::Vec3;

use std::f32::consts::PI;

const ROTATE_SPEED: f32 = PI / 10.0;
const ZOOM_SPEED: f32 = 100.0;

pub fn startup(
    mut camera: Mut<Camera>,
    mut world: Mut<World>,
) {
    world.spawn(Some((
        Light::Simple {
            // direction: Vec3::new(0.3, -0.5, -0.6),
            position: Vec3::new(0.0, 1000.0, 0.0),
            color: Color::white(),
            intensity: 0.8,
            enabled: true,
        },
    )));
    // spawn source of white light at (0.0, 100.0, 0.0)
    world.spawn(Some((
        Light::Ambient {
            color: Color::white(),
            intensity: 0.8,
        },
    )));

    camera.distance = 4000.0;
}

pub fn control(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    let distance = camera.distance - ZOOM_SPEED * mouse_scroll * time_delta;
    camera.distance = if distance > -1.0 { distance } else { -1.0 };

    if Some(InputState::Hold) == input.button_state(Button::MouseRight) {
        camera.y_angle += mouse_delta.x * ROTATE_SPEED * time_delta;

        let xz_angle = camera.xz_angle + mouse_delta.y * ROTATE_SPEED * time_delta;  
        let half_pi = PI / 2.0;

        camera.xz_angle = if xz_angle >= half_pi {
            half_pi - 0.01
        } else if xz_angle <= -half_pi {
            -half_pi + 0.01
        } else {
            xz_angle
        };
    }
}
