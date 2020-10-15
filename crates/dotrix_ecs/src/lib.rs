use std::vec::Vec;

pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}

pub struct World {
    entities: Vec<Entity>,



    id_generator: u64,
}

impl World {
    pub fn new() -> Self {
        Self {

        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
