# Dotrix

Dotrix is an OpenSource 3D engine for Rust developers. The name is a derivation
from dot and matrix. Two entities that both together and separately are
keystones of rendering.

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)
[![](https://tokei.rs/b1/github/lowenware/dotrix)](https://github.com/lowenware/dotrix)
[![Discord](https://img.shields.io/discord/706575068515532851.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/DrzwBysNRd)

[![](https://img.shields.io/badge/lowenware%20-%23FF0000.svg?&style=for-the-badge&logo=YouTube&logoColor=white)](https://www.youtube.com/channel/UCdriNXRizbBFQhqZefaw44A)
[![](https://img.shields.io/badge/lowenware%20-%231DA1F2.svg?&style=for-the-badge&logo=Twitter&logoColor=white)](http://www.twitter.com/lowenware)

## Important

**Dotrix is under migration to Vulkan**. If you are looking for old WGPU version, then check out
[release 0.5.3](https://github.com/lowenware/dotrix/tree/release/v0.5.3).

The **main** branch now holds version 0.6 of the engine that is under active
development, but we try to keep it functional.

## Demo

**Dotrix Demo** is a binary that comes within the engine to demonstrate it possibilities.

```
cargo run --release
```

## Shaders

We are using GLSL shaders. There is no auto-compilation to SPV right now, so please use `glslc`:

```
glslc -fshader-stage=vertex src/models/shaders/only_mesh.vert -o src/models/shaders/only_mesh.vert.spv
glslc -fshader-stage=fragment src/models/shaders/only_mesh.frag -o src/models/shaders/only_mesh.frag.spv
glslc -fshader-stage=vertex src/models/shaders/skin_mesh.vert -o src/models/shaders/skin_mesh.vert.spv
glslc -fshader-stage=fragment src/models/shaders/skin_mesh.frag -o src/models/shaders/skin_mesh.frag.spv
```

## Sponsors

* Johan Andersson <[@repi](https://github.com/repi)>

## Contributors

* Andrew King <[@QuantumEntangledAndy](https://github.com/QuantumEntangledAndy)>
* Russell Wong <[@russellwmy](https://github.com/russellwmy)>

### 3rd Party Assets

Following 3rd party assets are being used in examples

* [Night SkyBox](https://www.vippng.com/preview/wmRTT_city-skyline-silhouette/)
* [Car Model](https://free3d.com/3d-model/cartoon-vehicles-low-poly-cars-free-874937.html)
* [Fox Model](https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0/Fox)
* [Day Skybox](https://opengameart.org/content/elyvisions-skyboxes)
* [PBR Crate](https://3dtextures.me/2021/12/20/stylized-crate-002/)
