use dotrix::{camera, ecs::Mut, Camera, Dotrix, System, World};
use dotrix_voxel::{Grid, Light, TexSdf, VoxelJumpFlood};
use rand::Rng;

fn main() {
    Dotrix::application("Dotrix: Voxel Load")
        .with(System::from(startup))
        .with(System::from(camera::control))
        .with(dotrix_voxel::extension)
        .run();
}

fn startup(mut camera: Mut<Camera>, mut world: Mut<World>) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 30.0;
    camera.tilt = 0.0;

    // let values: Vec<u8> = vec![0u8; 16 * 16 * 16]
    //     .iter()
    //     .map(|&v| {
    //         let chance: u8 = rand::thread_rng().gen();
    //         if chance > 100 {
    //             rand::thread_rng().gen()
    //         } else {
    //             v
    //         }
    //     })
    //     .collect();

    let mut values: Vec<u8> = vec![0u8; 16 * 16 * 16];
    values[16 * 16 * 16 / 2] = 1;
    values[16 * 16 * 16 / 2 + 1] = 1;
    values[16 * 16 * 16 / 2 + 2] = 1;

    let grid = Grid::default().with_values(values);
    world.spawn(vec![(grid, VoxelJumpFlood::default(), TexSdf::default())]);

    world.spawn(Some((Light::Ambient {
        color: [0., 0., 0.].into(),
        intensity: 0.,
    },)));
    world.spawn(Some((Light::Directional {
        color: [1., 1., 1.].into(),
        direction: [100., -100., -100.].into(),
        intensity: 1.,
        enabled: true,
    },)));
}
