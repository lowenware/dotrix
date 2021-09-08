# Changelog

## v0.5.0 / 2021-09-07

* **NEW:** Added prelude module. Add `use dotrix::prelude::*` to include `Dotrix`,
`Mut`, `Const`, `Context`, `System` and `Service` to your namespace.

* **NEW:** `Dotrix::application(...)` now includes all core services
(Assets, Camera, Frame, Input, Globals, Renderer, Window, World) and appropriate
systems in the background.

* **NEW:** Added `Dotrix::bare(...)` function, that initializes empty application,
as it was before with `Dotrix::application(...)`. When migrate old code to newer
engine version it is recommended to keep using `Dotrix::application(...)` and
remove manually added services and systems.

* **NEW:** Added crate `dotrix_pbr` that can be accessible using `dotrix::pbr`.
It contain everything necessary to perform physically based rendering of
solid and skeletal models.

* **NEW:** Spawning of PBR entities has been changed, due to refactoring of `Model`
component, that was decomposed into multiple components. Entity representing a
solid PBR model requires following components: `Model`, `Material`, `Transform`,
`Pipeline`. Entity representing a skeletal model also requires `Pose` component.

* **NEW:** Added crate `dotrix_overlay`. Appropriate functionality has been removed
from renderer. The new service `Overlay` has been introduced instead.

```
/// Show FPS counter
fn ui(overlay: Const<Overlay>, frame: Const<Frame>) {
    let egui_overlay = overlay.get::<Egui>()
        .expect("Egui overlay must be added on startup");

    egui::Area::new("FPS counter")
        .fixed_pos(egui::pos2(16.0, 16.0))
        .show(&egui_overlay.ctx, |ui| {
            ui.colored_label(
                egui::Rgba::from_rgb(255.0, 255.0, 255.0),
                format!("FPS: {:.1}", frame.fps())
            );
        });
}
```

* **NEW:** Added crate `dotrix_sky` and skybox implementation has been moved there.
Refer `skybox` example code for more details about how to use it.

* **NEW:** Added crate `dotrix_terrain` for height map based terrain rendering.
Appropriate example will be added in future releases.

* **NEW:** Added engine extensions mechanism. To enable some extra features, like
PBR rendering, that may require enabling of several systems and services, it is
the easies way to use extensions:

```
use dotrix::prelude::*;
use dotrix::egui;
use dotrix::pbr;
use dotrix::overlay;

fn main() {
    Dotrix::application("With Extensions")
        .with(pbr::extension)
        .with(overlay::extension)
        .with(egui::extension)
        .run();
}
```

The following modules provide `extension` functions:
`dotrix::{ egui, pbr, sky, terrain }`. The order of extensions in application
builder does not matter.

* **NEW:** The method `.with(..)` of application  constructor can be used to add
everything now:
```
    Dotrix::application("Mighty With")
        .with(pbr::extension)
        .with(System::from(my_system))
        .with(Service::from(MyService::default()))
        .run();
```

* **NEW:** Added `Globals` service that can store and provide access to global
application data. It is useful, when you don't want to have extra services in
your app. It is now used to store `ProjView`, `Lights`, texture `Sampler`
and terrain `Layers`.

* **NEW:** Multiple `Light` components has been turned into enumeration of
structures to optimize access over ECS.

* **NEW:** Added auto detection of a system run level. If system is named after
a certain run level, it will be automatically added to it.

* **NEW:** Added prioritization of systems. It allows to influence the execution
order without dependency on the order of enabling systems in your code.
```
Dotrix::application("Priorities")
    .with(System::from(execute_last))
    .with(System::from(execute_first).with(Priority::High))
    .run();
```

* **NEW:** Camera service is no longer requires to be set manually. Developer
can directly change properties in systems. The engine will do the rest. To
initialize camera parameters use `startup` system.

* **NEW:** The biggest change of the release is rendering mechanism. There is
no longer a dependency on WGPU code in your rendering systems. Own wrappers has
been added instead. It allowed to make Dotrix even more modular and scalable.
There is no longer a need to create layout and pipeline separately. Developer
can define, what kind of pipeline layout is necessary for specific rendering
and Dotrix will create a new pipeline or reuse the previously created.

* **NEW:** Dotrix has been completely switched to WGSL shaders.

* **NEW:** Added new asset type `Shader`. 

* **FIXED:** It is now possible to use custom textures stored in `Assets` with
`egui`.

* **FIXED:** Texture Sampler now solves tiling properly. It is also possible to
customize Sampler or even have more Samplers in your code.

* **UPDATE:** Egui dependency has been updated to version 0.14.2.

* **UPDATE:** WGPU dependency has been updated to version 0.9.0.

* **UPDATE:** Winit dependency has been updated to version 0.25.0.
