use dotrix::prelude::*;
use dotrix::input::{ Input, Button, State as InputState };
use dotrix::ray::Ray;
use dotrix::terrain::{ Map, HeightMap };
use dotrix::math::Vec3;


pub fn update(
    ray: Const<Ray>,
    input: Const<Input>,
    mut map: Mut<Map>,
) {
    if input.button_state(Button::MouseLeft) != Some(InputState::Hold) {
        return;
    }

    /*
    if let Some(intersection) = terrain.ray_intersection(&ray) {
        if let Some(heightmap) = terrain.heightmap_mut::<HeightMap>() {
            let size = heightmap.size() as f32;
            let offset = size as f32 / 2.0;
            let x = intersection.x + offset;
            let z = intersection.z + offset;

            if x < 0.0 || x >= size || z < 0.0 || z >= size {
                return;
            }

            let x = x.round() as usize;
            let z = z.round() as usize;

            if let Some(value) = heightmap.get(x, z) {
                heightmap.set(x, z, value + 256);
                terrain.force_spawn = true;
            }
        }
    }
    */
}

