use dotrix::assets::Mesh;
use dotrix::camera;
use dotrix::ecs::Mut;
use dotrix::math::Vec3;
use dotrix::pbr::{self, Light};
use dotrix::{Assets, Camera, Color, Dotrix, System, World};

fn main() {
    Dotrix::application("Dotrix: PBR")
        .with(System::from(startup))
        .with(System::from(camera::control))
        .with(pbr::extension)
        .run();
}

fn startup(mut camera: Mut<Camera>, mut world: Mut<World>, mut assets: Mut<Assets>) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 30.0;
    camera.tilt = 0.0;

    assets.import("assets/models/sphere.gltf");
    let mesh_handle = assets.register::<Mesh>("sphere::mesh");

    let mut spheres = vec![];
    for i in 0..11 {
        let x = i as f32 * 3.0 - (10. * 3. / 2.);
        let metallic = i as f32 * 0.1;
        for j in 0..11 {
            let y = j as f32 * 3.0 - (10. * 3. / 2.);
            let roughness = j as f32 * 0.1;
            let obj = (pbr::solid::Entity {
                mesh: mesh_handle,
                albedo: [0.2, 0.8, 0.2, 1.].into(),
                roughness,
                metallic,
                translate: Vec3::new(x, y, 0.),
                ..Default::default()
            })
            .tuple();
            spheres.push(obj);
        }
    }
    world.spawn(spheres);

    // Spawn lights
    world.spawn(vec![(Light::Directional {
        direction: Vec3::new(1.0, 1.0, 10.0),
        color: Color::rgb(1.0, 1.0, 1.0),
        intensity: 3.0,
        enabled: true,
    },)]);
}
