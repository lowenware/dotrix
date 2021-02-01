use crate::car::CarSettings;
use crate::Editable;

use dotrix::{
    components::{ AmbientLight, DirLight, PointLight, SimpleLight, SpotLight },
    ecs::{ Const, Mut },
    egui::{
        CollapsingHeader,
        DragValue,
        Egui,
        Grid,
        ScrollArea,
        SidePanel,
        Slider,
    },
    input::{ Button, State as InputState },
    math::{ Vec3 },
    renderer::{ Color },
    services::{ Camera, Frame, Input, Renderer, World },
};
use std::f32::consts::PI;


pub struct Settings {
    // Lights here are only structs, not real components.
    pub amb_light: AmbientLight,
    pub dir_light: DirLight,
    pub point_light: PointLight,
    pub simple_light: SimpleLight,
    pub spot_light: SpotLight,

    pub amb_light_clr_pick: [f32; 3],
    pub dir_light_clr_pick: [f32; 3],
    pub point_light_clr_pick: [f32; 3],
    pub simple_light_clr_pick: [f32; 3],
    pub spot_light_clr_pick: [f32; 3],

    pub car: CarSettings,
}

impl Settings {
    /// Reset to default values
    pub fn reset(&mut self) {
        let default = Settings::default();

        self.amb_light = default.amb_light;
        self.dir_light = default.dir_light;
        self.point_light = default.point_light;
        self.simple_light = default.simple_light;
        self.spot_light = default.spot_light;

        self.amb_light_clr_pick = default.amb_light_clr_pick;
        self.dir_light_clr_pick = default.dir_light_clr_pick;
        self.point_light_clr_pick = default.point_light_clr_pick;
        self.simple_light_clr_pick = default.simple_light_clr_pick;
        self.spot_light_clr_pick = default.spot_light_clr_pick;

        self.car = Settings::default().car;
    }
}

impl Default for Settings {
    fn default() -> Self {
        let amb_light = AmbientLight {
            color: Color::rgb(0.04, 0.04, 0.08),
            intensity: 0.0,
        };
        let dir_light = DirLight {
            enabled: false,
            direction: Vec3::new(0.3, -0.5, -0.6),
            color: Color::white(),
            intensity: 0.5,
        };
        let point_light = PointLight {
            position: Vec3::new(6.0, 1.0, 0.0),
            color: Color::green(),
            ..Default::default()
        };
        let simple_light = SimpleLight {
            position: Vec3::new(-2.0, 2.0, 2.0),
            color: Color::rgb(0.15, 0.08, 0.08),
            intensity: 0.35,
            ..Default::default()
        };
        let spot_light = SpotLight {
            color: Color::yellow(),
            position: Vec3::new(12.0, 2.5, -10.0),
            direction: Vec3::new(-20.0, -20.0, 0.0),
            ..Default::default()
        };

        Self {
            amb_light_clr_pick: amb_light.color.to_f32_3(),
            dir_light_clr_pick: dir_light.color.to_f32_3(),
            point_light_clr_pick: point_light.color.to_f32_3(),
            simple_light_clr_pick: simple_light.color.to_f32_3(),
            spot_light_clr_pick: spot_light.color.to_f32_3(),

            amb_light,
            dir_light,
            point_light,
            simple_light,
            spot_light,

            car: CarSettings {
                animate: true,
                point_lights: true,
                spot_lights: true,
            }
        }
    }
}

pub fn ui(mut settings: Mut<Settings>, renderer: Mut<Renderer>) {
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    SidePanel::left("side_panel", 300.0).show(&egui.ctx, |ui| {
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
                    ui.color_edit_button_rgb(&mut settings.amb_light_clr_pick);
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.amb_light_clr_pick = Settings::default().amb_light_clr_pick;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::f32(&mut settings.amb_light.intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.amb_light.intensity = Settings::default().amb_light.intensity;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Directional Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("dir light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.dir_light.enabled, "");
                    ui.end_row();

                    ui.label("Direction");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.dir_light.direction.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.dir_light.direction.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.dir_light.direction.z).prefix("z: ").speed(0.01));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.dir_light_clr_pick);
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.dir_light_clr_pick = Settings::default().dir_light_clr_pick;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::f32(&mut settings.dir_light.intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.dir_light.intensity = Settings::default().dir_light.intensity;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Point Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("point light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.point_light.enabled, "");
                    ui.end_row();

                    ui.label("Position");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.point_light.position.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.point_light.position.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.point_light.position.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.point_light_clr_pick);
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.point_light_clr_pick = Settings::default().point_light_clr_pick;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::f32(&mut settings.point_light.intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.point_light.intensity = Settings::default().point_light.intensity;
                    };
                    ui.end_row();

                    ui.label("Constant Attenuation");
                    ui.add(Slider::f32(&mut settings.point_light.constant, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.point_light.constant = Settings::default().point_light.constant;
                    };
                    ui.end_row();

                    ui.label("Linear Attenuation");
                    ui.add(Slider::f32(&mut settings.point_light.linear, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.point_light.linear = Settings::default().point_light.linear;
                    };
                    ui.end_row();

                    ui.label("Quadratic Attenuation");
                    ui.add(Slider::f32(&mut settings.point_light.quadratic, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.point_light.quadratic = Settings::default().point_light.quadratic;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Simple Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("simple light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.simple_light.enabled, "");
                    ui.end_row();

                    ui.label("Position");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.simple_light.position.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.simple_light.position.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.simple_light.position.z).prefix("z: ").speed(0.01));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.simple_light_clr_pick);
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.simple_light_clr_pick = Settings::default().simple_light_clr_pick;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::f32(&mut settings.simple_light.intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.simple_light.intensity = Settings::default().simple_light.intensity;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Spot Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("spot light").show(ui, |ui| {
                    ui.label("Enabled");
                    ui.checkbox(&mut settings.spot_light.enabled, "");
                    ui.end_row();

                    ui.label("Position");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.spot_light.position.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.spot_light.position.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.spot_light.position.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Direction");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.spot_light.direction.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.spot_light.direction.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.spot_light.direction.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Color");
                    ui.color_edit_button_rgb(&mut settings.spot_light_clr_pick);
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.spot_light_clr_pick = Settings::default().spot_light_clr_pick;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::f32(&mut settings.spot_light.intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.spot_light.intensity = Settings::default().spot_light.intensity;
                    };
                    ui.end_row();

                    ui.label("Cut-off");
                    ui.add(Slider::f32(&mut settings.spot_light.cut_off, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.spot_light.cut_off = Settings::default().spot_light.cut_off;
                    };
                    ui.end_row();

                    ui.label("Outer cut-off");
                    ui.add(Slider::f32(&mut settings.spot_light.outer_cut_off, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.spot_light.outer_cut_off = Settings::default().spot_light.outer_cut_off;
                    };
                    ui.end_row();
                });
            });

            if ui.button("Reset all").clicked {
                settings.reset();
            };
        });
    });
}

pub fn init(mut renderer: Mut<Renderer>) {
    renderer.add_overlay(Box::new(Egui::default()));
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
    camera.set_view();
}

/// This func updates all editable light entities based on values in settings, and car settings.
pub fn update_settings(
    settings: Const<Settings>,
    world: Mut<World>,
) {
    // Set ambient light, should be only one
    let query = world.query::<(&mut AmbientLight, &Editable)>();
    for (amb_light, _) in query {
        amb_light.clone_from(&settings.amb_light);
        amb_light.color = Color::from(settings.amb_light_clr_pick);
    }

    // Query directional lights entities
    let query = world.query::<(&mut DirLight, &Editable)>();
    for (dir_light, _) in query {
        dir_light.clone_from(&settings.dir_light);
    }

    // Query point light entities
    let query = world.query::<(&mut PointLight, &Editable)>();
    for (point_light, _) in query {
        point_light.clone_from(&settings.point_light);
        point_light.color = Color::from_f32_3( settings.point_light_clr_pick);
    }

    // Query simple light entities
    let query = world.query::<(&mut SimpleLight, &Editable)>();
    for (simple_light, _) in query {
        simple_light.clone_from(&settings.simple_light);
        simple_light.color = Color::from_f32_3( settings.simple_light_clr_pick);
    }

    // Query spot light entities
    let query = world.query::<(&mut SpotLight, &Editable)>();
    for (spot_light, _) in query {
        spot_light.clone_from(&settings.spot_light);
        spot_light.color = Color::from_f32_3( settings.spot_light_clr_pick);
    }

    // Query car settings
    let query = world.query::<(&mut CarSettings,)>();
    for (car_settings,) in query {
        car_settings.clone_from(&settings.car);
    }
}