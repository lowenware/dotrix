use dotrix::{
    assets::{ Mesh, Texture },
    components::{ Model, PointLight, SpotLight },
    ecs::{ Const, Mut },
    math::{ Vec3 },
    renderer::{ Color },
    services::{ Assets, Frame, World },
};
use std::vec;

pub struct CarLight;

#[derive(Clone)]
pub struct CarSettings {
    pub animate: bool,
    pub point_lights: bool,
    pub spot_lights: bool,
}

struct LightController {
    increasing: bool,
    min: f32,
    max: f32,
    current: f32,
    speed: f32,
}

pub fn init(mut world: Mut<World>, mut assets: Mut<Assets>) {

    assets.import("examples/light/assets/car.gltf");
    let car_mesh = assets.register::<Mesh>("car::mesh");
    // In real game, use one shared mesh with different transform
    let wh1_mesh = assets.register::<Mesh>("car::wheel-1::mesh");
    let wh2_mesh = assets.register::<Mesh>("car::wheel-2::mesh");
    let wh3_mesh = assets.register::<Mesh>("car::wheel-3::mesh");
    let wh4_mesh = assets.register::<Mesh>("car::wheel-4::mesh");
    let texture = assets.register::<Texture>("car::texture");

    world.spawn(vec![
        ( Model { mesh: car_mesh, texture, ..Default::default() }, ),
        ( Model { mesh: wh1_mesh, texture, ..Default::default() }, ),
        ( Model { mesh: wh2_mesh, texture, ..Default::default() }, ),
        ( Model { mesh: wh3_mesh, texture, ..Default::default() }, ),
        ( Model { mesh: wh4_mesh, texture, ..Default::default() }, ),
    ],);

    world.spawn(Some((CarSettings {
        animate: true,
        point_lights: true,
        spot_lights: true,
    },)));

    // Spawn lights
    world.spawn(vec![
        (
            PointLight {
                position: Vec3::new(0.28, 1.7, 0.487),
                color: Color::rgb(0.0, 0.25, 0.94),
                intensity: 3.0,
                ..Default::default()
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
            PointLight {
                position: Vec3::new(0.28, 1.7, -0.487),
                color: Color::rgb(0.94, 0.0, 0.25),
                intensity: 3.0,
                ..Default::default()
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
            SpotLight {
                position: Vec3::new(-1.4, 0.68, 0.58),
                direction: Vec3::new(-45.0, -5.0, 0.0),
                color: Color::white(),
                intensity: 0.9,
                cut_off: 0.8,
                outer_cut_off: 0.58,
                ..Default::default()
            },
            CarLight {},
        ),
        (
            SpotLight {
                position: Vec3::new(-1.4, 0.68, -0.58),
                direction: Vec3::new(-45.0, -5.0, 0.0),
                color: Color::white(),
                intensity: 0.9,
                cut_off: 0.8,
                outer_cut_off: 0.58,
                ..Default::default()
            },
            CarLight {},
        ),
    ],);
}

pub fn update(world: Mut<World>, frame: Const<Frame>) {

    // Get car settings
    let mut query = world.query::<(&mut CarSettings,)>();

    if let Some((settings,)) = query.next() {
        // Light animation
        if settings.animate {
            animate_lights(&world, &frame);
        }

        // Enable/disable point lights
        let query = world.query::<(&mut PointLight, &CarLight,)>();
        for (point_lights, _) in query {
            point_lights.enabled = settings.point_lights;
        }

        // Enable/disable spot lights
        let query = world.query::<(&mut SpotLight, &CarLight,)>();
        for (spot_light, _) in query {
            spot_light.enabled = settings.spot_lights;
        }
    }
}

fn animate_lights(world: &World, frame: &Frame) {
    let query = world.query::<(&mut PointLight, &mut LightController,)>();
    for (light, c) in query {
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

        light.intensity = c.current;
    }
}