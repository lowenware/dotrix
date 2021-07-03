use dotrix::{ Assets, World };
use dotrix::assets::{ Mesh };
use dotrix::ecs::Mut;
use dotrix::pbr;
use dotrix::math::Vec3;

const TERRAIN_SIZE: usize = 128; // Number of sqaures per side

pub fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
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

    let normals = Mesh::calculate_normals(&positions, None);

    let mut mesh = Mesh::default();

    mesh.with_vertices(&positions);
    mesh.with_vertices(&normals);
    mesh.with_vertices(&uvs);

    // Store mesh and get its ID
    let mesh = assets.store_as(mesh, "terrain");

    // import terrain texture and get its ID
    assets.import("assets/textures/terrain.png");
    let texture = assets.register("terrain");

    // Center terrain tile at coordinate system center (0.0, 0.0, 0.0) by moving the tile on a
    // half of its size by X and Z axis
    let shift = (size / 2) as f32;

    world.spawn(
        (pbr::solid::Entity {
            mesh,
            texture,
            translate: Vec3::new(-shift, 0.0, -shift),
            ..Default::default()
        }).some()
    );
}
