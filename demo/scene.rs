use dotrix::log;
use dotrix::math::Vec3;
use dotrix::{Id, Mut};

// NOTE: For one time spawn separate task is not optimal. In that case it would be better
// to spawn everything before adding the World context. But one of purposes of this demo is to
// suggest possible application architecture. In that case just think about `spawned` property of
// `SpawnEntities` task as of some condition under which the application should spawn or remove
// entity.

#[derive(Default)]
pub struct SpawnEntities {
    spawned: Id<dotrix::Entity>,
}

#[derive(Default)]
pub struct Scene;

impl dotrix::Task for SpawnEntities {
    type Context = (Mut<dotrix::Assets>, Mut<dotrix::World>);

    type Output = Scene;

    fn run(&mut self, (mut assets, mut world): Self::Context) -> Self::Output {
        if self.spawned.is_null() {
            let cube = assets.set(dotrix::Mesh::cube("Cube"));
            let red_material = assets.set(dotrix::Material {
                name: String::from("Red Material"),
                albedo: dotrix::Color::red(),
                ..Default::default()
            });
            let blue_material = assets.set(dotrix::Material {
                name: String::from("Blue Material"),
                albedo: dotrix::Color::blue(),
                ..Default::default()
            });
            let entities = [
                dotrix::Model {
                    mesh: cube,
                    material: red_material,
                    scale: Vec3::new(0.5, 0.5, 0.5),
                    ..Default::default()
                },
                dotrix::Model {
                    mesh: cube,
                    material: blue_material,
                    scale: Vec3::new(0.3, 0.3, 0.3),
                    translate: Vec3::new(1.0, 0.0, -0.4),
                    ..Default::default()
                },
            ]
            .into_iter()
            .map(dotrix::Entity::from);

            self.spawned = world
                .spawn(entities)
                .last()
                .expect("The entity must be spawned");

            log::info!("Spawn scene entities: {:?}", self.spawned);
        }
        Scene
    }
}
