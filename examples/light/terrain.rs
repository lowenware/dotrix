use dotrix::{
    assets::{ Mesh },
    components:: { Model },
    ecs::{ Mut },
    math::{ Vec3 },
    renderer::{ Transform },
    services::{ Assets, World },
};

const TERRAIN_SIZE: usize = 128; // Number of sqaures per side

pub fn init(mut world: Mut<World>, mut assets: Mut<Assets>) {
    // Generate terrain mesh like this:
    //   0   1
    // 0 +---+---+---> x
    //   | / | / |
    // 1 +---+---+
    //   | / | / |
    //   +---+---+
    //   |
    //   z

    let size = TERRAIN_SIZE;
    let mut positions = Vec::with_capacity(3 * 2 * size * size);
    let mut uvs = Vec::new();
    for x in 0..size {
        let x0 = x as f32;
        let x1 = x0 + 1.0;
        for z in 0..size {
            let z0 = z as f32;
            let z1 = z0 + 1.0;
            // Add vertices
            positions.push([x0, 0.0, z0]);
            positions.push([x0, 0.0, z1]);
            positions.push([x1, 0.0, z0]);
            positions.push([x1, 0.0, z0]);
            positions.push([x0, 0.0, z1]);
            positions.push([x1, 0.0, z1]);
            // Add texture vertices
            uvs.push([0.0, 0.0]);
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 1.0]);
        }
    }

    let mut mesh = Mesh {
        positions,
        uvs: Some(uvs),
        ..Default::default()
    };
    // Calculate mesh normals
    mesh.calculate();

    // Store mesh and get its ID
    let mesh = assets.store_as(mesh, "terrain");

    // import terrain texture and get its ID
    assets.import("examples/demo/terrain.png");
    let texture = assets.register("terrain");

    // Center terrain tile at coordinate system center (0.0, 0.0, 0.0) by moving the tile on a
    // half of its size by X and Z axis
    let shift = (size / 2) as f32;
    let transform = Transform {
        translate: Vec3::new(-shift, 0.0, -shift),
        ..Default::default()
    };

    // Spawn terrain in the world
    world.spawn(Some(
        (Model { mesh, texture, transform, ..Default::default() },)
    ));
}
