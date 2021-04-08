use crate::match_finder::{ MatchFinder, MatchFinderState };
use dotrix::{
    assets:: { Assets, Id, Texture },
    ecs::{ Const, Mut },
    egui::{
        CollapsingHeader,
        combo_box,
        Egui,
        Grid,
        SidePanel,
        TopPanel,
    },
    services::{ Camera, Frame, Renderer, Window },
    window::{ CursorIcon, UserAttentionType },
};
use std::collections::hash_map::HashMap;

pub struct Settings {
    fullscreen: bool,
    icon: String,
    icons: HashMap<String, Id<Texture>>,
    title: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            icon: String::from("none"),
            icons: HashMap::new(),
            title: String::new(),
        }
    }
}

pub fn ui(
    assets: Const<Assets>,
    camera: Const<Camera>,
    frame: Const<Frame>,
    mut match_finder: Mut<MatchFinder>,
    renderer: Const<Renderer>,
    mut settings: Mut<Settings>,
    mut window: Mut<Window>,
) {
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    TopPanel::top("top_panel").show(&egui.ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("🗙").clicked { println!("Close not implemented"); }
            if ui.button("🗕").clicked { window.set_minimized(true); }
            if window.maximized() {
                if ui.button("Ｓ").clicked { window.set_maximized(false); }
            } else {
                if ui.button("🗖").clicked { window.set_maximized(true); }
            }

            ui.horizontal(|ui| {
                if ui.text_edit_singleline(&mut settings.title).lost_kb_focus {
                    window.set_title(settings.title.as_str());
                };
            });
        });
    });

    SidePanel::left("side_panel", 300.0).show(&egui.ctx, |ui| {
        CollapsingHeader::new("Info")
            .default_open(false)
            .show(ui, |ui| {
                Grid::new("info_grid").show(ui, |ui| {
                    ui.label("FPS");
                    ui.label(format!("{}", frame.fps()));
                    ui.end_row();

                    ui.label("Window Inner Size");
                    ui.label(format!("x: {}, y: {}", window.inner_size().x, window.inner_size().y));
                    ui.end_row();

                    ui.label("Camera Position");
                    ui.label(format!("x: {:.4}, y: {:.4}, z: {:.4}",
                        camera.position().x, camera.position().y, camera.position().z));
                    ui.end_row();

                    let vec = format!("x: {:.4}, y: {:.4}, z: {:.4}",
                        camera.target.x, camera.target.y, camera.target.z);

                    ui.label("Camera Target");
                    ui.label(vec);
                    ui.end_row();

                    ui.label("Camera Disctance");
                    ui.label(format!("{:.4}", camera.distance));
                    ui.end_row();
                });
            });

        CollapsingHeader::new("Cursor")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("grid_cursor").show(ui, |ui| {
                    ui.label("Cursor icon");
                    let id = ui.make_persistent_id("cur_icon_combo_box");
                    let cur_icon = window.cursor_icon();
                    let mut new_cur_icon = cur_icon;

                    if combo_box(ui, id, format!("{:?}", new_cur_icon), |ui| {
                        for icon in CURSOR_ICONS.iter() {
                            ui.selectable_value(&mut new_cur_icon, *icon, format!("{:?}", icon));
                        }
                    }).active {
                        if cur_icon != new_cur_icon {
                            println!("cursor icon changed!");
                            window.set_cursor_icon(new_cur_icon);
                        }
                    };
                    ui.end_row();

                    ui.label("set Cursor visible");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.cursor_visible(), "true").clicked {
                            window.set_cursor_visible(true);
                        }
                        if ui.selectable_label(!window.cursor_visible(), "false").clicked {
                            window.set_cursor_visible(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Cursor grab");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.cursor_grab(), "true").clicked {
                            window.set_cursor_grab(true);
                        }
                        if ui.selectable_label(!window.cursor_grab(), "false").clicked {
                            window.set_cursor_grab(false);
                        }
                    });
                    ui.end_row();
                });
            });

        CollapsingHeader::new("Window")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("window").show(ui, |ui| {
                    ui.checkbox(&mut settings.fullscreen, "Fullscreen");
                    if ui.button("Apply").clicked {
                        window.set_fullscreen(settings.fullscreen)
                    };
                    ui.end_row();

                    ui.label("Window icon");
                    let id = ui.make_persistent_id("win_icon_combo_box");

                    let win_icon = String::from(settings.icon.as_str());
                    if combo_box(ui, id, format!("{}", settings.icon), |ui| {
                        ui.selectable_value(&mut settings.icon, String::from("dotrix"), "Dotrix");
                        ui.selectable_value(&mut settings.icon, String::from("lowenware"), "Lowenware");
                        ui.selectable_value(&mut settings.icon, String::from("rustacean"), "Rustacean");
                        ui.selectable_value(&mut settings.icon, String::from("None"), "None");
                    }).active {
                        if win_icon != settings.icon {
                            println!("window icon changed!");
                            if let Some(id) = settings.icons.get(&settings.icon) {
                                window.set_icon(assets.get(*id));
                            } else {
                                window.set_icon(None);
                            }
                        }
                    };
                    ui.end_row();

                    window.set_minimized(false);
                    ui.label("set Minimized");
                    ui.horizontal(|ui| {
                        if ui.button("true").clicked {
                            window.set_minimized(true);
                        }
                    });
                    ui.end_row();

                    ui.label("set Maximized");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.maximized(), "true").clicked {
                            window.set_maximized(true);
                        }
                        if ui.selectable_label(!window.maximized(), "false").clicked {
                            window.set_maximized(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Decorations");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.decorations(), "true").clicked {
                            window.set_decorations(true);
                        }
                        if ui.selectable_label(!window.decorations(), "false").clicked {
                            window.set_decorations(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Resizable");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.resizable(), "true").clicked {
                            window.set_resizable(true);
                        }
                        if ui.selectable_label(!window.resizable(), "false").clicked {
                            window.set_resizable(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Always on Top");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.always_on_top(), "true").clicked {
                            window.set_always_on_top(true);
                        }
                        if ui.selectable_label(!window.always_on_top(), "false").clicked {
                            window.set_always_on_top(false);
                        }
                    });
                    ui.end_row();
                });
            });

            CollapsingHeader::new("Match Finder")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("This is a demonstration of user attention type.");
                ui.label("Click \"Search for game\" button, then switch to other program. App will request attention after 10 seconds (on windows - icon on taskbar starts blinking).");
                Grid::new("match_finder").show(ui, |ui| {
                    ui.label("User Attention Type");
                    let id = ui.make_persistent_id("attention_combo_box");
                    combo_box(ui, id, format!("{:?}", match_finder.attention_type), |ui| {
                        for a_type in [UserAttentionType::Informational, UserAttentionType::Critical].iter() {
                            ui.selectable_value(
                                &mut match_finder.attention_type,
                                *a_type,
                                format!("{:?}", a_type)
                            );
                        }
                    });
                    ui.end_row();

                    match match_finder.state {
                        MatchFinderState::Idle => {
                            if ui.button("Search for game").clicked {
                                match_finder.start_searching();
                            }
                        },
                        MatchFinderState::Searching(from) => {
                            ui.label("Searching...");
                            ui.end_row();
                            if let Some(from) = from {
                                ui.label(format!("Estimated time: {:.0}s", match_finder.estimated_time));
                                ui.end_row();
                                let search_time_secs = (frame.time() - from).as_secs_f32();
                                ui.label(format!("Search time: {:.0}s", search_time_secs));
                                ui.end_row();
                            }
                            if ui.button("Stop searching").clicked {
                                match_finder.stop_searching();
                            }
                        },
                        MatchFinderState::Ready(until) => {
                            ui.label("Game is ready!");
                            ui.end_row();
                            let remaining_time_secs = (until - frame.time()).as_secs_f32();
                            ui.label(format!("You have {:.0}s to enter the game!", remaining_time_secs));
                                ui.end_row();
                            if ui.button("Enter the game").clicked {
                                match_finder.stop_searching();
                            }
                        },
                        _ => {},
                    }
                });
            });
    });
}

pub fn startup(
    mut assets: Mut<Assets>,
    mut renderer: Mut<Renderer>,
    mut settings: Mut<Settings>,
    window: Const<Window>,
) {
    renderer.add_overlay(Box::new(Egui::default()));
    settings.title = String::from(window.title());

    // Load icons
    for name in ["dotrix", "lowenware", "rustacean"].iter() {
        settings.icons.insert(String::from(*name), assets.register::<Texture>(name));
        assets.import(format!("examples/window/assets/{}.png", name).as_str());
    }
}

const CURSOR_ICONS: &[CursorIcon] = &[
    CursorIcon::Default,
    CursorIcon::Crosshair,
    CursorIcon::Alias,
    CursorIcon::AllScroll,
    CursorIcon::Arrow,
    CursorIcon::Cell,
    CursorIcon::ColResize,
    CursorIcon::ContextMenu,
    CursorIcon::Copy,
    CursorIcon::EResize,
    CursorIcon::EwResize,
    CursorIcon::Grab,
    CursorIcon::Grabbing,
    CursorIcon::Hand,
    CursorIcon::Help,
    CursorIcon::Move,
    CursorIcon::NResize,
    CursorIcon::NeResize,
    CursorIcon::NeswResize,
    CursorIcon::NoDrop,
    CursorIcon::NotAllowed,
    CursorIcon::NsResize,
    CursorIcon::NwResize,
    CursorIcon::NwseResize,
    CursorIcon::Progress,
    CursorIcon::RowResize,
    CursorIcon::SResize,
    CursorIcon::SeResize,
    CursorIcon::SwResize,
    CursorIcon::Text,
    CursorIcon::VerticalText,
    CursorIcon::WResize,
    CursorIcon::Wait,
    CursorIcon::ZoomIn,
    CursorIcon::ZoomOut,
];
