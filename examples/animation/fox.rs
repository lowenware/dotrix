// The Fox model used in this example is taken from Khronos repository
// https://github.com/KhronosGroup/glTF-Sample-Models
// under following License:
// CC0: Low poly fox by PixelMannen
// CC-BY 4.0: Rigging and animation by @tomkranis on Sketchfab
// glTF conversion by @AsoboStudio and @scurest
//

use dotrix::assets::{Animation, Mesh, Skin, Texture};
use dotrix::ecs::Mut;
use dotrix::pbr::{Material, Model};
use dotrix::{Animator, Assets, Id, Pipeline, Pose, Transform, World};

use std::collections::HashMap;

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
    assets.import("assets/models/Fox.gltf");

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

    world.spawn(Some((
        Model::from(mesh),
        Pose::from(skin),
        Material {
            texture,
            ..Default::default()
        },
        Transform::default(),
        Animator::looped(walk),
        Pipeline::default(),
        Fox { animations },
    )));
}
