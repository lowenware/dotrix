//
// ecs.rs
// Copyright (C) 2020 Ilja Kartašov <ik@lowenware.com>
// Distributed under terms of the MIT license.
//

struct Health {}
struct Armor {}
struct Speed {}
struct Damage {}


use dotrix::{
    ecs::{World}
};
use dotrix::asset;

fn main() {
    let mut world = World::new();
    let mut entities = (0..9).map(|_| (Health {}, Armor {}));

    world.spawn(Some((Health {},)));
    world.spawn(entities);
    world.spawn(Some((Health {}, Armor {}, Speed {})));
    world.spawn(Some((Health {}, Armor {}, Speed {}, Damage{})));
}
