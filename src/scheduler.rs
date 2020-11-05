use crate::{
    services::Services,
    ecs::{RunLevel, System, Systemized},
};

pub struct Scheduler {
    render: Vec<Box<dyn Systemized>>,
    standard: Vec<Box<dyn Systemized>>,
    startup: Vec<Box<dyn Systemized>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            render: Vec::new(),
            standard: Vec::new(),
            startup: Vec::new(),
        }
    }

    pub fn add(&mut self, system: System) {
        let (data, run_level) = system.tuple();
        match run_level {
            RunLevel::Render => self.render.push(data),
            RunLevel::Standard => self.standard.push(data),
            RunLevel::Startup => self.startup.push(data),
        };
    }

    pub fn run_render(&mut self, services: &mut Services) {
        for system in &mut self.render {
            system.run(services);
        }
    }

    pub fn run_standard(&mut self, services: &mut Services) {
        for system in &mut self.standard {
            system.run(services);
        }
    }

    pub fn run_startup(&mut self, services: &mut Services) {
        for system in &mut self.startup {
            system.run(services);
        }
    }
}
