use crate::{
    application::Application,
    ecs::{RunLevel, System, Systemized},
};

pub struct Systems {
    render: Vec<Box<dyn Systemized>>,
    standard: Vec<Box<dyn Systemized>>,
    startup: Vec<Box<dyn Systemized>>,
}

impl Systems {
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

    pub fn run_render(&mut self, app: &mut Application) {
        for system in &mut self.render {
            system.run(app);
        }
    }

    pub fn run_standard(&mut self, app: &mut Application) {
        for system in &mut self.standard {
            system.run(app);
        }
    }

    pub fn run_startup(&mut self, app: &mut Application) {
        for system in &mut self.startup {
            system.run(app);
        }
    }
}
