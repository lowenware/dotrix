use std::vec::Vec;

pub enum State {
    Busy,
    Ready,
    Fail,
}

pub struct Resource {
    name: String,
    path: String,
    state: State,
    data: Option<Vec<u8>>,
}

impl Resource {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            state: State::Busy,
            data: None,
        }
    }
}

