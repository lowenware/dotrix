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
    let entities = (0..9).map(|_| (Health {}, Armor {}));

    println!("spawn array, 2");
    world.spawn(entities);
    println!("spawn single, 1");
    world.spawn(Some((Health {},)));
    println!("spawn single, 3");
    world.spawn(Some((Health {}, Armor {}, Speed {})));
    println!("spawn single, 4");
    world.spawn(Some((Health {}, Armor {}, Speed {}, Damage{})));
}
