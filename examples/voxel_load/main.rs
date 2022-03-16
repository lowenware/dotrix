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

    let mut grid = Grid::default();

    let dims = grid.dimensions;
    let total_size: usize = dims.iter().fold(1usize, |acc, &item| acc * (item as usize));
    let values: Vec<u8> = vec![0u8; total_size]
        .iter()
        .map(|_v| {
            let chance: u8 = rand::thread_rng().gen();
            if chance > 128 {
                1
            } else {
                0
            }
        })
        .collect();
    //
    // let mut values: Vec<u8> = vec![0u8; 3 * 3 * 3];
    // values[13] = 3;
    // values[1] = 1;
    // values[2] = 1;
    // values[16 * 16 * 16 / 2] = 1;
    // values[16 * 16 * 16 / 2 + 1] = 1;
    // values[16 * 16 * 16 / 2 + 2] = 1;
    //
    // let values: Vec<u8> = vec![1u8; 16 * 16 * 16];

    // let values: Vec<u8> = vec![
    //     0, 0, 0, //
    //     0, 0, 0, //
    //     0, 0, 0, //
    //     //
    //     0, 0, 0, //
    //     0, 1, 0, //
    //     0, 0, 0, //
    //     //
    //     0, 0, 0, //
    //     0, 0, 0, //
    //     0, 0, 0, //
    // ];

    let formatted_values: Vec<Vec<Vec<u8>>> = values
        .chunks((dims[0] * dims[1]) as usize)
        .map(|img| {
            let rows: Vec<Vec<u8>> = img
                .chunks(dims[0] as usize)
                .map(|rows| rows.to_vec())
                .collect();
            rows
        })
        .collect();
    println!("Voxels: {:?}", formatted_values);

    grid = grid.with_values(values);
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
