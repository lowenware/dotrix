# Dotrix

3D Game Engine written in Rust (development stage)

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)
[![](https://tokei.rs/b1/github/lowenware/dotrix)](https://github.com/lowenware/dotrix)
[![Discord](https://img.shields.io/discord/706575068515532851.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/DrzwBysNRd)

[![](https://img.shields.io/badge/lowenware%20-%23FF0000.svg?&style=for-the-badge&logo=YouTube&logoColor=white)](https://www.youtube.com/channel/UCdriNXRizbBFQhqZefaw44A)
[![](https://img.shields.io/badge/lowenware%20-%231DA1F2.svg?&style=for-the-badge&logo=Twitter&logoColor=white)](http://www.twitter.com/lowenware)

## Overview

Dotrix has a flat linear ECS (Entity Component System) in its core, designed for fast querying of
entities and their components.

1. **Entities** in Dotrix are virtual abstractions, identified by `EntityId` component containing
numerical ID. Each entitiy agregates constant number of components.
2. **Components** are regular Rust structures.
3. **Systems** are Rust functions, implementing the core logic of a game.
4. **Services** are Rust objects available through systems, providing some key
features or access to global resources, like Assets, Input or Render management.

## Editor

Editor application is under development in the separate [branch](https://github.com/lowenware/dotrix/tree/feat/editor)

```
cargo run --release --bin editor
```

![Dotrix Editor](https://github.com/lowenware/dotrix/blob/feat/editor/editor-screenshot.png)

## Getting started

The best place to start is to review examples distributed with the engine. All examples are grouped
under [examples/](examples/) folder. Later when API becomes more or less stable we will prepare a
Book for a quick start.

## Examples
[![Demo Example](https://img.youtube.com/vi/KXOr_KxMNWM/0.jpg)](https://www.youtube.com/watch?v=KXOr_KxMNWM)

**Features:** input, skeletal animation, light, terrain, player control
```
cargo run --release --example demo
```

**Features:** Light, UI, camera control
```
cargo run --release --example light
```

**Features:** skeletal animation, light, camera control
```
cargo run --release --example animation
```

**Features:** skybox, camera control
```
cargo run --release --example skybox
```

**Features:** window management
```
cargo run --release --example window
```

### 3rd Party Assets

Following 3rd party assets are being used in examples

* [Night SkyBox](https://www.vippng.com/preview/wmRTT_city-skyline-silhouette/)
* [Car Model](https://free3d.com/3d-model/cartoon-vehicles-low-poly-cars-free-874937.html)
* [Fox Model](https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0/Fox)
* [Day Skybox](https://opengameart.org/content/elyvisions-skyboxes)
