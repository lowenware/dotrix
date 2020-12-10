use cgmath::Vector2;
use dotrix::{
    Dotrix,
    assets::{ Mesh, Texture },
    components::{ Light, StaticModel, Transform },
    ecs::{ Mut, Const, RunLevel, System },
    services::{ Assets, Camera, Input, Frame, World },
    systems::{ static_renderer },
    input::{ ActionMapper, Button, KeyCode, Mapper, MouseButton },
};

fn main() {
    // Create controls to actions input mapper
    let mapper: Mapper<Action> = Mapper::new();

    Dotrix::application("Demo Example")
        .with_system(System::from(static_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(camera_control))
        .with_system(System::from(mappings_to_stdout))
        .with_service(Assets::new())
        .with_service(Camera::new(10.0, std::f32::consts::PI / 2.0, 4.0))
        .with_service(World::new())
        .with_service(Input::new(Box::new(mapper)))
        .run();
}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>, mut input: Mut<Input>) {
    use cgmath::Rotation3;
    assets.import("assets/crate.png", "crate");

    let texture = assets.register::<Texture>("crate");
    let cube1 = assets.store::<Mesh>(Mesh::cube(), "cube1");
    let cube2 = assets.store::<Mesh>(Mesh::cube(), "cube2");
    let transform = Transform {
        scale: Some(cgmath::Vector3::<f32>::new(2.0, 0.5, 0.8)),
        translate: Some(cgmath::Vector3::<f32>::new(3.5, 0.0, 1.0)),
        rotate: Some(cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI / 4.0))),
    };

    world.spawn(vec![
        (StaticModel::new(cube1, texture, transform),),
        (StaticModel::new(cube2, texture, Transform::default()),),
    ]);

    world.spawn(Some((Light::white([10.0, 5.0, 4.0]),)));

    // Populate input mappings
    input.mapper_mut::<Mapper<Action>>()
        .set(vec![
            (Action::Rotate, Button::Mouse(MouseButton::Left)),
            (Action::Jump, Button::Key(KeyCode::Space)),
            (Action::Run, Button::Key(KeyCode::W)),
            (Action::Spell1, Button::Key(KeyCode::Key1)),
            (Action::Spell2, Button::Mouse(MouseButton::Other(1))),
        ]);
}

fn camera_control(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
    let zoom_speed = 15.0;
    let rotation_speed_x = 0.2;
    let rotation_speed_y = 0.6;

    let zoom = -input.mouse_scroll() * zoom_speed * frame.delta().as_secs_f32();
    let rotation: Vector2<f32> = if input.is_action_hold(Action::Rotate) {
        Vector2::new(
            input.mouse_delta().x * rotation_speed_x * frame.delta().as_secs_f32(),
            input.mouse_delta().y * rotation_speed_y * frame.delta().as_secs_f32(),
        )
    } else {
        Vector2::new(0.0, 0.0)
    };

    let target = cgmath::Point3::new(0.0, 0.0, 0.0);
    let distance = camera.distance() + zoom;
    let angle = camera.angle() + rotation.x;
    let height = camera.height() + rotation.y;

    camera.set(target, distance, angle, height);
}

fn mappings_to_stdout(input: Const<Input>) {
    // Mouse Scroll
    let scroll = input.mouse_scroll();
    if scroll != 0.0 {
        println!("mouse scroll: {}", scroll)
    }

    // Mouse position
    if input.is_action_hold(Action::Rotate) {
        println!("Rotating camera, mouse position: {:?}; normalized: {:?}", input.mouse_position(), input.mouse_position_normalized());
    }

    // Actions
    if input.is_action_hold(Action::Run) {
        println!("Running!");
    }
    if input.is_action_activated(Action::Jump) {
        println!("Preparing for big jump!");
    }
    if input.is_action_deactivated(Action::Jump) {
        println!("Jumped!");
    }
    if input.is_action_activated(Action::Spell1) {
        println!("Casted spell 1!");
    }
    if input.is_action_activated(Action::Spell2) {
        println!("Casted spell 2!");
    }
}


#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
/// All bindable actions
enum Action {
    Rotate,
    Jump,
    Run,
    Spell1,
    Spell2,
}

impl ActionMapper<Action> for Input {
    fn action_mapped(&self, action: Action) -> Option<&Button> {
        let mapper = self.mapper::<Mapper<Action>>();
        mapper.get_button(action)
    }
}
