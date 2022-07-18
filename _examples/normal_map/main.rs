use dotrix::assets::{Mesh, Texture};
use dotrix::camera;
use dotrix::ecs::Mut;
use dotrix::math::Vec3;
use dotrix::pbr::{self, Light};
use dotrix::{Assets, Camera, Color, Dotrix, System, World};

fn main() {
    Dotrix::application("Dotrix: Normal Map")
        .with(System::from(startup))
        .with(System::from(camera::control))
        .with(pbr::extension)
        .run();
}

fn startup(mut camera: Mut<Camera>, mut world: Mut<World>, mut assets: Mut<Assets>) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 3.0;
    camera.tilt = 0.0;

    assets.import("assets/models/sphere.gltf");
    let mesh_handle = assets.register::<Mesh>("sphere::mesh");

    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_AmbientOcclusion.jpg");
    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_Color.jpg");
    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_NormalDX.jpg");
    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_Roughness.jpg");

    let ao = assets.register::<Texture>("Bricks076C_1K_AmbientOcclusion");
    let albedo = assets.register::<Texture>("Bricks076C_1K_Color");
    let normal = assets.register::<Texture>("Bricks076C_1K_NormalDX");
    let roughness = assets.register::<Texture>("Bricks076C_1K_Roughness");

    let spheres = vec![
        // Only color
        (pbr::solid::Entity {
            mesh: mesh_handle,
            texture: albedo,
            translate: Vec3::new(0., 0., 0.),
            ..Default::default()
        })
        .tuple(),
        // Color, pbr roughness/ao and normal map
        (pbr::solid::Entity {
            mesh: mesh_handle,
            texture: albedo,
            roughness_texture: roughness,
            ao_texture: ao,
            normal_texture: normal,
            translate: Vec3::new(-2., 0., 0.),
            ..Default::default()
        })
        .tuple(),
        // Color and pbr roughness/ao
        (pbr::solid::Entity {
            mesh: mesh_handle,
            texture: albedo,
            roughness_texture: roughness,
            ao_texture: ao,
            translate: Vec3::new(2., 0., 0.),
            ..Default::default()
        })
        .tuple(),
    ];
    world.spawn(spheres);

    // Spawn lights
    world.spawn(vec![(Light::Directional {
        direction: Vec3::new(1.0, 1.0, 10.0),
        color: Color::rgb(1.0, 1.0, 1.0),
        intensity: 3.0,
        enabled: true,
    },)]);
}
