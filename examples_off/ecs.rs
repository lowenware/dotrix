//
// ecs.rs
// Copyright (C) 2020 Ilja Kartašov <ik@lowenware.com>
// Distributed under terms of the MIT license.
//

struct Health {}
struct Armor (u64);
struct Speed (u64);
struct Damage {}


use dotrix::{
    ecs::World,
};

fn main() {
    let mut world = World::new();
    let entities = (0..2).map(|_| (Health {}, Armor(100), Speed(50), Damage{}));

    println!("spawn");
    world.spawn(entities);
    println!("spawn single, 1");
    world.spawn(Some((Health {}, Armor (90), Speed (70))));
    println!("spawn single, 3");
    world.spawn(Some((Armor (200), Speed (20), Damage{})));
    println!("spawn single, 4");
    world.spawn(Some((Health {}, Damage{})));

    println!("select");
    world.select::<(Health, Damage)>();
    /*
    for i in world.select::<(Armor, Speed)>() {
        let (&armor, &speed) = i;
        println!("Match -> Armor: {}, Speed: {}", armor.0, speed.0);
    }
    */
}
