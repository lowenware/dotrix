use dotrix::{camera, ecs::Mut, Camera, Dotrix, System, World};

fn main() {
    Dotrix::application("Dotrix: Voxel Load")
        .with(System::from(startup))
        .with(System::from(camera::control))
        .run();
}

fn startup(mut camera: Mut<Camera>, mut world: Mut<World>) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 30.0;
    camera.tilt = 0.0;
}
