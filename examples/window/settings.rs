use crate::match_finder::{ MatchFinder, MatchFinderState };
use dotrix::ecs::{ Const, Mut };
use dotrix::assets:: { Assets, Texture };
use dotrix::{ Id, Frame, Input, Window };
use dotrix::egui::{
    CollapsingHeader,
    ComboBox,
    Egui,
    Grid,
    ScrollArea,
    SidePanel,
    Slider,
    TopBottomPanel,
};
use dotrix::overlay::Overlay;
use dotrix::math::{ Vec2i, Vec2u };
use dotrix::window::{ CursorIcon, Fullscreen, UserAttentionType, VideoMode };

use std::collections::hash_map::HashMap;

pub struct Settings {
    current_monitor_number: usize,
    current_video_mode: Option<VideoMode>,
    icon: String,
    icons: HashMap<String, Id<Texture>>,
    min_inner_size: Vec2u,
    title: String,
    window_mode: WindowMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current_monitor_number: 0,
            current_video_mode: None,
            icon: String::from("none"),
            icons: HashMap::new(),
            min_inner_size: Vec2u::new(640, 480),
            title: String::new(),
            window_mode: WindowMode::Windowed,
        }
    }
}

pub fn ui(
    assets: Const<Assets>,
    frame: Const<Frame>,
    input: Const<Input>,
    mut match_finder: Mut<MatchFinder>,
    overlay: Const<Overlay>,
    mut settings: Mut<Settings>,
    mut window: Mut<Window>,
) {
    let egui = overlay.get::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    TopBottomPanel::top("top_panel").show(&egui.ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("🗙").clicked() { window.close(); }
            if ui.button("🗕").clicked() { window.set_minimized(true); }
            if window.maximized() {
                if ui.button("Ｓ").clicked() { window.set_maximized(false); }
            } else if ui.button("🗖").clicked() { window.set_maximized(true); }

            ui.horizontal(|ui| {
                if ui.text_edit_singleline(&mut settings.title).lost_focus() {
                    window.set_title(settings.title.as_str());
                };
            });
        });
    });

    SidePanel::left("side_panel")
    .show(&egui.ctx, |ui| {
        ScrollArea::auto_sized().show(ui, |ui| {
            CollapsingHeader::new("ℹ Info")
            .default_open(false)
            .show(ui, |ui| {
                Grid::new("info_grid").show(ui, |ui| {
                    ui.label("FPS");
                    ui.label(format!("{}", frame.fps()));
                    ui.end_row();

                    ui.label("Screen size");
                    ui.label(format!("x: {}, y: {}", window.screen_size().x, window.screen_size().y));
                    ui.end_row();
                });

                CollapsingHeader::new("ℹ Cursor")
                .default_open(false)
                .show(ui, |ui| {
                    Grid::new("info_cursor_grid").show(ui, |ui| {
                        ui.label("Position");
                        ui.label(format!("x: {}, y: {}",
                            input.mouse_position().unwrap().x, input.mouse_position().unwrap().y));
                        ui.end_row();

                        ui.label("Normalized position");
                        ui.label(format!("x: {:.4}, y: {:.4}",
                            input.mouse_position_normalized().x,
                            input.mouse_position_normalized().y
                        ));
                        ui.end_row();

                        ui.label("Delta");
                        ui.label(format!("x: {}, y: {}",
                            input.mouse_delta().x,
                            input.mouse_delta().y
                        ));
                        ui.end_row();
                    });
                });

                CollapsingHeader::new("ℹ Monitors")
                .default_open(false)
                .show(ui, |ui| {
                    for monitor in window.monitors() {
                        CollapsingHeader::new(format!("Monitor {}", monitor.number))
                        .default_open(false)
                        .show(ui, |ui| {
                            Grid::new(format!("info_monitors_{}", monitor.name)).show(ui, |ui| {
                                ui.label("Number");
                                ui.label(monitor.number.to_string());
                                ui.end_row();

                                ui.label("Name");
                                ui.label(monitor.name.to_string());
                                ui.end_row();

                                ui.label("Scale Factor");
                                ui.label(monitor.scale_factor.to_string());
                                ui.end_row();

                                ui.label("Size");
                                ui.label(format!("x: {}, y: {}", monitor.size.x, monitor.size.y));
                                ui.end_row();
                            });
                        });
                    }
                });

                CollapsingHeader::new("ℹ Window")
                .default_open(true)
                .show(ui, |ui| {
                    Grid::new("info_window_grid").show(ui, |ui| {
                        ui.label("Inner Position");
                        ui.label(format!("x: {}, y: {}", window.inner_position().x, window.inner_position().y));
                        ui.end_row();

                        ui.label("Inner Size");
                        ui.label(format!("x: {}, y: {}", window.inner_size().x, window.inner_size().y));
                        ui.end_row();

                        ui.label("Min. Inner Size");
                        ui.label(format!("x: {}, y: {}", window.min_inner_size().x, window.min_inner_size().y));
                        ui.end_row();

                        ui.label("Outer Position");
                        ui.label(format!("x: {}, y: {}", window.outer_position().x, window.outer_position().y));
                        ui.end_row();

                        ui.label("Outer Size");
                        ui.label(format!("x: {}, y: {}", window.outer_size().x, window.outer_size().y));
                        ui.end_row();

                        ui.label("Scale factor");
                        ui.label(format!("{}", window.scale_factor()));
                        ui.end_row();
                    });
                });
            });

        CollapsingHeader::new("🖱 Cursor")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("grid_cursor").show(ui, |ui| {
                    ui.label("Icon");
                    let cur_icon = window.cursor_icon();
                    let mut new_cur_icon = cur_icon;

                    ComboBox::from_id_source("Icon")
                        .selected_text(format!("{:?}", new_cur_icon))
                        .show_ui(ui, |ui| {
                            for icon in CURSOR_ICONS.iter() {
                                ui.selectable_value(&mut new_cur_icon, *icon, format!("{:?}", icon));
                            }
                        });

                    if cur_icon != new_cur_icon {
                        window.set_cursor_icon(new_cur_icon);
                    }
                    ui.end_row();

                    ui.label("set Cursor visible");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.cursor_visible(), "true").clicked() {
                            window.set_cursor_visible(true);
                        }
                        if ui.selectable_label(!window.cursor_visible(), "false").clicked() {
                            window.set_cursor_visible(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Cursor grab");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.cursor_grab(), "true").clicked() {
                            window.set_cursor_grab(true);
                        }
                        if ui.selectable_label(!window.cursor_grab(), "false").clicked() {
                            window.set_cursor_grab(false);
                        }
                    });
                    ui.end_row();
                });
            });

        CollapsingHeader::new("Ｓ Window")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("window").show(ui, |ui| {
                    ui.label("Display mode");

                    ComboBox::from_id_source("Display mode")
                        .selected_text(format!("{:?}", settings.window_mode))
                        .show_ui(ui, |ui| {
                            for mode in WINDOW_MODES.iter() {
                                ui.selectable_value(&mut settings.window_mode, *mode, format!("{:?}", mode));
                            }
                        });
                    ui.end_row();

                    if WindowMode::BorderlessFullscreen == settings.window_mode
                    || WindowMode::Fullscreen == settings.window_mode {
                        let mut current_monitor_number = settings.current_monitor_number;
                        let mut current_monitor = &window.monitors()[current_monitor_number];

                        ui.label("Monitor");
                        ComboBox::from_id_source("Monitor")
                            .selected_text(format!("{:?}", current_monitor.name))
                            .show_ui(ui, |ui| {
                                for (i, monitor) in window.monitors().iter().enumerate() {
                                    ui.selectable_value(&mut current_monitor_number, i, &monitor.name);
                                }
                            });
                        if current_monitor_number != settings.current_monitor_number {
                            settings.current_monitor_number = current_monitor_number;
                            settings.current_video_mode = None;
                            current_monitor = &window.monitors()[current_monitor_number];
                        }
                        ui.end_row();

                        if WindowMode::Fullscreen == settings.window_mode {
                            let video_modes = &current_monitor.video_modes;
                            ui.label("Video mode");
                            if settings.current_video_mode.is_none() {
                                settings.current_video_mode = Some(video_modes[0]);
                            }
                            let mut video_mode = settings.current_video_mode;

                            ComboBox::from_id_source("Video mode")
                                .selected_text(format!("{:?}", video_mode.unwrap()))
                                .show_ui(ui, |ui| {
                                    for mode in video_modes {
                                        ui.selectable_value(&mut video_mode, Some(*mode), mode.to_string());
                                    }
                                });
                            settings.current_video_mode = video_mode;
                            ui.end_row();
                        }
                    }

                    ui.label("");
                    if ui.button("Apply").clicked() {
                        match settings.window_mode {
                            WindowMode::Fullscreen => window.set_fullscreen(
                                Some(Fullscreen::Exclusive(settings.current_video_mode.unwrap()))
                            ),
                            WindowMode::BorderlessFullscreen => window.set_fullscreen(
                                Some(Fullscreen::Borderless(settings.current_monitor_number))
                            ),
                            WindowMode::Windowed => window.set_fullscreen(None),
                        }
                    }

                    ui.end_row();

                    ui.label("Icon");

                    let win_icon = String::from(settings.icon.as_str());

                    ComboBox::from_id_source("Icon")
                        .selected_text(settings.icon.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.icon, String::from("dotrix"), "Dotrix");
                            ui.selectable_value(&mut settings.icon, String::from("lowenware"), "Lowenware");
                            ui.selectable_value(&mut settings.icon, String::from("rustacean"), "Rustacean");
                            ui.selectable_value(&mut settings.icon, String::from("None"), "None");
                        });

                    if win_icon != settings.icon {
                        if let Some(id) = settings.icons.get(&settings.icon) {
                            window.set_icon(assets.get(*id));
                        } else {
                            window.set_icon(None);
                        }
                    }
                    ui.end_row();

                    ui.label("set Minimized");
                    ui.horizontal(|ui| {
                        if ui.button("true").clicked() {
                            window.set_minimized(true);
                        }
                    });
                    ui.end_row();

                    ui.label("set Maximized");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.maximized(), "true").clicked() {
                            window.set_maximized(true);
                        }
                        if ui.selectable_label(!window.maximized(), "false").clicked() {
                            window.set_maximized(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Decorations");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.decorations(), "true").clicked() {
                            window.set_decorations(true);
                        }
                        if ui.selectable_label(!window.decorations(), "false").clicked() {
                            window.set_decorations(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Resizable");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.resizable(), "true").clicked() {
                            window.set_resizable(true);
                        }
                        if ui.selectable_label(!window.resizable(), "false").clicked() {
                            window.set_resizable(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Always on Top");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(window.always_on_top(), "true").clicked() {
                            window.set_always_on_top(true);
                        }
                        if ui.selectable_label(!window.always_on_top(), "false").clicked() {
                            window.set_always_on_top(false);
                        }
                    });
                    ui.end_row();

                    ui.label("set Min Inner Size - x");
                    ui.add(Slider::new(&mut settings.min_inner_size.x, 100..=window.screen_size().x)
                        .text(""));
                    ui.end_row();

                    ui.label("set Min Inner Size - y");
                    ui.add(Slider::new(&mut settings.min_inner_size.y, 100..=window.screen_size().y)
                        .text(""));
                    ui.end_row();

                    if settings.min_inner_size != window.min_inner_size() {
                        window.set_min_inner_size(settings.min_inner_size);
                    };

                    let mut inner_size = window.inner_size();
                    ui.label("set Inner Size - x");
                    ui.add(Slider::new(&mut inner_size.x, 100..=window.screen_size().x)
                        .text(""));
                    ui.end_row();

                    ui.label("set Inner Size - y");
                    ui.add(Slider::new(&mut inner_size.y, 100..=window.screen_size().y)
                        .text(""));
                    ui.end_row();

                    if window.inner_size() != inner_size {
                        window.set_inner_size(inner_size);
                    }

                    // TODO: we should use work area for that
                    // https://docs.microsoft.com/en-gb/windows/win32/api/winuser/nf-winuser-systemparametersinfoa
                    ui.label("Move");
                    Grid::new("window_movement_grid").show(ui, |ui| {
                        ui.label("");
                        ui.vertical_centered(|ui| {
                            if ui.button("Top").clicked() {
                                move_window_top(&window);
                            };
                        });
                        ui.label("");
                        ui.end_row();

                        ui.end_row();
                        if ui.button("Left").clicked() {
                            move_window_left(&window);
                        };
                        ui.vertical_centered(|ui| {
                            if ui.button("Center").clicked() {
                                move_window_center(&window);
                            };
                        });
                        if ui.button("Right").clicked() {
                            move_window_right(&window);
                        };
                        ui.end_row();

                        ui.label("");
                        ui.vertical_centered(|ui| {
                            if ui.button("Bottom").clicked() {
                                move_window_bottom(&window);
                            };
                        });
                        ui.label("");
                        ui.end_row();
                    });
                });
            });

            CollapsingHeader::new("🚨 Match Finder")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("This is a demonstration of user attention type.");
                ui.label("Click \"Search for game\" button, then switch to other program. App will request attention after 10 seconds (on windows - icon on taskbar starts blinking).");
                Grid::new("match_finder").show(ui, |ui| {
                    ui.label("User Attention Type");
                    ComboBox::from_id_source("match_finder")
                        .selected_text(format!("{:?}", match_finder.attention_type))
                        .show_ui(ui, |ui| {
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
                            if ui.button("Search for game").clicked() {
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
                            if ui.button("Stop searching").clicked() {
                                match_finder.stop_searching();
                            }
                        },
                        MatchFinderState::Ready(until) => {
                            ui.label("Game is ready!");
                            ui.end_row();
                            let remaining_time_secs = (until - frame.time()).as_secs_f32();
                            ui.label(format!("You have {:.0}s to enter the game!", remaining_time_secs));
                                ui.end_row();
                            if ui.button("Enter the game").clicked() {
                                match_finder.stop_searching();
                            }
                        },
                        _ => {},
                    }
                });
            });
        });
    });
}

pub fn startup(
    mut assets: Mut<Assets>,
    mut settings: Mut<Settings>,
    window: Const<Window>,
) {
    settings.title = String::from(window.title());

    // Load icons
    for name in ["dotrix", "lowenware", "rustacean"].iter() {
        settings.icons.insert(String::from(*name), assets.register::<Texture>(name));
        assets.import(format!("assets/{}.png", name).as_str());
    }
}

fn move_window_left(window: &Window) {
    window.set_outer_position(Vec2i::new(0, window.outer_position().y));
}

fn move_window_right(window: &Window) {
    window.set_outer_position(
        Vec2i::new(
            window.screen_size().x as i32 - window.outer_size().x as i32,
            window.outer_position().y,
        )
    );
}

fn move_window_top(window: &Window) {
    window.set_outer_position(Vec2i::new(window.outer_position().x, 0));
}

fn move_window_bottom(window: &Window) {
    window.set_outer_position(
        Vec2i::new(
            window.outer_position().x,
            window.screen_size().y as i32 - window.outer_size().y as i32,
        )
    );
}

fn move_window_center(window: &Window) {
    window.set_outer_position(
        Vec2i::new(
            (window.screen_size().x - window.outer_size().x) as i32 / 2,
            (window.screen_size().y - window.outer_size().y) as i32 / 2,
        )
    );
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

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum WindowMode {
    Fullscreen,
    BorderlessFullscreen,
    Windowed,
}

const WINDOW_MODES: &[WindowMode] = &[
    WindowMode::Fullscreen,
    WindowMode::BorderlessFullscreen,
    WindowMode::Windowed,
];
