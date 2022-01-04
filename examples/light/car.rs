use dotrix::assets::{Mesh, Texture};
use dotrix::ecs::{Const, Mut};
use dotrix::math::Vec3;
use dotrix::pbr::{self, Light};
use dotrix::{Assets, Color, Frame, World};

use crate::settings::Settings;

pub struct CarLight;

struct LightController {
    increasing: bool,
    min: f32,
    max: f32,
    current: f32,
    speed: f32,
}

pub fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    assets.import("assets/models/car.gltf");
    let car_mesh = assets.register::<Mesh>("car::mesh");
    // In real game, use one shared mesh with different transform
    let wh1_mesh = assets.register::<Mesh>("car::wheel-1::mesh");
    let wh2_mesh = assets.register::<Mesh>("car::wheel-2::mesh");
    let wh3_mesh = assets.register::<Mesh>("car::wheel-3::mesh");
    let wh4_mesh = assets.register::<Mesh>("car::wheel-4::mesh");
    let texture = assets.register::<Texture>("car::texture");

    world.spawn(vec![
        (pbr::solid::Entity {
            mesh: car_mesh,
            texture,
            ..Default::default()
        })
        .tuple(),
        (pbr::solid::Entity {
            mesh: wh1_mesh,
            texture,
            ..Default::default()
        })
        .tuple(),
        (pbr::solid::Entity {
            mesh: wh2_mesh,
            texture,
            ..Default::default()
        })
        .tuple(),
        (pbr::solid::Entity {
            mesh: wh3_mesh,
            texture,
            ..Default::default()
        })
        .tuple(),
        (pbr::solid::Entity {
            mesh: wh4_mesh,
            texture,
            ..Default::default()
        })
        .tuple(),
    ]);

    // Spawn lights
    world.spawn(vec![
        (
            Light::Point {
                position: Vec3::new(0.28, 1.7, 0.487),
                color: Color::rgb(0.0, 0.25, 0.94),
                intensity: 3.0,
                constant: 1.0,
                linear: 0.35,
                quadratic: 0.44,
                enabled: true,
            },
            CarLight {},
            LightController {
                increasing: true,
                current: 0.0,
                min: 0.0,
                max: 3.0,
                speed: 12.0,
            },
        ),
        (
            Light::Point {
                position: Vec3::new(0.28, 1.7, -0.487),
                color: Color::rgb(0.94, 0.0, 0.25),
                intensity: 3.0,
                constant: 1.0,
                linear: 0.35,
                quadratic: 0.44,
                enabled: true,
            },
            CarLight {},
            LightController {
                increasing: false,
                current: 3.0,
                min: 0.0,
                max: 3.0,
                speed: 12.0,
            },
        ),
    ]);

    world.spawn(vec![
        (
            Light::Spot {
                position: Vec3::new(-1.4, 0.68, 0.58),
                direction: Vec3::new(-45.0, -5.0, 0.0),
                color: Color::white(),
                intensity: 0.9,
                cut_off: 0.8,
                outer_cut_off: 0.58,
                enabled: true,
            },
            CarLight {},
        ),
        (
            Light::Spot {
                position: Vec3::new(-1.4, 0.68, -0.58),
                direction: Vec3::new(-45.0, -5.0, 0.0),
                color: Color::white(),
                intensity: 0.9,
                cut_off: 0.8,
                outer_cut_off: 0.58,
                enabled: true,
            },
            CarLight {},
        ),
    ]);
}

pub fn update(world: Mut<World>, frame: Const<Frame>, settings: Const<Settings>) {
    if settings.car.animate {
        animate_lights(&world, &frame);
    }

    // Enable/disable point lights
    let query = world.query::<(&mut Light, &CarLight)>();
    for (light, _) in query {
        match light {
            Light::Point { enabled, .. } => *enabled = settings.car.point_lights,
            Light::Spot { enabled, .. } => *enabled = settings.car.spot_lights,
            _ => continue,
        }
    }
}

fn animate_lights(world: &World, frame: &Frame) {
    let query = world.query::<(&mut Light, &mut LightController)>();
    for (light, c) in query {
        match light {
            Light::Point { intensity, .. } => {
                if c.increasing {
                    c.current += c.speed * frame.delta().as_secs_f32();
                } else {
                    c.current -= c.speed * frame.delta().as_secs_f32();
                }

                if c.current > c.max {
                    c.current = c.max;
                    c.increasing = !c.increasing;
                } else if c.current < c.min {
                    c.current = c.min;
                    c.increasing = !c.increasing;
                }

                *intensity = c.current;
            }
            _ => continue,
        }
    }
}
