use crate::Editable;

use dotrix::{ Color, Camera, Frame, Input, World };
use dotrix::ecs::{ Const, Mut };
use dotrix::egui::{
    CollapsingHeader,
    DragValue,
    Egui,
    Grid,
    ScrollArea,
    SidePanel,
    Slider,
};
use dotrix::input::{ Button, State as InputState };
use dotrix::math::Vec3;
use dotrix::overlay::Overlay;
use dotrix::pbr::Light;

use std::f32::consts::PI;

pub struct CarSettings {
    pub animate: bool,
    pub point_lights: bool,
    pub spot_lights: bool,
}

pub struct Settings {
    // Lights here are only structs, not real components.
    pub ambient_light_intensity: f32,
    pub directional_light_intensity: f32,
    pub simple_light_intensity: f32,
    pub point_light_intensity: f32,
    pub spot_light_intensity: f32,

    pub ambient_light_color: [f32; 3],
    pub directional_light_color: [f32; 3],
    pub point_light_color: [f32; 3],
    pub simple_light_color: [f32; 3],
    pub spot_light_color: [f32; 3],

    pub directional_light_enabled: bool,
    pub point_light_enabled: bool,
    pub simple_light_enabled: bool,
    pub spot_light_enabled: bool,

    pub directional_light_direction: Vec3,
    pub spot_light_direction: Vec3,

    pub point_light_position: Vec3,
    pub simple_light_position: Vec3,
    pub spot_light_position: Vec3,

    pub spot_light_cut_off: f32,
    pub spot_light_outer_cut_off: f32,

    pub point_light_constant: f32,
    pub point_light_linear: f32,
    pub point_light_quadratic: f32,

    pub car: CarSettings,
}

impl Settings {
    /// Reset to default values
    pub fn reset(&mut self) {
        *self = Settings::default();
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ambient_light_color: Color::rgb(0.04, 0.04, 0.08).into(),
            directional_light_color: Color::white().into(), 
            point_light_color: Color::green().into(),
            simple_light_color: Color::rgb(0.15, 0.08, 0.08).into(),
            spot_light_color: Color::yellow().into(),

            ambient_light_intensity: 0.8,
            directional_light_intensity: 0.5,
            simple_light_intensity: 0.35,
            point_light_intensity: 1.0,
            spot_light_intensity: 1.0,

            directional_light_enabled: true,
            point_light_enabled: true,
            simple_light_enabled: true,
            spot_light_enabled: true,

            directional_light_direction: Vec3::new(0.3, -0.5, -0.6),
            spot_light_direction: Vec3::new(-20.0, -20.0, 0.0),

            point_light_position: Vec3::new(6.0, 1.0, 0.0),
            simple_light_position: Vec3::new(-2.0, 2.0, 2.0),
            spot_light_position: Vec3::new(12.0, 2.5, -10.0),

            spot_light_cut_off: 0.8,
            spot_light_outer_cut_off: 0.65,

            point_light_constant: 1.0,
            point_light_linear: 0.35,
            point_light_quadratic: 0.44,

            car: CarSettings {
                animate: true,
                point_lights: true,
                spot_lights: true,
            }
        }
    }
}

pub fn ui(mut settings: Mut<Settings>, overlay: Mut<Overlay>) {
    let egui = overlay.get::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    SidePanel::left("side_panel").show(&egui.ctx, |ui| {
        ScrollArea::auto_sized().show(ui, |ui| {
            CollapsingHeader::new("Car Settings")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("car settings").show(ui, |ui| {
                    ui.label("Animate");
                    ui.checkbox(&mut settings.car.animate, "");
                    ui.end_row();

                    ui.label("Point lights");
                    ui.checkbox(&mut settings.car.point_lights, "");
                    ui.end_row();

                    ui.label("Spot lights");
                    ui.checkbox(&mut settings.car.spot_lights, "");
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Ambient Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("ambient light").show(ui, |ui| {
                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.ambient_light_color);
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.ambient_light_color = Settings::default().ambient_light_color;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::new(&mut settings.ambient_light_intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.ambient_light_intensity = Settings::default().ambient_light_intensity;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Directional Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("dir light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.directional_light_enabled, "");
                    ui.end_row();

                    ui.label("Direction");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::new(&mut settings.directional_light_direction.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::new(&mut settings.directional_light_direction.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::new(&mut settings.directional_light_direction.z).prefix("z: ").speed(0.01));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.directional_light_color);
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.directional_light_color = Settings::default().directional_light_color;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::new(&mut settings.directional_light_intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.directional_light_intensity = Settings::default().directional_light_intensity;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Point Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("point light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.point_light_enabled, "");
                    ui.end_row();

                    ui.label("Position");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::new(&mut settings.point_light_position.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::new(&mut settings.point_light_position.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::new(&mut settings.point_light_position.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.point_light_color);
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.point_light_color = Settings::default().point_light_color;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::new(&mut settings.point_light_intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.point_light_intensity = Settings::default().point_light_intensity;
                    };
                    ui.end_row();

                    ui.label("Constant Attenuation");
                    ui.add(Slider::new(&mut settings.point_light_constant, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.point_light_constant = Settings::default().point_light_constant;
                    };
                    ui.end_row();

                    ui.label("Linear Attenuation");
                    ui.add(Slider::new(&mut settings.point_light_linear, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.point_light_linear = Settings::default().point_light_linear;
                    };
                    ui.end_row();

                    ui.label("Quadratic Attenuation");
                    ui.add(Slider::new(&mut settings.point_light_quadratic, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.point_light_quadratic = Settings::default().point_light_quadratic;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Simple Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("simple light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.simple_light_enabled, "");
                    ui.end_row();

                    ui.label("Position");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::new(&mut settings.simple_light_position.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::new(&mut settings.simple_light_position.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::new(&mut settings.simple_light_position.z).prefix("z: ").speed(0.01));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.simple_light_color);
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.simple_light_color = Settings::default().simple_light_color;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::new(&mut settings.simple_light_intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.simple_light_intensity = Settings::default().simple_light_intensity;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Spot Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("spot light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.spot_light_enabled, "");
                    ui.end_row();

                    ui.label("Position");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::new(&mut settings.spot_light_position.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::new(&mut settings.spot_light_position.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::new(&mut settings.spot_light_position.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Direction");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::new(&mut settings.spot_light_direction.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::new(&mut settings.spot_light_direction.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::new(&mut settings.spot_light_direction.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.spot_light_color);
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.spot_light_color = Settings::default().spot_light_color;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::new(&mut settings.spot_light_intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.spot_light_intensity = Settings::default().spot_light_intensity;
                    };
                    ui.end_row();

                    ui.label("Cut-off");
                    ui.add(Slider::new(&mut settings.spot_light_cut_off, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.spot_light_cut_off = Settings::default().spot_light_cut_off;
                    };
                    ui.end_row();

                    ui.label("Outer cut-off");
                    ui.add(Slider::new(&mut settings.spot_light_outer_cut_off, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.spot_light_outer_cut_off = Settings::default().spot_light_outer_cut_off;
                    };
                    ui.end_row();
                });
            });

            if ui.button("Reset all").clicked() {
                settings.reset();
            };
        });
    });
}

/// This func updates camera based on values in editor and controls
pub fn update_camera(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
    const ROTATE_SPEED: f32 = PI / 10.0;
    const ZOOM_SPEED: f32 = 10.0;

    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    // Get values from camera
    let mut distance = camera.distance;
    let target = camera.target;
    let mut y_angle = camera.y_angle;
    let mut xz_angle = camera.xz_angle;

    // Calculate new values
    distance -= ZOOM_SPEED * mouse_scroll * time_delta;
    distance = if distance > -1.0 { distance } else { -1.0 };

    if input.button_state(Button::MouseRight) == Some(InputState::Hold) {
        y_angle += mouse_delta.x * ROTATE_SPEED * time_delta;
        xz_angle = camera.xz_angle + mouse_delta.y * ROTATE_SPEED * time_delta;
        let half_pi = PI / 2.0;

        xz_angle = if xz_angle >= half_pi {
            half_pi - 0.01
        } else if xz_angle <= -half_pi {
            -half_pi + 0.01
        } else {
            xz_angle
        };
    }

    // Apply values to camera
    camera.target = target;
    camera.distance = distance;
    camera.y_angle = y_angle;
    camera.xz_angle = xz_angle;
}

/// This func updates all editable light entities based on values in settings, and car settings.
pub fn update_settings(
    settings: Const<Settings>,
    world: Mut<World>,
) {
    // Set ambient light, should be only one
    let query = world.query::<(&mut Light, &Editable)>();
    for (light, _) in query {
        match light {
            Light::Ambient { color, intensity, .. } => {
                *intensity = settings.ambient_light_intensity;
                *color = Color::from(settings.ambient_light_color);
            },
            Light::Directional { color, intensity, direction, enabled, .. } => {
                *color = settings.directional_light_color.into();
                *intensity = settings.directional_light_intensity;
                *direction = settings.directional_light_direction;
                *enabled = settings.directional_light_enabled;
            },
            Light::Point {
                color,
                intensity,
                position,
                enabled,
                constant,
                linear,
                quadratic,
                ..
            } => {
                *color = settings.point_light_color.into();
                *intensity = settings.point_light_intensity;
                *position = settings.point_light_position;
                *constant = settings.point_light_constant;
                *linear = settings.point_light_linear;
                *quadratic = settings.point_light_quadratic;
                *enabled = settings.point_light_enabled;
            },
            Light::Simple { color, intensity, position, enabled, .. } => {
                *color = settings.simple_light_color.into();
                *intensity = settings.simple_light_intensity;
                *position = settings.simple_light_position;
                *enabled = settings.simple_light_enabled;
            },
            Light::Spot {
                color, intensity, position, direction, enabled, cut_off, outer_cut_off, ..
            } => {
                *color = settings.spot_light_color.into();
                *intensity = settings.spot_light_intensity;
                *position = settings.spot_light_position;
                *direction = settings.spot_light_direction;
                *cut_off = settings.spot_light_cut_off;
                *outer_cut_off = settings.spot_light_outer_cut_off;
                *enabled = settings.spot_light_enabled;
            },
        }
    }

    /*
    // Query directional lights entities
    let query = world.query::<(&mut DirLight, &Editable)>();
    for (directional_light, _) in query {
        directional_light.clone_from(&settings.directional_light);
    }

    // Query point light entities
    let query = world.query::<(&mut PointLight, &Editable)>();
    for (point_light, _) in query {
        point_light.clone_from(&settings.point_light);
        point_light.color = Color::from( settings.point_light_color);
    }

    // Query simple light entities
    let query = world.query::<(&mut SimpleLight, &Editable)>();
    for (simple_light, _) in query {
        simple_light.clone_from(&settings.simple_light);
        simple_light.color = Color::from( settings.simple_light_color);
    }

    // Query spot light entities
    let query = world.query::<(&mut SpotLight, &Editable)>();
    for (spot_light, _) in query {
        spot_light.clone_from(&settings.spot_light);
        spot_light.color = Color::from( settings.spot_light_color);
    }
    */
}
