mod controls;
mod editor;
mod mouse_ray;

use dotrix::{
    Dotrix,
    ecs::{ RunLevel, System },
    math::{ Point3 },
    services::{ Assets, Frame, Camera, Input, World },
    systems::{ overlay_update, world_renderer },
    input::{ ActionMapper, Button, Mapper },
    terrain,
};

use mouse_ray::{ MouseRay, mouse_ray };
use controls::{ Action };
use editor::{ Editor };

fn main() {
    Dotrix::application("Dotrix Editor")
        .with_system(System::from(editor::startup).with(RunLevel::Startup))
        .with_system(System::from(editor::camera_control))
        .with_system(System::from(overlay_update))
        .with_system(System::from(mouse_ray))
        .with_system(System::from(editor::ui))
        .with_system(System::from(terrain::spawn))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_service(Assets::new())
        .with_service(terrain::service())
        .with_service(Frame::new())
        .with_service(Editor::new())
        .with_service(MouseRay::default())
        .with_service(Camera {
            distance: 100.0,
            xz_angle: std::f32::consts::PI / 4.0,
            y_angle: std::f32::consts::PI / 2.0,
            target: Point3::new(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .with_service(World::new())
        .with_service(Input::new(Box::new(Mapper::<Action>::new())))
        .run();
}

impl ActionMapper<Action> for Input {
    fn action_mapped(&self, action: Action) -> Option<&Button> {
        let mapper = self.mapper::<Mapper<Action>>();
        mapper.get_button(action)
    }
}
