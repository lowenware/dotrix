use dotrix::camera;
use dotrix::egui::{self, Egui};
use dotrix::input::{ActionMapper, Button, KeyCode, Mapper};
use dotrix::overlay::{self, Overlay};
use dotrix::prelude::*;
use dotrix::sky::{skybox, SkyBox};
use dotrix::{Assets, CubeMap, Frame, Input, Pipeline, State, World};

/// In main state you can rotate camera and see FPS counter
struct MainState {
    name: String,
}

/// In paused state you can't rotate camera
struct PauseState {
    name: String,
    handled: bool,
}

fn main() {
    Dotrix::application("Dotrix: Demo Example")
        .with(System::from(startup))
        .with(System::from(ui_main).with(State::off::<PauseState>()))
        .with(System::from(ui_paused).with(State::on::<PauseState>()))
        .with(
            // Camera control should work only in Main state
            System::from(camera::control).with(State::on::<MainState>()),
        )
        .with(overlay::extension)
        .with(egui::extension)
        .with(skybox::extension)
        .run();
}

/// Initial game routines
fn startup(
    mut assets: Mut<Assets>,
    mut input: Mut<Input>,
    mut state: Mut<State>,
    mut world: Mut<World>,
) {
    // Push main state
    state.push(MainState {
        name: String::from("Main state"),
    });

    input.set_mapper(Box::new(Mapper::<Action>::new()));

    // Import skybox textures
    assets.import("assets/skybox-day/skybox_right.png");
    assets.import("assets/skybox-day/skybox_left.png");
    assets.import("assets/skybox-day/skybox_top.png");
    assets.import("assets/skybox-day/skybox_bottom.png");
    assets.import("assets/skybox-day/skybox_back.png");
    assets.import("assets/skybox-day/skybox_front.png");

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
        Pipeline::default(),
    )));

    // Map Escape key to Pause the game
    input
        .mapper_mut::<Mapper<Action>>()
        .set(vec![(Action::TogglePause, Button::Key(KeyCode::Escape))]);
}

/// Enumeration of actions provided by the game
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Action {
    TogglePause,
}

fn ui_main(
    mut state: Mut<State>,
    input: Const<Input>,
    overlay: Const<Overlay>,
    frame: Const<Frame>,
) {
    let egui_overlay = overlay
        .get::<Egui>()
        .expect("Egui overlay must be added on startup");

    let main_state = state
        .get::<MainState>()
        .expect("The system to be run in main state");

    if input.is_action_activated(Action::TogglePause) {
        state.push(PauseState {
            name: String::from("Paused State"),
            handled: false,
        });
        return;
    }

    egui::Area::new("Info label")
        .fixed_pos(egui::pos2(16.0, 16.0))
        .show(&egui_overlay.ctx, |ui| {
            ui.colored_label(
                egui::Rgba::from_rgb(255.0, 255.0, 255.0),
                format!("Press ESC to exit {} and pause", main_state.name),
            );
        });

    egui::Area::new("FPS counter")
        .fixed_pos(egui::pos2(16.0, 32.0))
        .show(&egui_overlay.ctx, |ui| {
            ui.colored_label(
                egui::Rgba::from_rgb(255.0, 255.0, 255.0),
                format!("FPS: {:.1}", frame.fps()),
            );
        });
}

fn ui_paused(mut state: Mut<State>, input: Const<Input>, overlay: Const<Overlay>) {
    let egui_overlay = overlay
        .get::<Egui>()
        .expect("Egui overlay must be added on startup");

    let states_stack_dump = state.dump().join(",\n  ");

    let mut pause_state = state
        .get_mut::<PauseState>()
        .expect("The system to be run in pause state");

    let mut exit_state = pause_state.handled && input.is_action_activated(Action::TogglePause);
    pause_state.handled = true;

    egui::containers::Window::new("Paused")
        .resizable(false)
        .default_width(200.0)
        .show(&egui_overlay.ctx, |ui| {
            ui.label("Execution is paused. Camera is not controllable");
            ui.label(format!(
                "Current states stack: [\n  {}\n]",
                states_stack_dump
            ));
            if ui.button("Resume").clicked() {
                exit_state = true;
            }
        });

    egui::Area::new("Info label")
        .fixed_pos(egui::pos2(16.0, 16.0))
        .show(&egui_overlay.ctx, |ui| {
            ui.colored_label(
                egui::Rgba::from_rgb(255.0, 255.0, 255.0),
                format!("Press ESC to exit {} and resume", pause_state.name),
            );
        });

    if exit_state {
        state.pop_any();
    }
}

/// Bind Inputs and Actions
impl ActionMapper<Action> for Input {
    fn action_mapped(&self, action: Action) -> Option<&Button> {
        let mapper = self.mapper::<Mapper<Action>>();
        mapper.get_button(action)
    }
}
