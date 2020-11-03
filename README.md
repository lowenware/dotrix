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

## Getting started

Dotrix provides an application builder named after the engine itself, to build and run your
application.

```
// THE EXAMPLE ONLY DEMONSTRATES HOW THINGS ARE SUPPOSED TO BE. THE RENDERER IS NOT IMPLEMENTED YET

use dotrix::{
  Dotrix,
  ecs::{ System, RunLevel },
  renderer::{static_renderer, static_renderer_startup}
};

fn main() {
    Dotrix::application("My Application")
        .with_system(System::from(static_renderer_startup).with(RunLevel::startup))
        .with_system(System::from(static_renderer).with(RunLevel::render))
        .with_system(System::from(my_system))
        .run();

    struct Armor(u32);
    struct Health(u32);
    struct Damage(u32);

    fn my_system(mut world: Mut<World>) {
        // bulk spawning of entities with the same Archetype (faster that spawning in cycle) 
        let bulk = (0..9).map(|_| (Armor(35), Health(5000), Damage(350)));
        world.spawn(bulk);

        // spawn single entity
        world.spawn(Some((Damage(600), Armor(10))));

        
        let iter = world.query::<(&mut Health,)>();
        for (hp,) in iter {
            hp.0 -= 1;
        }
    }
```

## Systems with context

It is possible to define a context for the system. Context is a data structure, available only for
the system. Context has one requirement: `Default` trait has to be implemented.

```

#[derive(Default)]
struct Control {
    is_jump: boolean;
}

fn control_system(mut ctx: Context<Control>, input: Const<Input>) {
  ctx.is_jump = input.get_button(Action::Jump);
}
```
