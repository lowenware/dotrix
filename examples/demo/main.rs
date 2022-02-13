use dotrix::assets::Mesh;
use dotrix::camera;
use dotrix::egui::{self, Egui};
use dotrix::input::{ActionMapper, Button, KeyCode, Mapper, Modifiers};
use dotrix::math::{Point3, Quat, Rad, Rotation3, Vec3};
use dotrix::overlay::{self, Overlay};
use dotrix::pbr::{self, Light, Material, Model};
use dotrix::prelude::*;
use dotrix::renderer::Render;
use dotrix::sky::{skybox, SkyBox};
use dotrix::{Animator, Assets, Camera, Color, CubeMap, Frame, Input, Pose, Transform, World};

use std::f32::consts::PI;

const CAMERA_HEIGHT: f32 = 2.0; // Default camera target feight
const TERRAIN_SIZE: usize = 128; // Number of sqaures per side

fn main() {
    Dotrix::application("Dotrix: Demo Example")
        .with(System::from(startup))
        .with(System::from(player_control))
        .with(System::from(camera::control))
        .with(System::from(ui))
        .with(overlay::extension)
        .with(egui::extension)
        .with(pbr::extension)
        .with(skybox::extension)
        .run();
}

/// Initial game routines
fn startup(
    mut assets: Mut<Assets>,
    mut camera: Mut<Camera>,
    mut input: Mut<Input>,
    mut world: Mut<World>,
) {
    input.set_mapper(Box::new(Mapper::<Action>::new()));
    init_camera(&mut camera);
    init_skybox(&mut world, &mut assets);
    init_terrain(&mut world, &mut assets);
    init_light(&mut world);
    init_player(&mut world, &mut assets, &mut input);
}

fn init_camera(camera: &mut Camera) {
    camera.pan = PI / 2.0;
    camera.tilt = 0.0;
    camera.target = Point3::new(0.0, CAMERA_HEIGHT, 0.0);
    camera.distance = 5.0;
}

fn init_skybox(world: &mut World, assets: &mut Assets) {
    // Import skybox textures
    assets.import("assets/skybox-compass/skybox_right.png");
    assets.import("assets/skybox-compass/skybox_left.png");
    assets.import("assets/skybox-compass/skybox_top.png");
    assets.import("assets/skybox-compass/skybox_bottom.png");
    assets.import("assets/skybox-compass/skybox_back.png");
    assets.import("assets/skybox-compass/skybox_front.png");

    // Spawn skybox
    world.spawn(Some((
        SkyBox {
            view_range: 500.0,
            ..Default::default()
        },
        CubeMap {
            right: assets.register("skybox_right"),
            left: assets.register("skybox_left"),
            top: assets.register("skybox_top"),
            bottom: assets.register("skybox_bottom"),
            back: assets.register("skybox_back"),
            front: assets.register("skybox_front"),
            ..Default::default()
        },
        Render::default(),
    )));
}

fn init_terrain(world: &mut World, assets: &mut Assets) {
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

    let normals = Mesh::calculate_normals(&positions, None);
    let (tangents, bitangents) = Mesh::calculate_tangents_bitangents(&positions, &uvs, None);

    let mut mesh = Mesh::default();

    mesh.with_vertices(&positions);
    mesh.with_vertices(&normals);
    mesh.with_vertices(&tangents);
    mesh.with_vertices(&bitangents);
    mesh.with_vertices(&uvs);

    // Store mesh and get its ID
    let mesh = assets.store_as(mesh, "terrain");

    // import terrain texture and get its ID
    assets.import("assets/textures/terrain.png");
    let texture = assets.register("terrain");

    // Center terrain tile at coordinate system center (0.0, 0.0, 0.0) by moving the tile on a
    // half of its size by X and Z axis
    let shift = (size / 2) as f32;

    world.spawn(
        (pbr::solid::Entity {
            mesh,
            texture,
            translate: Vec3::new(-shift, 0.0, -shift),
            ..Default::default()
        })
        .some(),
    );
}

fn init_player(world: &mut World, assets: &mut Assets, input: &mut Input) {
    // Import character model from GLTF file, it provides several assets: mesh, skin, and run
    // animation
    assets.import("assets/models/character.gltf");
    let mesh = assets.register("character::Cube::mesh");
    let skin = assets.register("character::Cube::skin");
    let run = assets.register("character::run");

    // spawn model in the world
    world.spawn(Some((
        Model::from(mesh),
        Pose::from(skin),
        Material {
            albedo: Color::rgb(0.2, 0.2, 0.2),
            ao: 0.2,
            ..Default::default()
        },
        Transform {
            scale: Vec3::new(0.5, 0.5, 0.5),
            translate: Vec3::new(0.0, 0.1, 0.0),
            ..Default::default()
        },
        Animator::new(run), // Animation control (stopped by default)
        Render::default(),
        Player { is_running: false },
    )));

    // Map W key to Run Action
    input.mapper_mut::<Mapper<Action>>().set(&[(
        Action::Run,
        Button::Key(KeyCode::W),
        Modifiers::empty(),
    )]);
}

fn init_light(world: &mut World) {
    // spawn source of white light at (0.0, 100.0, 0.0)
    world.spawn(Some((Light::Simple {
        // direction: Vec3::new(0.3, -0.5, -0.6),
        position: Vec3::new(0.0, 1000.0, 0.0),
        color: Color::white(),
        intensity: 0.5,
        enabled: true,
    },)));
    // spawn source of white light at (0.0, 100.0, 0.0)
    world.spawn(Some((Light::Ambient {
        color: Color::white(),
        intensity: 0.2,
    },)));
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
    let query = world.query::<(&mut Transform, &mut Animator, &mut Player)>();
    // this loop will run only once, because Player component is assigned to only one entity
    for (transform, animator, player) in query {
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
        let pan = camera.pan;
        // rotate model to the right direction
        transform.rotate = Quat::from_angle_y(Rad(-(PI / 2.0 + pan)));
        if distance > 0.00001 {
            // calculate X and Z deltas if player is moving
            let dx = distance * pan.cos();
            let dz = distance * pan.sin();
            // calculate new model positions
            let mut pos_x = transform.translate.x - dx;
            let mut pos_z = transform.translate.z - dz;
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
            transform.translate.x = pos_x;
            transform.translate.z = pos_z;

            // make camera following the player
            camera.target = Point3::new(pos_x, CAMERA_HEIGHT, pos_z);
        }
    }
}

/// Enumeration of actions provided by the game
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Action {
    Run,
    // Jump,
}

fn ui(overlay: Const<Overlay>, frame: Const<Frame>) {
    let egui_overlay = overlay
        .get::<Egui>()
        .expect("Egui overlay must be added on startup");

    egui::Area::new("FPS counter")
        .fixed_pos(egui::pos2(16.0, 16.0))
        .show(&egui_overlay.ctx, |ui| {
            ui.colored_label(
                egui::Rgba::from_rgb(255.0, 255.0, 255.0),
                format!("FPS: {:.1}", frame.fps()),
            );
        });
}

/// Bind Inputs and Actions
impl ActionMapper<Action> for Input {
    fn action_mapped(&self, action: Action) -> Option<(Button, Modifiers)> {
        let mapper = self.mapper::<Mapper<Action>>();
        mapper.get_button(action)
    }
}
