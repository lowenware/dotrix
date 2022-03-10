use dotrix::{camera, ecs::Mut, Camera, Dotrix, System, World};
use dotrix_voxel::{Grid, VoxelJumpFlood};

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

    let mut values = vec![0u8; 16 * 16 * 16];
    values[30] = 1;
    values[31] = 1;
    values[32] = 1;
    values[33] = 1;
    values[40] = 1;
    values[41] = 1;
    values[42] = 1;
    values[43] = 1;
    let grid = Grid::default().with_values(values);
    world.spawn(vec![(grid, VoxelJumpFlood::default())]);
}
