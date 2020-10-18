mod container;
mod ecs;
mod world;

pub use ecs::*;
pub use world::*;

/// Count parameters
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt, $($xs:tt)* ) => (1usize + count!($($xs)*));
}

/// Recursive macro treating arguments as a progression
///
/// Expansion of recursive!(macro, A, B, C) is equivalent to the expansion of sequence
/// macro!(A)
/// macro!(A, B)
/// macro!(A, B, C)
#[macro_export]
macro_rules! recursive {
    ($macro: ident, $args: ident) => {
        $macro!{$args}
    };
    ($macro: ident, $first: ident, $($rest: ident),*) => {
        $macro!{$first, $($rest),*}
        recursive!{$macro, $($rest),*}
    };
}

#[cfg(test)]
mod tests {
    use crate::World;

    struct Armor(u32);
    struct Health(u32);
    struct Speed(u32);
    struct Damage(u32);
    struct Weight(u32);

    fn spawn() -> World {
        let mut world = World::new();
        world.spawn(Some((Armor(100), Health(100), Damage(300))));
        world.spawn(Some((Health(80), Speed(10))));
        world.spawn(Some((Speed(50), Damage(45))));
        world.spawn(Some((Damage(600), Armor(10))));

        let bulk = (0..9).map(|_| (Speed(35), Weight(5000)));
        world.spawn(bulk);

        world
    }

    #[test]
    fn spawn_and_query() {
        let world = spawn();

        let mut iter = world.query::<(Armor, Damage)>();

        let item = iter.next();
        assert_eq!(item.is_some(), true);

        let item = item.unwrap();
        assert_eq!(item.0.0, 100); // Armor(100)
        assert_eq!(item.1.0, 300); // Damage(300)

        let item = iter.next();
        assert_eq!(item.is_some(), true);

        let item = item.unwrap();
        assert_eq!(item.0.0, 10); // Armor(10)
        assert_eq!(item.1.0, 600); // Damage(600)

        let item = iter.next();
        assert_eq!(item.is_some(), false);
    }
}
