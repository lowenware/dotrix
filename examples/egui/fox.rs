// The Fox model used in this example is taken from Khronos repository
// https://github.com/KhronosGroup/glTF-Sample-Models
// under following License:
// CC0: Low poly fox by PixelMannen
// CC-BY 4.0: Rigging and animation by @tomkranis on Sketchfab
// glTF conversion by @AsoboStudio and @scurest
//

use dotrix::{
    assets::{ Animation, Id, Mesh, Skin, Texture },
    components::{ Animator, Model },
    ecs::{ Mut },
    services::{ Assets, World },
};
use std::{
    collections::HashMap,
    vec,
};

 /// Component indentifying fox's entity
 pub struct Fox {
    /// This is only for editor - to know what animations fox have
    pub animations: HashMap<FoxAnimClip, Id<Animation>>,
}

#[derive(Debug, PartialEq, std::cmp::Eq, std::hash::Hash)]
pub enum FoxAnimClip {
    Walk,
    Run,
    Survey,
}

pub fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    assets.import("examples/egui/Fox.gltf");

    let texture = assets.register::<Texture>("Fox::fox::texture");
    let mesh = assets.register::<Mesh>("Fox::fox::mesh");
    let skin = assets.register::<Skin>("Fox::fox::skin");

    let walk = assets.register::<Animation>("Fox::Walk");
    let run = assets.register::<Animation>("Fox::Run");
    let survey = assets.register::<Animation>("Fox::Survey");

    let mut animations: HashMap<FoxAnimClip, Id<Animation>> = HashMap::new();
    animations.insert(FoxAnimClip::Walk, walk);
    animations.insert(FoxAnimClip::Run, run);
    animations.insert(FoxAnimClip::Survey, survey);

    world.spawn(vec![
        (
            Model {
                mesh,
                texture,
                skin,
                ..Default::default()
            },
            Animator::looped(walk),
            Fox { animations },
        ),
    ]);
}

