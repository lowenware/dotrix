//! EGUI integration interface
//!
//! Dotrix provides a simple
//! [example](https://github.com/lowenware/dotrix/blob/main/examples/egui/main.rs)
//! demonstrating the EGUI integration.

#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

/// Extra widgets provided by dotrix.
pub mod extras;

use dotrix_core::{ Application, Id, Pipeline, Assets, Input, Window };
use dotrix_core::assets::{ Mesh, Texture };
use dotrix_core::ecs::{ Mut, System };
use dotrix_core::input::{
    Button,
    Event as InputEvent,
    KeyCode,
    Modifiers,
    State as InputState
};

use dotrix_overlay::{ Overlay, Ui, Widget };

pub use egui::*;

const TEXTURE_NAME: &str = "egui::texture";
const SCROLL_SENSITIVITY: f32 = 10.0;

/// EGUI overlay provider
pub struct Egui {
    /// EGUI context
    pub ctx: egui::CtxRef,
    /// UI scale factor
    pub scale_factor: f32,
    widgets: Vec<(Widget, Pipeline)>,
    texture: Option<Id<Texture>>,
    texture_version: Option<u64>,
}

impl Default for Egui {
    fn default() -> Self {
        Egui {
            ctx: egui::CtxRef::default(),
            scale_factor: 1.0,
            widgets: Vec::new(),
            texture: None,
            texture_version: None,
        }
    }
}

impl Ui for Egui {
    fn bind(
        &mut self,
        assets: &mut Assets,
        input: &mut Input,
        window: &Window,
    ) {
        let scale_factor = self.scale_factor * window.scale_factor();
        let surface_size = window.inner_size();
        let surface_width = surface_size.x as f32;
        let surface_height = surface_size.y as f32;

        let mut events = input.events.iter()
            .map(|e| match e {
                InputEvent::Copy => Some(egui::Event::Copy),
                InputEvent::Cut => Some(egui::Event::Cut),
                InputEvent::Text(text) => Some(egui::Event::Text(String::from(text))),
                InputEvent::Key(event) => to_egui_key_code(event.key_code)
                    .map(|key| egui::Event::Key {
                        key,
                        pressed: event.pressed,
                        modifiers: to_egui_modifiers(event.modifiers),
                    }),
            })
            .flatten()
            .collect::<Vec<_>>();

        let mouse_pos = input.mouse_position()
            .map(|p| egui::math::Pos2::new(p.x / scale_factor, p.y / scale_factor))
            .unwrap_or_else(|| egui::math::Pos2::new(0.0, 0.0));

        let dotrix_to_egui = [
            (Button::MouseLeft, egui::PointerButton::Primary),
            (Button::MouseRight, egui::PointerButton::Secondary),
            (Button::MouseMiddle, egui::PointerButton::Middle),
        ];

        for &(dotrix_button, egui_button) in dotrix_to_egui.iter() {
            if let Some(button_state) = input.button_state(dotrix_button) {
                events.push(egui::Event::PointerButton {
                    pos: mouse_pos,
                    button: egui_button,
                    pressed: button_state == InputState::Hold,
                    modifiers: Default::default(),
                });
            }
        }

        if input.mouse_moved() {
            events.push(egui::Event::PointerMoved(
                Pos2{
                    x: mouse_pos.x,
                    y: mouse_pos.y,
                }
            ));
        }

        let dropped_files = input.dropped_files.take()
            .map(|dropped_files| dropped_files.into_iter()
                .map(|path| DroppedFile {
                    path: Some(path),
                    ..Default::default()
                })
                .collect::<Vec<_>>()
            )
            .unwrap_or_default();

        let hovered_files = input.hovered_files.iter().map(|path| {
            HoveredFile{
                path: Some(path.to_owned()),
                ..Default::default()
            }
        }).collect::<Vec<_>>();

        self.ctx.begin_frame(egui::RawInput {
            screen_rect: Some(egui::math::Rect::from_min_size(
                Default::default(),
                egui::math::vec2(surface_width as f32, surface_height as f32) / scale_factor
            )),
            pixels_per_point: Some(scale_factor),
            events,
            dropped_files,
            hovered_files,
            scroll_delta: egui::math::vec2(0.0, input.mouse_scroll() * SCROLL_SENSITIVITY),
            ..Default::default()
        });

        let texture = self.ctx.texture();

        if let Some(texture_id) = self.texture {
            if self.texture_version != Some(texture.version) {
                let mut asset_texture = assets.get_mut::<Texture>(texture_id)
                    .expect("EGUI texture must be loaded already");

                asset_texture.data = egui_texture_to_rgba(&texture);
                asset_texture.width = texture.width as u32;
                asset_texture.height = texture.height as u32;
                asset_texture.unload();
            } else {
                return;
            }
        } else {
            self.texture = Some(assets.store_as(Texture {
                width: texture.width as u32,
                height: texture.height as u32,
                data: egui_texture_to_rgba(&texture),
                depth: 1,
                changed: true,
                ..Default::default()
            }, TEXTURE_NAME));
        }
        self.texture_version = Some(texture.version);
    }

    fn tessellate(&mut self, window: &Window) -> &mut [(Widget, Pipeline)] {
        let scale_factor = self.scale_factor * window.scale_factor();
        let surface_size = window.inner_size();
        let surface_width = surface_size.x as f32;
        let surface_height = surface_size.y as f32;

        let (_output, paint_commands) = self.ctx.end_frame();
        let paint_jobs = self.ctx.tessellate(paint_commands);

        let physical_width = surface_width * scale_factor;
        let physical_height = surface_height * scale_factor;

        self.widgets = paint_jobs.into_iter().map(|egui::ClippedMesh(clip_rect, egui_mesh)| {
            // Transform clip rect to physical pixels.
            let clip_min_x = scale_factor * clip_rect.min.x;
            let clip_min_y = scale_factor * clip_rect.min.y;
            let clip_max_x = scale_factor * clip_rect.max.x;
            let clip_max_y = scale_factor * clip_rect.max.y;

            // Make sure clip rect can fit within an `u32`.
            let clip_min_x = clip_min_x.clamp(0.0, physical_width);
            let clip_min_y = clip_min_y.clamp(0.0, physical_height);
            let clip_max_x = clip_max_x.clamp(clip_min_x, physical_width);
            let clip_max_y = clip_max_y.clamp(clip_min_y, physical_height);

            let clip_min_x = clip_min_x.round() as u32;
            let clip_min_y = clip_min_y.round() as u32;
            let clip_max_x = clip_max_x.round() as u32;
            let clip_max_y = clip_max_y.round() as u32;

            let width = (clip_max_x - clip_min_x).max(1);
            let height = (clip_max_y - clip_min_y).max(1);

            let vertices_count = egui_mesh.vertices.len();
            let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count);
            let mut colors: Vec<[f32; 4]> = Vec::with_capacity(vertices_count);

            let positions = egui_mesh.vertices.iter().map(|v| {
                uvs.push([v.uv.x, v.uv.y]);
                colors.push([
                    v.color.r() as f32,
                    v.color.g() as f32,
                    v.color.b() as f32,
                    v.color.a() as f32 / 255.0,
                ]);
                [v.pos.x, v.pos.y]
            }).collect::<Vec<[f32; 2]>>();

            let texture = match egui_mesh.texture_id {
                TextureId::Egui => self.texture.expect("texture for widget should be loaded"),
                TextureId::User(id) => Id::new(id),
            };

            let mut mesh = Mesh::default();
            mesh.with_vertices(&positions);
            mesh.with_vertices(&uvs);
            mesh.with_vertices(&colors);

            if !egui_mesh.indices.is_empty() {
                mesh.with_indices(&egui_mesh.indices);
            }

            (
                Widget {
                    mesh,
                    texture,
                },
                Pipeline::default()
                    .with_scissors_rect(
                        clip_min_x,
                        clip_min_y,
                        width,
                        height,
                    )
            )
        }).collect();
        self.widgets.as_mut_slice()
    }
}

fn egui_texture_to_rgba(texture: &egui::Texture) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 * texture.pixels.len());

    for &alpha in texture.pixels.iter() {
        data.extend(egui::paint::color::Color32::from_white_alpha(alpha).to_array().iter());
    }
    data
}

/// Translates winit to egui keycodes.
#[inline]
fn to_egui_key_code(key: KeyCode) -> Option<egui::Key> {
    Some(match key {
        KeyCode::Escape => Key::Escape,
        KeyCode::Insert => Key::Insert,
        KeyCode::Home => Key::Home,
        KeyCode::Delete => Key::Delete,
        KeyCode::End => Key::End,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::Left => Key::ArrowLeft,
        KeyCode::Up => Key::ArrowUp,
        KeyCode::Right => Key::ArrowRight,
        KeyCode::Down => Key::ArrowDown,
        KeyCode::Back => Key::Backspace,
        KeyCode::Return => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Space => Key::Space,

        KeyCode::A => Key::A,
        KeyCode::K => Key::K,
        KeyCode::U => Key::U,
        KeyCode::W => Key::W,
        KeyCode::Z => Key::Z,

        _ => {
            return None;
        }
    })
}

/// Translates winit to egui modifier keys.
#[inline]
fn to_egui_modifiers(modifiers: Modifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        #[cfg(target_os = "macos")]
        mac_cmd: modifiers.logo(),
        #[cfg(target_os = "macos")]
        command: modifiers.logo(),
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        #[cfg(not(target_os = "macos"))]
        command: modifiers.ctrl(),
    }
}

/// EGUI overlay startup system
pub fn startup(mut overlay: Mut<Overlay>) {
    overlay.set(Egui::default());
}

/// Enables EGUI extension into Dotrix application
pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
}
