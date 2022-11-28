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

**Dotrix is under heavy refactoring.** If you are looking for the latest stable
version, then check out
[release 0.5.3](https://github.com/lowenware/dotrix/tree/release/v0.5.3).

The **main** branch now holds version 0.6 of the engine that is under active
development.

## Aproximate TODO list

- [x] Tasks concept with parallel execution
- [x] New application API with possibility to configure extensions
- [x] Flexible Mesh API that allow to construct meshes and their buffers with
validation and various layouts
- [x] Shader preprocessor (includes, conditions, variables)
- [x] UUID format for IDs with namespaces
- [x] ECS update: locks, soft deletion of entities and reuse of released slots,
faster lookup for an entity by its ID
- [x] Tasks based assets loading, flexible API, custom assets
- [x] Constant FPS (if host machine is powerful enough)
- [x] GPU Driven PBR Rendering (Prototype)
- [x] Lights in storage buffers
- [ ] PBR materials using texture arrays
- [ ] New Camera crate
- [ ] New GLTF crate
- [ ] High-quality PBR rendering
- [ ] Shadows support
- [ ] Skinning within the new PBR
- [ ] Unit tests review
- [ ] API documentation
 
## Studio

**Dotrix Studio** is a binary that comes within the engine. At this moment it is
being used for proof-of-concept tests.

```
cargo run --release --bin studio
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
