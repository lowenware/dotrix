use crate::fox::{Fox, FoxAnimClip};
use dotrix::animation::State as AnimState;
use dotrix::ecs::{Const, Mut};
use dotrix::egui::{CollapsingHeader, ComboBox, DragValue, Egui, Grid, SidePanel, Slider};
use dotrix::input::{Button, State as InputState};
use dotrix::math::{Point3, Vec3};
use dotrix::overlay::Overlay;
use dotrix::pbr::Light;
use dotrix::{Animator, Camera, Color, Frame, Input, Transform, World};

use std::f32::consts::PI;

pub struct Settings {
    pub fox_transform: Transform,

    pub anim_clip: FoxAnimClip,
    pub anim_play: bool,
    pub anim_speed: f32,

    pub cam_distance: f32,
    pub cam_pan: f32,
    pub cam_tilt: f32,
    pub cam_target: Point3,

    // Lights here are only structs, not real components.
    pub ambient_light_color: Color,
    pub ambient_light_intensity: f32,
    pub simple_light_color: Color,
    pub simple_light_intensity: f32,
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
            fox_transform: Transform {
                translate: Vec3 {
                    x: 80.0,
                    y: 0.0,
                    z: 0.0,
                },
                ..Default::default()
            },

            anim_clip: FoxAnimClip::Walk,
            anim_play: true,
            anim_speed: 1.0,

            cam_distance: 222.0,
            cam_pan: 0.74,
            cam_tilt: 0.25,
            cam_target: Point3::new(0.0, 0.5, 0.0),

            simple_light_color: Color::white(),
            simple_light_intensity: 0.8,
            ambient_light_color: Color::white(),
            ambient_light_intensity: 0.8,
        }
    }
}

pub fn ui(mut settings: Mut<Settings>, overlay: Const<Overlay>) {
    let egui = overlay
        .get::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    SidePanel::left("side_panel").show(&egui.ctx, |ui| {
        CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("transform").show(ui, |ui| {
                    ui.label("Translation");
                    ui.horizontal(|ui| {
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.translate.x)
                                .prefix("x: ")
                                .speed(0.1),
                        );
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.translate.y)
                                .prefix("y: ")
                                .speed(0.1),
                        );
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.translate.z)
                                .prefix("z: ")
                                .speed(0.1),
                        );
                    });
                    ui.end_row();

                    ui.label("Rotation");
                    ui.horizontal(|ui| {
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.rotate.v.x)
                                .prefix("x: ")
                                .speed(0.01),
                        );
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.rotate.v.y)
                                .prefix("y: ")
                                .speed(0.01),
                        );
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.rotate.v.z)
                                .prefix("z: ")
                                .speed(0.01),
                        );
                    });
                    ui.end_row();

                    ui.label("Scale");
                    ui.horizontal(|ui| {
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.scale.x)
                                .prefix("x: ")
                                .speed(0.01),
                        );
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.scale.y)
                                .prefix("y: ")
                                .speed(0.01),
                        );
                        ui.add(
                            DragValue::new(&mut settings.fox_transform.scale.z)
                                .prefix("z: ")
                                .speed(0.01),
                        );
                    });
                    ui.end_row();
                });
            });

        CollapsingHeader::new("Animation")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("animation").show(ui, |ui| {
                    ui.label("Clip");
                    ComboBox::from_id_source("Clip")
                        .selected_text(format!("{:?}", settings.anim_clip))
                        .show_ui(ui, |ui| {
                            // There should be some EnumIterator in real implementation
                            ui.selectable_value(
                                &mut settings.anim_clip,
                                FoxAnimClip::Walk,
                                format!("{:?}", FoxAnimClip::Walk),
                            );
                            ui.selectable_value(
                                &mut settings.anim_clip,
                                FoxAnimClip::Run,
                                format!("{:?}", FoxAnimClip::Run),
                            );
                            ui.selectable_value(
                                &mut settings.anim_clip,
                                FoxAnimClip::Survey,
                                format!("{:?}", FoxAnimClip::Survey),
                            );
                        });

                    if ui
                        .button(if settings.anim_play { "Stop" } else { "Play" })
                        .clicked()
                    {
                        settings.anim_play = !settings.anim_play;
                    };
                    ui.end_row();

                    ui.label("Speed");
                    ui.add(Slider::new(&mut settings.anim_speed, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.anim_speed = Settings::default().anim_speed;
                    };
                });
            });

        CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("Target").show(ui, |ui| {
                    ui.label("Target");
                    ui.horizontal(|ui| {
                        ui.add(
                            DragValue::new(&mut settings.cam_target.x)
                                .prefix("x: ")
                                .speed(0.1),
                        );
                        ui.add(
                            DragValue::new(&mut settings.cam_target.y)
                                .prefix("y: ")
                                .speed(0.1),
                        );
                        ui.add(
                            DragValue::new(&mut settings.cam_target.z)
                                .prefix("z: ")
                                .speed(0.1),
                        );
                        if ui.button("¤").on_hover_text("Target fox").clicked() {
                            settings.cam_target.x = settings.fox_transform.translate.x;
                            settings.cam_target.y = settings.fox_transform.translate.y;
                            settings.cam_target.z = settings.fox_transform.translate.z;
                        };
                    });
                    ui.end_row();

                    ui.label("Distance");
                    ui.add(DragValue::new(&mut settings.cam_distance).speed(0.1));
                    ui.end_row();

                    ui.label("Y Angle");
                    ui.add(DragValue::new(&mut settings.cam_pan).speed(0.01));
                    ui.end_row();

                    ui.label("YZ Angle");
                    ui.add(DragValue::new(&mut settings.cam_tilt).speed(0.01));
                    ui.end_row();
                });
            });

        CollapsingHeader::new("Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("light").show(ui, |ui| {
                    ui.label("Red");
                    ui.add(Slider::new(&mut settings.simple_light_color.r, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.simple_light_color.r = Settings::default().simple_light_color.r;
                    };
                    ui.end_row();

                    ui.label("Green");
                    ui.add(Slider::new(&mut settings.simple_light_color.g, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.simple_light_color.g = Settings::default().simple_light_color.g;
                    };
                    ui.end_row();

                    ui.label("Blue");
                    ui.add(Slider::new(&mut settings.simple_light_color.b, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.simple_light_color.b = Settings::default().simple_light_color.b;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::new(&mut settings.simple_light_intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.simple_light_intensity =
                            Settings::default().simple_light_intensity;
                    };
                    ui.end_row();
                });
            });

        CollapsingHeader::new("Ambient Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("ambient light").show(ui, |ui| {
                    ui.label("Red");
                    ui.add(Slider::new(&mut settings.ambient_light_color.r, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.ambient_light_color.r = Settings::default().ambient_light_color.r;
                    };
                    ui.end_row();

                    ui.label("Green");
                    ui.add(Slider::new(&mut settings.ambient_light_color.g, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.ambient_light_color.g = Settings::default().ambient_light_color.g;
                    };
                    ui.end_row();

                    ui.label("Blue");
                    ui.add(Slider::new(&mut settings.ambient_light_color.b, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.ambient_light_color.b = Settings::default().ambient_light_color.b;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::new(&mut settings.ambient_light_intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked() {
                        settings.ambient_light_intensity =
                            Settings::default().ambient_light_intensity;
                    };
                    ui.end_row();
                });
            });

        if ui.button("Reset all").clicked() {
            settings.reset();
        };
    });
}

/// This func updates camera based on values in settings and controls
pub fn update_camera(
    mut camera: Mut<Camera>,
    mut settings: Mut<Settings>,
    input: Const<Input>,
    frame: Const<Frame>,
) {
    const ROTATE_SPEED: f32 = PI / 10.0;
    const ZOOM_SPEED: f32 = 500.0;

    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    // Get values from settings
    let mut distance = settings.cam_distance;
    let target = settings.cam_target;
    let mut pan = settings.cam_pan;
    let mut tilt = settings.cam_tilt;

    // Calculate new values
    distance -= ZOOM_SPEED * mouse_scroll * time_delta;
    distance = if distance > -1.0 { distance } else { -1.0 };

    if input.button_state(Button::MouseRight) == Some(InputState::Hold) {
        pan += mouse_delta.x * ROTATE_SPEED * time_delta;
        tilt = camera.tilt + mouse_delta.y * ROTATE_SPEED * time_delta;
        let half_pi = PI / 2.0;

        tilt = if tilt >= half_pi {
            half_pi - 0.01
        } else if tilt <= -half_pi {
            -half_pi + 0.01
        } else {
            tilt
        };
    }

    // Apply values to settings
    settings.cam_distance = distance;
    settings.cam_pan = pan;
    settings.cam_tilt = tilt;

    // Apply values to camera
    camera.target = target;
    camera.distance = distance;
    camera.pan = pan;
    camera.tilt = tilt;
}

/// This func updates fox's entity based on values in settings
pub fn update_fox(settings: Const<Settings>, world: Mut<World>) {
    // Query fox entity
    let query = world.query::<(&mut Transform, &mut Animator, &mut Fox)>();

    // This loop will run only once, because Fox component is assigned to only one entity
    for (transform, animator, fox) in query {
        // Set transformation
        transform.translate = settings.fox_transform.translate;
        transform.rotate = settings.fox_transform.rotate;
        transform.scale = settings.fox_transform.scale;

        // Set animation state
        match animator.state() {
            AnimState::Loop(_) => {
                if !settings.anim_play {
                    animator.stop();
                }
            }
            AnimState::Stop => {
                if settings.anim_play {
                    animator.start_loop();
                }
            }
            _ => {}
        };

        // Set animation clip
        let clip = fox.animations[&settings.anim_clip];
        if animator.animation() != clip {
            animator.animate(clip);
        }

        // Set animation speed
        animator.speed = settings.anim_speed;
    }
}

/// This func updates all light entities based on values in settings
pub fn update_lights(settings: Const<Settings>, world: Mut<World>) {
    // Query ambient light entities
    let query = world.query::<(&mut Light,)>();

    for (light,) in query {
        match light {
            Light::Ambient { color, intensity } => {
                *color = settings.ambient_light_color;
                *intensity = settings.ambient_light_intensity;
            }
            Light::Simple {
                color, intensity, ..
            } => {
                *color = settings.simple_light_color;
                *intensity = settings.simple_light_intensity;
            }
            _ => continue,
        }
    }
}
