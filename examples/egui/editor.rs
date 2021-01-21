use crate::fox::{ Fox, FoxAnimClip };
use dotrix::{
    animation::State as AnimState,
    components::{ Animator, Light, Model },
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
    math::{ Point3, Vec3, Vec4 },
    renderer::transform::Transform,
    services::{ Camera, Frame, Input, Renderer, World },
};
use std::f32::consts::PI;

pub struct Editor {
    pub fox_transform: Transform,

    pub anim_clip: FoxAnimClip,
    pub anim_play: bool,
    pub anim_speed: f32,

    pub cam_distance: f32,
    pub cam_y_angle: f32,
    pub cam_xz_angle: f32,
    pub cam_target: Point3,

    pub light_color: Vec4,
}

impl Editor {
    pub fn new() -> Self {
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

            light_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    /// Reset to default values
    pub fn reset(&mut self) {
        let default = Editor::new();

        self.fox_transform = default.fox_transform;
        self.anim_clip = default.anim_clip;
        self.anim_play = default.anim_play;

        self.cam_distance = default.cam_distance;
        self.cam_y_angle = default.cam_y_angle;
        self.cam_xz_angle = default.cam_xz_angle;
        self.cam_target = default.cam_target;

        self.light_color = default.light_color;
    }
}

pub fn ui(mut editor: Mut<Editor>, renderer: Mut<Renderer>) {
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    SidePanel::left("side_panel", 300.0).show(&egui.ctx, |ui| {
        CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("transform").show(ui, |ui| {
                    ui.label("Translation");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut editor.fox_transform.translate.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::f32(&mut editor.fox_transform.translate.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::f32(&mut editor.fox_transform.translate.z).prefix("z: ").speed(0.1));
                    });
                    ui.end_row();

                    ui.label("Rotation");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut editor.fox_transform.rotate.v.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::f32(&mut editor.fox_transform.rotate.v.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::f32(&mut editor.fox_transform.rotate.v.z).prefix("z: ").speed(0.01));
                    });
                    ui.end_row();

                    ui.label("Scale");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut editor.fox_transform.scale.x).prefix("x: ").speed(0.01));
                        ui.add(DragValue::f32(&mut editor.fox_transform.scale.y).prefix("y: ").speed(0.01));
                        ui.add(DragValue::f32(&mut editor.fox_transform.scale.z).prefix("z: ").speed(0.01));
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
                    combo_box(ui, id, format!("{:?}", editor.anim_clip), |ui| {
                        // There should be some EnumIterator in real implementation
                        ui.selectable_value(&mut editor.anim_clip, FoxAnimClip::Walk, format!("{:?}", FoxAnimClip::Walk));
                        ui.selectable_value(&mut editor.anim_clip, FoxAnimClip::Run, format!("{:?}", FoxAnimClip::Run));
                        ui.selectable_value(&mut editor.anim_clip, FoxAnimClip::Survey, format!("{:?}", FoxAnimClip::Survey));
                    });

                    if ui.button(if editor.anim_play {"Stop"} else {"Play"}).clicked {
                        editor.anim_play = !editor.anim_play;
                    };
                    ui.end_row();

                    ui.label("Speed");
                    ui.add(Slider::f32(&mut editor.anim_speed, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        editor.anim_speed = 1.0;
                    };
                });
            });

            CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("Target").show(ui, |ui| {
                    ui.label("Target");
                    ui.horizontal(|ui| {
                        ui.add(DragValue::f32(&mut editor.cam_target.x).prefix("x: ").speed(0.1));
                        ui.add(DragValue::f32(&mut editor.cam_target.y).prefix("y: ").speed(0.1));
                        ui.add(DragValue::f32(&mut editor.cam_target.z).prefix("z: ").speed(0.1));
                        if ui.button("¤").on_hover_text("Target fox").clicked {
                            editor.cam_target.x = editor.fox_transform.translate.x;
                            editor.cam_target.y = editor.fox_transform.translate.y;
                            editor.cam_target.z = editor.fox_transform.translate.z;
                        };
                    });
                    ui.end_row();

                    ui.label("Distance");
                    ui.add(DragValue::f32(&mut editor.cam_distance).speed(0.1));
                    ui.end_row();

                    ui.label("Y Angle");
                    ui.add(DragValue::f32(&mut editor.cam_y_angle).speed(0.01));
                    ui.end_row();

                    ui.label("YZ Angle");
                    ui.add(DragValue::f32(&mut editor.cam_xz_angle).speed(0.01));
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Light")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("light").show(ui, |ui| {
                    ui.label("Red");
                    ui.add(Slider::f32(&mut editor.light_color.x, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        editor.light_color.x = 1.0;
                    };
                    ui.end_row();

                    ui.label("Green");
                    ui.add(Slider::f32(&mut editor.light_color.y, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        editor.light_color.y = 1.0;
                    };
                    ui.end_row();

                    ui.label("Blue");
                    ui.add(Slider::f32(&mut editor.light_color.z, 0.0..=3.0).text(""));
                    if ui.button("↺").on_hover_text("Reset value").clicked {
                        editor.light_color.z = 1.0;
                    };
                    ui.end_row();
                });
            });

        if ui.button("Reset all").clicked {
            editor.reset();
        };
    });
}

pub fn startup(mut renderer: Mut<Renderer>) {
    renderer.add_overlay(Box::new(Egui::default()));
}

/// This func updates camera based on values in editor and controls
pub fn update_camera(mut camera: Mut<Camera>, mut editor: Mut<Editor>, input: Const<Input>, frame: Const<Frame>) {
    const ROTATE_SPEED: f32 = PI / 10.0;
    const ZOOM_SPEED: f32 = 500.0;

    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    // Get values from editor
    let mut distance = editor.cam_distance;
    let target = editor.cam_target;
    let mut y_angle = editor.cam_y_angle;
    let mut xz_angle = editor.cam_xz_angle;

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

    // Apply values to editor
    editor.cam_distance = distance;
    editor.cam_y_angle = y_angle;
    editor.cam_xz_angle = xz_angle;

    // Apply values to camera
    camera.target = target;
    camera.distance = distance;
    camera.y_angle = y_angle;
    camera.xz_angle = xz_angle;
    camera.set_view();
}

/// This func updates fox's entity based on values in editor
pub fn update_fox(
    editor: Const<Editor>,
    world: Mut<World>,
) {
    // Query fox entity
    let query = world.query::<(&mut Model, &mut Animator, &mut Fox)>();

    // This loop will run only once, because Fox component is assigned to only one entity
    for (model, animator, fox) in query {

        // Set transformation
        model.transform.translate = editor.fox_transform.translate;
        model.transform.rotate = editor.fox_transform.rotate;
        model.transform.scale = editor.fox_transform.scale;

        // Set animation state
        match animator.state() {
            AnimState::Loop(_) => {
                if !editor.anim_play {
                    animator.stop();
                }
            }
            AnimState::Stop => {
                if editor.anim_play {
                    animator.start_loop();
                }
            }
            _ => {}
        };

        // Set animation clip
        let clip = fox.animations[&editor.anim_clip];
        if animator.animation() != clip {
            animator.animate(clip);
        }

        // Set animation speed
        animator.speed = editor.anim_speed;
    }
}

/// This func updates all light entities based on values in editor
pub fn update_lights(
    editor: Const<Editor>,
    world: Mut<World>,
) {
    // Query fox light entities
    let query = world.query::<(&mut Light,)>();

    for (light,) in query {
        light.color = editor.light_color;
    }

}