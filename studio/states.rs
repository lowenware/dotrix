use dotrix::ecs::Entity;
use dotrix::math::Vec3;
use dotrix::Mesh;
use dotrix::{log, pbr};

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

        let grass_mat = assets.store(dotrix::pbr::Material {
            albedo: dotrix::Color::rgb(0.595, 0.680, 0.280),
            ..Default::default()
        });

        let ground_mat = assets.store(dotrix::pbr::Material {
            albedo: dotrix::Color::rgb(0.294, 0.227, 0.196),
            ..Default::default()
        });

        let sand_mat = assets.store(dotrix::pbr::Material {
            albedo: dotrix::Color::rgb(0.376, 0.333, 0.310),
            ..Default::default()
        });

        let cube = assets.store(Mesh::cube(String::from("Cube")));

        let platform = [
            (
                Vec3::new(10.0, 10.0, 0.6),
                Vec3::new(-0.0, -0.0, -2.8),
                sand_mat,
            ),
            (
                Vec3::new(10.0, 10.0, 0.2),
                Vec3::new(-0.0, -0.0, -2.4),
                ground_mat,
            ),
            (
                Vec3::new(10.0, 10.0, 0.2),
                Vec3::new(-0.0, -0.0, -2.0),
                grass_mat,
            ),
        ];

        let entities = platform
            .into_iter()
            .map(|(scale, translate, material)| pbr::Entity {
                mesh: cube,
                scale,
                translate,
                material,
                ..Default::default()
            })
            .map(Entity::from)
            .collect::<Vec<_>>();

        world.spawn(entities);

        let objects = [
            (cube, Vec3::new(-6.0, 0.0, 0.0), materials[0]),
            (
                assets.store(Mesh::cylinder(String::from("Cylinder"), 8, Some(8))),
                Vec3::new(-3.0, 0.0, 0.0),
                materials[1],
            ),
            (cube, Vec3::new(0.0, -6.0, 0.0), materials[0]),
            (cube, Vec3::new(0.0, 0.0, -0.0), materials[0]),
            (
                assets.store(Mesh::sphere(String::from("Sphere"), 16, 16)),
                Vec3::new(0.0, 5.0, 0.0),
                materials[2],
            ),
            (
                assets.store(Mesh::cone(String::from("Cone"), 16)),
                Vec3::new(8.0, 0.0, 0.0),
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

        world.spawn([
            (dotrix::pbr::Light::ambient(1.0, 1.0, -3.0),),
            (dotrix::pbr::Light::ambient(-1.0, -1.0, -3.0)
                .intensity(0.1)
                .shadow(false),),
        ]);

        state.push(Execution {});
    }
}
