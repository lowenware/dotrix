use dotrix::{
    assets::{ Mesh },
    components::{ Light, Model },
    ecs::{ Mut, Const },
    egui::{
        Egui,
        CollapsingHeader,
        Grid,
        Label,
        TopPanel,
        Separator,
        Slider,
        Window
    },
    math::{ Vec3 },
    renderer::{ Transform },
    input::{ Button, State as InputState, Mapper, KeyCode },
    services::{ Assets, Camera, Frame, Input, World, Ray, Renderer },
    terrain::Terrain,
};

use crate::controls::Action;

use noise::{ Fbm, MultiFractal };
use std::f32::consts::PI;

pub struct Editor {
    pub sea_level: u8,
    pub terrain_size: usize,
    pub terrain_size_changed: bool,
    pub noise_octaves: usize,
    pub noise_frequency: f64,
    pub noise_lacunarity: f64,
    pub noise_persistence: f64,
    pub noise_scale: f64,
    pub noise_amplitude: f64,
    pub show_toolbox: bool,
    pub show_info: bool,
    pub brush_x: f32,
    pub brush_y: f32,
    pub brush_z: f32,
    pub brush_radius: f32,
    pub brush_add: bool,
    pub brush_sub: bool,
    pub brush_changed: bool,
    pub apply_noise: bool,
    pub lod: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            sea_level: 0,
            terrain_size: 64,
            terrain_size_changed: true,
            noise_octaves: 8,
            noise_frequency: 1.1,
            noise_lacunarity: 4.5,
            noise_persistence: 0.1,
            noise_scale: 256.0,
            noise_amplitude: 93.0,
            show_toolbox: true,
            show_info: false,
            brush_x: 0.0,
            brush_y: 10.0,
            brush_z: 0.0,
            brush_radius: 5.0,
            brush_add: false,
            brush_sub: false,
            brush_changed: false,
            apply_noise: true,
            lod: 2,
        }
    }

    pub fn noise(&self) -> Fbm {
        let noise = Fbm::new();
        let noise = noise.set_octaves(self.noise_octaves);
        let noise = noise.set_frequency(self.noise_frequency);
        let noise = noise.set_lacunarity(self.noise_lacunarity);
        noise.set_persistence(self.noise_persistence)
    }
}

pub fn ui(
    mut editor: Mut<Editor>,
    renderer: Mut<Renderer>,
    mut terrain: Mut<Terrain>,
    camera: Const<Camera>,
    frame: Const<Frame>,
    ray: Const<Ray>,
) {
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    TopPanel::top("side_panel").show(&egui.ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("🗋").clicked { println!("New"); }
            if ui.button("🖴").clicked { println!("Save"); }
            if ui.button("🗁").clicked { println!("Open"); }
            if ui.button("🛠").clicked { editor.show_toolbox = !editor.show_toolbox; }
            if ui.button("ℹ").clicked { editor.show_info = !editor.show_info; }
        });
    });

    let mut show_window = editor.show_toolbox;

    Window::new("Toolbox").open(&mut show_window).show(&egui.ctx, |ui| {

        CollapsingHeader::new("View").default_open(true).show(ui, |ui| {
            ui.add(Label::new("LOD"));
            ui.add(Slider::usize(&mut editor.lod, 1..=16).text("level"));
        });

        CollapsingHeader::new("Terrain")
            .default_open(true)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(Label::new("Size:"));
                        if ui.button("Resize").clicked { editor.terrain_size_changed = true; }
                    });
                    ui.add(Slider::usize(&mut editor.terrain_size, 8..=256).text("Meters"));

                    ui.add(Separator::new());

                    ui.horizontal(|ui| {
                        ui.add(Label::new("Brush:"));
                        if ui.button("Add").clicked { editor.brush_add = true; }
                        if ui.button("Sub").clicked { editor.brush_sub = true; }
                    });

                    ui.add(Slider::f32(&mut editor.brush_x, -64.0..=64.0).text("X"));
                    ui.add(Slider::f32(&mut editor.brush_y, -64.0..=64.0).text("Y"));
                    ui.add(Slider::f32(&mut editor.brush_z, -64.0..=64.0).text("Z"));
                    ui.add(Slider::f32(&mut editor.brush_radius, 1.0..=16.0).text("Radius"));

                    ui.add(Separator::new());

                    ui.add(Label::new("Noise:"));
                    ui.add(Slider::f64(&mut editor.noise_scale, 1.0..=256.0).text("Scale"));
                    ui.add(Slider::f64(&mut editor.noise_amplitude, 1.0..=256.0).text("Amplitude"));
                    ui.add(Slider::usize(&mut editor.noise_octaves, 1..=10).text("Octaves"));
                    ui.add(Slider::f64(&mut editor.noise_frequency, 0.1..=10.0).text("Frequency"));
                    ui.add(Slider::f64(&mut editor.noise_lacunarity, 0.1..=10.0).text("Lacunarity"));
                    ui.add(Slider::f64(&mut editor.noise_persistence, 0.1..=10.0).text("Persistence"));
                    if ui.button("Apply").clicked {
                        terrain.populate(
                            &editor.noise(),
                            editor.noise_amplitude,
                            editor.noise_scale
                        );
                    }
                });
            });
    });

    editor.show_toolbox = show_window;

    let mut show_window = editor.show_info;

    Window::new("Info").open(&mut show_window).show(&egui.ctx, |ui| {
        let grid = Grid::new("info")
            .striped(true)
            .spacing([40.0, 4.0]);

        grid.show(ui, |ui| {
            ui.label("FPS");
            ui.label(format!("{}", frame.fps()));
            ui.end_row();

            let vec = ray.origin.as_ref()
                .map(|v| format!("x: {:.4}, y: {:.4}, z: {:.4}", v.x, v.y, v.z));

            ui.label("Camera Position");
            ui.label(vec.as_ref().map(|s| s.as_str()).unwrap_or("-"));
            ui.end_row();

            let vec = format!("x: {:.4}, y: {:.4}, z: {:.4}",
                camera.target.x, camera.target.y, camera.target.z);

            ui.label("Camera Target");
            ui.label(vec);
            ui.end_row();

            let vec = ray.direction.as_ref()
                .map(|v| format!("x: {:.4}, y: {:.4}, z: {:.4}", v.x, v.y, v.z));

            ui.label("Mouse Ray");
            ui.label(vec.as_ref().map(|s| s.as_str()).unwrap_or("-"));
            ui.end_row();

            ui.label("Generated in");
            ui.label(format!("{} us", terrain.generated_in.as_micros()));
            ui.end_row();
        });
    });

    editor.show_info = show_window;
}

const ROTATE_SPEED: f32 = PI / 10.0;
const ZOOM_SPEED: f32 = 10.0;
const MOVE_SPEED: f32 = 64.0;

pub struct Cursor(Vec3);

pub fn startup(
    mut assets: Mut<Assets>,
    editor: Const<Editor>,
    mut input: Mut<Input>,
    mut renderer: Mut<Renderer>,
    mut terrain: Mut<Terrain>,
    mut world: Mut<World>,
) {

    assets.import("editor/assets/terrain.png");
    renderer.add_overlay(Box::new(Egui::default()));

    world.spawn(Some((Light::white([0.0, 500.0, 0.0]),)));

    input.mapper_mut::<Mapper<Action>>()
        .set(vec![
            (Action::Move, Button::Key(KeyCode::W)),
        ]);

    let cursor = assets.store(Mesh::cube());
    assets.import("assets/green.png");
    let texture = assets.register("green");
    let transform = Transform {
        translate: Vec3::new(0.0, 0.5, 0.0),
        scale: Vec3::new(0.05, 0.05, 0.05),
        ..Default::default()
    };

    world.spawn(
        Some((
            Model { mesh: cursor, texture, transform, ..Default::default() },
            Cursor(Vec3::new(0.0, 0.0, 0.0))
        ))
    );

    terrain.populate(&editor.noise(), editor.noise_amplitude, editor.noise_scale);
}

pub fn camera_control(
    mut camera: Mut<Camera>,
    input: Const<Input>,
    frame: Const<Frame>,
    world: Const<World>,
) {
    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    let distance = camera.distance - ZOOM_SPEED * mouse_scroll * time_delta;
    camera.distance = if distance > -1.0 { distance } else { -1.0 };

    if input.button_state(Button::MouseRight) == Some(InputState::Hold) {
        camera.y_angle += mouse_delta.x * ROTATE_SPEED * time_delta;

        let xz_angle = camera.xz_angle + mouse_delta.y * ROTATE_SPEED * time_delta;
        let half_pi = PI / 2.0;

        camera.xz_angle = if xz_angle >= half_pi {
            half_pi - 0.01
        } else if xz_angle <= -half_pi {
            -half_pi + 0.01
        } else {
            xz_angle
        };
    }

    // move
    let distance = if input.is_action_hold(Action::Move) {
        MOVE_SPEED * frame.delta().as_secs_f32()
    } else {
        0.0
    };

    if distance > 0.00001 {
        let y_angle = camera.y_angle;

        let dx = distance * y_angle.cos();
        let dz = distance * y_angle.sin();

        camera.target.x -= dx;
        camera.target.z -= dz;

        let query = world.query::<(&mut Model, &Cursor)>();
        for (model, _) in query {
            model.transform.translate.x = camera.target.x;
            model.transform.translate.z = camera.target.z;
        }
    }

    camera.set_view();
}
