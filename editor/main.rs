mod brush;
mod controls;
mod editor;

use dotrix::{
    Dotrix, Display,
    ecs::{ RunLevel, System },
    math::{ Point3 },
    services::{ Assets, Frame, Camera, Input, World },
    systems::{ overlay_update, world_renderer },
    input::{ ActionMapper, Button, Mapper, Ray, mouse_ray },
    terrain,
};

use controls::{ Action };
use editor::{ Editor };

fn main() {
    Dotrix::application("Dotrix Editor")
        .with_display(Display { clear_color: [1.00, 1.00, 1.00, 1.0], ..Default::default()})
        .with_system(System::from(editor::startup).with(RunLevel::Startup))
        .with_system(System::from(editor::camera_control))
        .with_system(System::from(overlay_update))
        .with_system(System::from(mouse_ray))
        .with_system(System::from(editor::ui))
        .with_system(System::from(terrain::spawn))
        .with_system(System::from(brush::picker))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_service(Assets::new())
        .with_service(terrain::service())
        .with_service(Frame::new())
        .with_service(Editor::new())
        .with_service(Ray::default())
        .with_service(Camera {
            distance: 400.0,
            xz_angle: std::f32::consts::PI / 2.0,
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
