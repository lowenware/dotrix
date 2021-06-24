use crate::fox::{ Fox, FoxAnimClip };
use dotrix::{
    animation::State as AnimState,
    components::{ AmbientLight, Animator, SimpleLight, Model },
    ecs::{ Const, Mut },
    egui::{
        CollapsingHeader,
        combo_box,
        DragValue,
        Egui,
        Grid,
        SidePanel,
        Slider,
    },
    input::{ Button, State as InputState },
    math::{ Point3, Vec3 },
    renderer::{ Transform },
    services::{ Camera, Frame, Input, Renderer, World },
};
use std::f32::consts::PI;

pub struct Settings {
    pub fox_transform: Transform,

    pub anim_clip: FoxAnimClip,
    pub anim_play: bool,
    pub anim_speed: f32,

    pub cam_distance: f32,
    pub cam_y_angle: f32,
    pub cam_xz_angle: f32,
    pub cam_target: Point3,

    // Lights here are only structs, not real components.
    pub amb_light: AmbientLight,
    pub light: SimpleLight,
}

impl Settings {
    /// Reset to default values
    pub fn reset(&mut self) {
        let default = Settings::default();

        self.fox_transform = default.fox_transform;

        self.anim_clip = default.anim_clip;
        self.anim_play = default.anim_play;
        self.anim_speed = default.anim_speed;

        self.cam_distance = default.cam_distance;
        self.cam_y_angle = default.cam_y_angle;
        self.cam_xz_angle = default.cam_xz_angle;
        self.cam_target = default.cam_target;

        self.light = default.light;
        self.amb_light = default.amb_light;
    }

}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fox_transform: Transform {
                translate: Vec3 {x: 80.0, y: 0.0, z: 0.0},
                ..Default::default()
            },

            anim_clip: FoxAnimClip::Walk,
            anim_play: true,
            anim_speed: 1.0,

            cam_distance: 222.0,
            cam_y_angle: 0.74,
            cam_xz_angle: 0.25,
            cam_target: Point3::new(0.0, 0.5, 0.0),

            light: SimpleLight::default(),
            amb_light: AmbientLight::default(),
        }
    }
}

pub fn ui(mut settings: Mut<Settings>, renderer: Const<Renderer>) {
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    SidePanel::left("side_panel", 300.0).show(&egui.ctx, |ui| {
        CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("transform").show(ui, |ui| {
                    ui.label("Translation");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.fox_transform.translate.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.fox_transform.translate.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.fox_transform.translate.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Rotation");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.fox_transform.rotate.v.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.fox_transform.rotate.v.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.fox_transform.rotate.v.z).prefix("z: ").speed(0.01));
                    });
                    ui.end_row();

                    ui.label("Scale");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut settings.fox_transform.scale.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.fox_transform.scale.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::f32(&mut settings.fox_transform.scale.z).prefix("z: ").speed(0.01));
                    });
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Animation")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("animation").show(ui, |ui| {
                    ui.label("Clip");
                    let id = ui.make_persistent_id("clip_combo_box");
                    combo_box(ui, id, format!("{:?}", settings.anim_clip), |ui| {
                        // There should be some EnumIterator in real implementation
                        ui.selectable_value(&mut settings.anim_clip, FoxAnimClip::Walk, format!("{:?}", FoxAnimClip::Walk));
                        ui.selectable_value(&mut settings.anim_clip, FoxAnimClip::Run, format!("{:?}", FoxAnimClip::Run));
                        ui.selectable_value(&mut settings.anim_clip, FoxAnimClip::Survey, format!("{:?}", FoxAnimClip::Survey));
                    });

                    if ui.button(if settings.anim_play {"Stop"} else {"Play"}).clicked {
                        settings.anim_play = !settings.anim_play;
                    };
                    ui.end_row();

                    ui.label("Speed");
                    ui.add(Slider::f32(&mut settings.anim_speed, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
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
                        ui.add(DragValue::f32(&mut settings.cam_target.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.cam_target.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::f32(&mut settings.cam_target.z).prefix("z: ").speed(0.1));
                        if ui.button("¤").on_hover_text("Target fox").clicked {
                            settings.cam_target.x = settings.fox_transform.translate.x;
                            settings.cam_target.y = settings.fox_transform.translate.y;
                            settings.cam_target.z = settings.fox_transform.translate.z;
                        };
                    });
                    ui.end_row();

                    ui.label("Distance");
                    ui.add(DragValue::f32(&mut settings.cam_distance).speed(0.1));
                    ui.end_row();

                    ui.label("Y Angle");
                    ui.add(DragValue::f32(&mut settings.cam_y_angle).speed(0.01));
                    ui.end_row();

                    ui.label("YZ Angle");
                    ui.add(DragValue::f32(&mut settings.cam_xz_angle).speed(0.01));
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("light").show(ui, |ui| {
                    ui.label("Red");
                    ui.add(Slider::f32(&mut settings.light.color.r, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.light.color.r = Settings::default().light.color.r;
                    };
                    ui.end_row();

                    ui.label("Green");
                    ui.add(Slider::f32(&mut settings.light.color.g, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.light.color.g = Settings::default().light.color.g;
                    };
                    ui.end_row();

                    ui.label("Blue");
                    ui.add(Slider::f32(&mut settings.light.color.b, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.light.color.b = Settings::default().light.color.b;
                    };
                    ui.end_row();

                    ui.label("Intensity");
                    ui.add(Slider::f32(&mut settings.light.intensity, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.light.intensity = Settings::default().light.intensity;
                    };
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Ambient Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("ambient light").show(ui, |ui| {
                    ui.label("Red");
                    ui.add(Slider::f32(&mut settings.amb_light.color.r, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.amb_light.color.r = Settings::default().amb_light.color.r;
                    };
                    ui.end_row();

                    ui.label("Green");
                    ui.add(Slider::f32(&mut settings.amb_light.color.g, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.amb_light.color.g = Settings::default().amb_light.color.g;
                    };
                    ui.end_row();

                    ui.label("Blue");
                    ui.add(Slider::f32(&mut settings.amb_light.color.b, 0.0..=1.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        settings.amb_light.color.b = Settings::default().amb_light.color.b;
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

        if ui.button("Reset all").clicked {
            settings.reset();
        };
    });
}

pub fn startup(mut renderer: Mut<Renderer>) {
    renderer.add_overlay(Box::new(Egui::default()));
}

/// This func updates camera based on values in settings and controls
pub fn update_camera(mut camera: Mut<Camera>, mut settings: Mut<Settings>, input: Const<Input>, frame: Const<Frame>) {
    const ROTATE_SPEED: f32 = PI / 10.0;
    const ZOOM_SPEED: f32 = 500.0;

    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    // Get values from settings
    let mut distance = settings.cam_distance;
    let target = settings.cam_target;
    let mut y_angle = settings.cam_y_angle;
    let mut xz_angle = settings.cam_xz_angle;

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

    // Apply values to settings
    settings.cam_distance = distance;
    settings.cam_y_angle = y_angle;
    settings.cam_xz_angle = xz_angle;

    // Apply values to camera
    camera.target = target;
    camera.distance = distance;
    camera.y_angle = y_angle;
    camera.xz_angle = xz_angle;
    camera.set_view();
}

/// This func updates fox's entity based on values in settings
pub fn update_fox(
    settings: Const<Settings>,
    world: Mut<World>,
) {
    // Query fox entity
    let query = world.query::<(&mut Model, &mut Animator, &mut Fox)>();

    // This loop will run only once, because Fox component is assigned to only one entity
    for (model, animator, fox) in query {

        // Set transformation
        model.transform.translate = settings.fox_transform.translate;
        model.transform.rotate = settings.fox_transform.rotate;
        model.transform.scale = settings.fox_transform.scale;

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
pub fn update_lights(
    settings: Const<Settings>,
    world: Mut<World>,
) {
    // Query ambient light entities
    let query = world.query::<(&mut AmbientLight,)>();

    for (amb_light,) in query {
        amb_light.clone_from(&settings.amb_light);
    }

    // Query simple light entities
    let query = world.query::<(&mut SimpleLight,)>();

    for (light,) in query {
        light.color = settings.light.color;
        light.intensity = settings.light.intensity;
    }
}
