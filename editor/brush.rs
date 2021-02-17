use dotrix::{
    ecs::{ Const, Mut },
    components::{ Model },
    services::{ Assets, Ray, World },
    terrain::{ Block },
    math::{ Vec3 },
};

use crate::editor::Editor;

pub fn picker(
    mut editor: Mut<Editor>,
    assets: Const<Assets>,
    ray: Const<Ray>,
    world: Const<World>,
) {

    let gray = assets.find("terrain").unwrap();
    let red = assets.find("red").unwrap();

    let query = world.query::<(&mut Model, &Block)>();
    editor.picked_block = None;
    for (model, block) in query {
        if model.disabled { continue; }
        let bounds = [
            Vec3::new(
                block.bound_min.x as f32,
                block.bound_min.y as f32,
                block.bound_min.z as f32
            ),
            Vec3::new(
                block.bound_max.x as f32,
                block.bound_max.y as f32,
                block.bound_max.z as f32
            ),
        ];
        let texture = if let Some((t_min, t_max)) = ray.intersect_box(bounds) {
            editor.picked_block = Some(block.position);
            // println!("Intersects in {:?} and {:?}", t_min, t_max);
            red
        } else {
            gray
        };

        if model.texture != texture {
            model.texture = texture;
            model.buffers = None;
        }
    }

}
