# Dotrix

3D Game Engine written in Rust (development stage)

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)
[![](https://tokei.rs/b1/github/lowenware/dotrix)](https://github.com/lowenware/dotrix)

## Overview

Dotrix has a flat linear ECS (Entity Component System) in its core, designed for fast querying of
entities and their components.

1. **Entities** in Dotrix are virtual abstractions, identified by `EntityId` component containing
numerical ID. Each entitiy agregates constant number of components.
2. **Components** are regular Rust structures.
3. **Systems** are Rust functions, implementing the core logic of a game.
4. **Services** are Rust objects available through systems, providing some key
features or access to global resources, like Assets, Input or Render management.

## Example
To compile and run demo, execute following command:

```
cargo run --release --example demo
```

## Getting started

Dotrix provides transparent API and application builder named after the engine itself, to build and run your
application. The following code is a copy of `demo` example.

```
use dotrix::{
    Dotrix,
    assets::{ Mesh, Texture },
    components::{ Light, StaticModel },
    ecs::{ Mut, RunLevel, System },
    services::{ Assets, Camera, World },
    systems::{ static_renderer },
};

fn main() {

    Dotrix::application("Input Example")
        .with_system(System::from(static_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(fly_around))
        .with_service(Assets::new())
        .with_service(Camera::new(10.0, 3.14 / 2.0, 4.0))
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    assets.import("assets/crate.png", "crate");

    let texture = assets.find::<Texture>("crate");
    let cube1 = assets.register::<Mesh>(Mesh::cube(), String::from("cube1"));
    let cube2 = assets.register::<Mesh>(Mesh::cube2(), String::from("cube2"));

    world.spawn(vec![
        (StaticModel::new(cube2, texture),),
        (StaticModel::new(cube1, texture),),
    ]);

    world.spawn(Some((Light::white([10.0, 2.0, 4.0]),)));
}

fn fly_around(mut camera: Mut<Camera>) {
    let target = cgmath::Point3::new(0.0, 0.0, 0.0);
    let distance = camera.distance();
    let angle = camera.angle() + 0.002;
    let height = camera.height();

    camera.set(target, distance, angle, height);
}
```

## Systems with context

It is possible to define a context for the system. Context is a data structure, only available for
that system. Context has one requirement: `Default` trait has to be implemented.

```

#[derive(Default)]
struct Control {
    is_jump: boolean;
}

fn control_system(mut ctx: Context<Control>, input: Const<Input>) {
  ctx.is_jump = input.get_button(Action::Jump);
}
```
