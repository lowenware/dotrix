use dotrix::{
    assets::{ Mesh },
    components::{ Light, Model },
    ecs::{ Mut, Const },
    egui::Egui,
    math::{ Vec3 },
    renderer::{ Transform },
    services::{ Assets, World, Overlay, Renderer },
    terrain::{
        MarchingCubes,
    }
};

use crate::editor::Editor;
use noise::{ NoiseFn, Fbm, /* Perlin, Turbulence, Seedable, */ MultiFractal };

pub fn startup(
    mut assets: Mut<Assets>,
    _editor: Const<Editor>,
    mut renderer: Mut<Renderer>,
    mut world: Mut<World>,
) {
    renderer.overlay = Some(Overlay::new(Box::new(Egui::default())));

    let mc = MarchingCubes::new();
    let noise = Fbm::new();
    let noise = noise.set_octaves(3);
    let noise = noise.set_frequency(1.0);
    let noise = noise.set_lacunarity(2.0);
    let noise = noise.set_persistence(0.5);

    let (positions, _) = mc.polygonize(|x, y, z| {
        let div_h = mc.size as f64 / 16.0;
        let div_v = mc.size as f64 / 8.0;
        let value = div_v * noise.get([
            (x as f64 / div_h) + 0.5,
            (y as f64 / div_h) + 0.5,
            (z as f64 / div_h) + 0.5,
        ]) - (y as f64);
        (if value < -(y as f64) { -(y as f64) } else { value }) as f32
    });

    let len = positions.len();
    let mut mesh = Mesh {
        positions,
        uvs: Some(vec![[0.0, 0.0]; len]),
        ..Default::default()
    };
    mesh.calculate();

    let mesh = assets.store(mesh, "Terrain");

    let half_size = -((mc.size / 2) as f32);
    let transform = Transform {
        translate: Vec3::new(half_size, 0.0, half_size),
        ..Default::default()
    };

    let texture = assets.register("gray");
    assets.import("editor/assets/gray.png");

    world.spawn(Some(
        (Model { mesh, transform, texture, ..Default::default() },)
    ));

    world.spawn(Some((Light::white([0.0, 500.0, 0.0]),)));
}
