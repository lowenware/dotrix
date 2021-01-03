use dotrix::{
    assets::{ Id, Mesh },
    components::{ Model },
    ecs::{ Mut, Context },
    math::{ Vec3 },
    renderer::{ Transform },
    services::{ Assets, World },
    terrain::{
        MarchingCubes,
    }
};

use crate::editor::Editor;
use noise::{ NoiseFn, Fbm, /* Perlin, Turbulence, Seedable, */ MultiFractal };

#[derive(Default)]
pub struct Terrain {
    mesh: Option<Id<Mesh>>,
}

pub fn draw(
    mut ctx: Context<Terrain>,
    mut assets: Mut<Assets>,
    mut editor: Mut<Editor>,
    mut world: Mut<World>,
) {
    if !editor.changed {
        return;
    }
    editor.changed = false;

    let mc = MarchingCubes {
        size: editor.chunk_size,
        ..Default::default()
    };
    let noise = Fbm::new();
    let noise = noise.set_octaves(editor.octaves);
    let noise = noise.set_frequency(editor.frequency);
    let noise = noise.set_lacunarity(editor.lacunarity);
    let noise = noise.set_persistence(editor.persistence);

    let (positions, _) = mc.polygonize(|x, y, z| {
        let div_h = editor.xz_div;
        let div_v = editor.y_div;
        let value = div_v * noise.get([
            (x as f64 / div_h) + 0.5,
            (y as f64 / div_h) + 0.5,
            (z as f64 / div_h) + 0.5,
        ]) - (y as f64);
        (if value < -(y as f64) { -(y as f64) } else { value }) as f32
    });

    let len = positions.len();
    let uvs = Some(vec![[0.0, 0.0]; len]);

    if let Some(mesh_id) = ctx.mesh {
        let mesh = assets.get_mut(mesh_id).unwrap();
        mesh.positions = positions;
        mesh.uvs = uvs;
        mesh.normals.take();
        mesh.calculate();
        mesh.unload();

    } else {
        let mut mesh = Mesh {
            positions,
            uvs,
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

        ctx.mesh = Some(mesh);
    }

}
