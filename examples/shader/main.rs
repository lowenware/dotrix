use dotrix::camera;
use dotrix::prelude::*;
use dotrix::{Assets, Camera, Pipeline, Renderer, World};

mod cube;
mod shader;

use shader::Gradient;

fn main() {
    Dotrix::application("Dotrix: Shader Example")
        .with(System::from(startup))
        .with(System::from(camera::control))
        .with(shader::extension)
        .run();
}

fn startup(
    mut camera: Mut<Camera>,
    mut world: Mut<World>,
    mut assets: Mut<Assets>,
    renderer: Const<Renderer>,
) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 2.0;
    camera.xz_angle = 0.0;

    let mut mesh = cube::cube(0.5);

    mesh.load(&renderer);

    let mesh_handle = assets.store(mesh);

    // Spawn skybox
    world.spawn(Some((
        Gradient {
            nadir_color: [1., 0., 0.4, 1.],
            zenith_color: [0.2, 0.4, 0.8, 1.],
            mesh: mesh_handle,
        },
        Pipeline::default(),
    )));
}
