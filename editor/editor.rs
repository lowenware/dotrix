use dotrix::{
    components::{ Light },
    ecs::{ Mut, Const },
    egui::{
        Egui,
        CollapsingHeader,
        SidePanel,
        Slider
    },
    input::{ Button, State as InputState },
    services::{ Camera, Frame, Input, World, Renderer },
};

use std::f32::consts::PI;

pub struct Editor {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
    pub chunk_size: usize,
    pub xz_div: f64,
    pub y_div: f64,
    pub changed: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            octaves: 6,
            frequency: 1.0,
            lacunarity: 2.0,
            persistence: 0.5,
            chunk_size: 64,
            xz_div: 4.0,
            y_div: 8.0,
            changed: true,
        }
    }
}

pub fn ui(mut editor: Mut<Editor>, renderer: Mut<Renderer>) {
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    SidePanel::left("side_panel", 240.0).show(&egui.ctx, |ui| {
        CollapsingHeader::new("Terrain")
            .default_open(true)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add(Slider::usize(&mut editor.chunk_size, 8..=256).text("Size"));
                    ui.add(Slider::f64(&mut editor.xz_div, 1.0..=256.0).text("XZ Divider"));
                    ui.add(Slider::f64(&mut editor.y_div, 1.0..=256.0).text("Y Divider"));
                    ui.add(Slider::usize(&mut editor.octaves, 1..=10).text("Octaves"));
                    ui.add(Slider::f64(&mut editor.frequency, 0.1..=10.0).text("Frequency"));
                    ui.add(Slider::f64(&mut editor.lacunarity, 0.1..=10.0).text("Lacunarity"));
                    ui.add(Slider::f64(&mut editor.persistence, 0.1..=10.0).text("Persistence"));
                });
                if ui.button("Update").clicked {
                    editor.changed = true;
                }
            });
                // ui.label(format!("Hello '{}', age {}", name, age));
    });
}

const ROTATE_SPEED: f32 = PI / 10.0;
const ZOOM_SPEED: f32 = 10.0;

pub fn startup(mut renderer: Mut<Renderer>, mut world: Mut<World>) {
    renderer.add_overlay(Box::new(Egui::default()));

    world.spawn(Some((Light::white([0.0, 500.0, 0.0]),)));
}

pub fn camera_control(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
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

    camera.set_view();
}
