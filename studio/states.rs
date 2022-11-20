use dotrix::ecs::Entity;
use dotrix::math::Vec3;
use dotrix::{log, pbr};
use dotrix::{Mesh, World};

/// Execution state of the application
pub struct Execution {}

/// Startup task performs initial routines and enters the Execution state
pub struct Startup {}

impl Startup {
    pub fn new() -> Self {
        Self {}
    }
}

impl dotrix::Task for Startup {
    type Context = (
        dotrix::Mut<dotrix::Assets>,
        dotrix::Mut<dotrix::World>,
        dotrix::State<dotrix::Ref<()>>,
    );
    type Output = ();

    fn run(&mut self, (mut assets, mut world, state): Self::Context) -> Self::Output {
        log::info!("Starting Dotrix Studio");
        let materials = [
            assets.store(dotrix::pbr::Material {
                albedo: dotrix::Color::red(),
                ..Default::default()
            }),
            assets.store(dotrix::pbr::Material {
                albedo: dotrix::Color::green(),
                ..Default::default()
            }),
            assets.store(dotrix::pbr::Material {
                albedo: dotrix::Color::blue(),
                ..Default::default()
            }),
            assets.store(dotrix::pbr::Material {
                albedo: dotrix::Color::yellow(),
                ..Default::default()
            }),
        ];

        let objects = [
            (
                assets.store(Mesh::cube(String::from("Cube"))),
                Vec3::new(-2.0, 0.0, 0.0),
                materials[0],
            ),
            (
                assets.store(Mesh::cylinder(String::from("Cylinder"), 8, Some(8))),
                Vec3::new(-1.0, 0.0, 0.0),
                materials[1],
            ),
            (
                assets.store(Mesh::sphere(String::from("Sphere"), 16, 16)),
                Vec3::new(0.0, 0.0, 0.0),
                materials[2],
            ),
            (
                assets.store(Mesh::cone(String::from("Cone"), 16)),
                Vec3::new(-1.0, 0.0, 0.0),
                materials[3],
            ),
        ];

        let entities = objects
            .into_iter()
            .map(|(mesh, translate, material)| pbr::Entity {
                mesh,
                translate,
                material,
                ..Default::default()
            })
            .map(Entity::from)
            .collect::<Vec<_>>();

        world.spawn(entities);

        state.push(Execution {});
    }
}
