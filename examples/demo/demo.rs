use dotrix::{
    Dotrix,
    assets::{ Mesh },
    ecs::{ Const, Mut, RunLevel, System },
    components::{ Animator, Light, Model, SkyBox },
    services::{ Assets, Camera, Frame, Input, World },
    systems::{ camera_control, skeletal_animation, world_renderer },
    renderer::transform::Transform,
    input::{ ActionMapper, Button, KeyCode, Mapper },
    math::{ Point3, Quat, Rotation3, Vec3, Rad },
};

use std::f32::consts::PI;

const CAMERA_HEIGHT: f32 = 2.0; // Default camera target feight
const TERRAIN_SIZE: usize = 128; // Number of sqaures per side

fn main() {

    Dotrix::application("Demo")
        // Rendering system draws models and skybox on a window surface
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        // Startup systems are being used to perform game initiation
        .with_system(System::from(startup).with(RunLevel::Startup))
        // Built-in camera control system with mouse look and zoom function
        .with_system(System::from(camera_control))
        // System controlling player's model
        .with_system(System::from(player_control))
        // System animating skinned models using Animator control component
        .with_system(System::from(skeletal_animation))
        // Service responsible for assets load and storage
        .with_service(Assets::new())
        // Service implementing camera
        .with_service(Camera {
            y_angle: PI / 2.0,
            xz_angle: 0.0,
            target: Point3::new(0.0, CAMERA_HEIGHT, 0.0),
            distance: 5.0,
            ..Default::default()
        })
        // Frame measurement service
        .with_service(Frame::new())
        // Input service uses Mapper to bind Actions to buttons
        .with_service(Input::new(Box::new(Mapper::<Action>::new())))
        // World service implements storage for game entities and query mechanism
        .with_service(World::new())
        // Start application
        .run();
}

/// Initial game routines
fn startup(
    mut world: Mut<World>,
    mut assets: Mut<Assets>,
    mut input: Mut<Input>,
) {
    init_skybox(&mut world, &mut assets);
    init_terrain(&mut world, &mut assets);
    init_light(&mut world);
    init_player(&mut world, &mut assets, &mut input);
}

fn init_skybox(
    world: &mut World,
    assets: &mut Assets,
) {
    // Import skybox textures
    assets.import("examples/demo/skybox_right.png");
    assets.import("examples/demo/skybox_left.png");
    assets.import("examples/demo/skybox_top.png");
    assets.import("examples/demo/skybox_bottom.png");
    assets.import("examples/demo/skybox_back.png");
    assets.import("examples/demo/skybox_front.png");

    // Get slice with textures
    let primary_texture = [
        assets.register("skybox_right"),
        assets.register("skybox_left"),
        assets.register("skybox_top"),
        assets.register("skybox_bottom"),
        assets.register("skybox_back"),
        assets.register("skybox_front"),
    ];

    // Spawn skybox
    world.spawn(Some(
        (SkyBox { primary_texture, ..Default::default() },),
    ));
}

fn init_terrain(
    world: &mut World,
    assets: &mut Assets,
) {
    // Generate terrain mesh like this:
    //   0   1
    // 0 +---+---+---> x
    //   | / | / |
    // 1 +---+---+
    //   | / | / |
    //   +---+---+
    //   |
    //   z

    let size = TERRAIN_SIZE;
    let mut positions = Vec::with_capacity(3 * 2 * size * size);
    let mut uvs = Vec::new();
    for x in 0..size {
        let x0 = x as f32;
        let x1 = x0 + 1.0;
        for z in 0..size {
            let z0 = z as f32;
            let z1 = z0 + 1.0;
            // Add vertices
            positions.push([x0, 0.0, z0]);
            positions.push([x0, 0.0, z1]);
            positions.push([x1, 0.0, z0]);
            positions.push([x1, 0.0, z0]);
            positions.push([x0, 0.0, z1]);
            positions.push([x1, 0.0, z1]);
            // Add texture vertices
            uvs.push([0.0, 0.0]);
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 1.0]);
        }
    }

    let mut mesh = Mesh {
        positions,
        uvs: Some(uvs),
        ..Default::default()
    };
    // Calculate mesh normals
    mesh.calculate();

    // Store mesh and get its ID
    let mesh = assets.store_as(mesh, "terrain");

    // import terrain texture and get its ID
    assets.import("examples/demo/terrain.png");
    let texture = assets.register("terrain");

    // Center terrain tile at coordinate system center (0.0, 0.0, 0.0) by moving the tile on a
    // half of its size by X and Z axis
    let shift = (size / 2) as f32;
    let transform = Transform {
        translate: Vec3::new(-shift, 0.0, -shift),
        ..Default::default()
    };

    // Spawn terrain in the world
    world.spawn(Some(
        (Model { mesh, texture, transform, ..Default::default() },)
    ));
}

fn init_player(
    world: &mut World,
    assets: &mut Assets,
    input: &mut Input,
) {
    // Import texture and store its ID in a variable
    assets.import("examples/demo/gray.png");
    let texture = assets.register("gray");

    // Import character model from GLTF file, it provides several assets: mesh, skin, and run
    // animation
    assets.import("examples/demo/character.gltf");
    let mesh = assets.register("character::Cube::mesh");
    let skin = assets.register("character::Cube::skin");
    let run = assets.register("character::run");

    // shrink player's character model
    let transform = Transform {
        scale: Vec3::new(0.5, 0.5, 0.5),
        translate: Vec3::new(0.0, 0.1, 0.0),
        ..Default::default()
    };

    // spawn model in the world
    world.spawn(Some(
        (
            Model { mesh, texture, skin, transform, ..Default::default() },
            Animator::new(run), // Animation control (stopped by default)
            Player {
                is_running: false,
            }
        ),
    ));

    // Map W key to Run Action
    input.mapper_mut::<Mapper<Action>>()
        .set(vec![
            (Action::Run, Button::Key(KeyCode::W)),
        ]);
}

fn init_light(world: &mut World) {
    // spawn source of white light at (0.0, 100.0, 0.0)
    world.spawn(Some((Light::white([0.0, 100.0, 0.0]),)));
}

 // Component indentifying players's entity
struct Player {
    is_running: bool,
}

fn player_control(
    world: Mut<World>,
    input: Const<Input>,
    frame: Const<Frame>,
    mut camera: Mut<Camera>,
) {
    const PLAYER_SPEED: f32 = 10.0;

    // Query player entity 
    let query = world.query::<(&mut Model, &mut Animator, &mut Player)>();
    // this loop will run only once, because Player component is assigned to only one entity
    for (model, animator, player) in query {
        // calculate distance offset if W is pressed and control animation
        let distance = if input.is_action_hold(Action::Run) {
            if !player.is_running {
                // start run animation
                animator.start_loop();
                player.is_running = true;
            }
            PLAYER_SPEED * frame.delta().as_secs_f32()
        } else {
            if player.is_running {
                // stop run animation
                animator.stop();
                player.is_running = false;
            }
            0.0
        };

        // get camera angle around Y axis
        let y_angle = camera.y_angle;
        // rotate model to the right direction
        model.transform.rotate = Quat::from_angle_y(Rad(-(PI / 2.0 + y_angle)));
        if distance > 0.00001 {
            // calculate X and Z deltas if player is moving
            let dx = distance * y_angle.cos();
            let dz = distance * y_angle.sin();
            // calculate new model positions
            let mut pos_x = model.transform.translate.x - dx;
            let mut pos_z = model.transform.translate.z - dz;
            // check terrain boundaries, so player won't run away from the terrain tile
            let half_terrain = TERRAIN_SIZE as f32 / 2.0;
            if pos_x < -half_terrain {
                pos_x = -half_terrain;
            } else if pos_x > half_terrain {
                pos_x = half_terrain;
            }
            if pos_z < -half_terrain {
                pos_z = -half_terrain;
            } else if pos_z > half_terrain {
                pos_z = half_terrain;
            }
            // apply translation
            model.transform.translate.x = pos_x;
            model.transform.translate.z = pos_z;

            // make camera following the player
            camera.target = Point3::new(pos_x, CAMERA_HEIGHT, pos_z);
            camera.set_view();
        }
    }
}

/// Enumeration of actions provided by the game
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Action {
    Run,
    // Jump,
}

/// Bind Inputs and Actions
impl ActionMapper<Action> for Input {
    fn action_mapped(&self, action: Action) -> Option<&Button> {
        let mapper = self.mapper::<Mapper<Action>>();
        mapper.get_button(action)
    }
}

