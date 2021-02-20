use dotrix_core::{
    assets::{ Id, Texture },
    input::{
        Button,
        Event as InputEvent,
        KeyCode,
        Modifiers,
        State as InputState
    },
    renderer::{ WidgetVertex, OverlayProvider, Widget },
    services::{ Assets, Input },
};

pub use egui::*;

const TEXTURE_NAME: &str = "egui::texture";

#[derive(Default)]
pub struct Egui {
    pub ctx: egui::CtxRef,
    texture: Option<Id<Texture>>,
    texture_version: Option<u64>,
}

impl OverlayProvider for Egui {

    fn feed(
        &mut self,
        assets: &mut Assets,
        input: &Input,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    ) {
        let events = input.events.iter()
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
                .filter(|e| e.is_some())
                .map(|e| e.unwrap())
                .collect::<Vec<_>>();

        self.ctx.begin_frame(egui::RawInput {
            mouse_down: input.button_state(Button::MouseLeft)
                .map(|state| state == InputState::Hold)
                .unwrap_or(false),
            mouse_pos: input.mouse_position()
                .map(|p| egui::math::Pos2::new(p.x / scale_factor, p.y / scale_factor)),
            screen_rect: Some(egui::math::Rect::from_min_size(
                Default::default(),
                egui::math::vec2(surface_width as f32, surface_height as f32) / scale_factor
            )),
            pixels_per_point: Some(scale_factor),
            events,
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
                ..Default::default()
            }, TEXTURE_NAME));
        }
        self.texture_version = Some(texture.version);
    }

    fn tessellate(
        &self,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    ) -> Vec<Widget> {
        let (_output, paint_commands) = self.ctx.end_frame();
        let paint_jobs = self.ctx.tessellate(paint_commands);
        let physical_width = surface_width * scale_factor;
        let physical_height = surface_height * scale_factor;

        paint_jobs.into_iter().map(|(clip_rect, triangles)| {
            // Transform clip rect to physical pixels.
            let clip_min_x = scale_factor * clip_rect.min.x;
            let clip_min_y = scale_factor * clip_rect.min.y;
            let clip_max_x = scale_factor * clip_rect.max.x;
            let clip_max_y = scale_factor * clip_rect.max.y;

            // Make sure clip rect can fit within an `u32`.
            let clip_min_x = egui::clamp(clip_min_x, 0.0..=physical_width);
            let clip_min_y = egui::clamp(clip_min_y, 0.0..=physical_height);
            let clip_max_x = egui::clamp(clip_max_x, clip_min_x..=physical_width);
            let clip_max_y = egui::clamp(clip_max_y, clip_min_y..=physical_height);

            let clip_min_x = clip_min_x.round() as u32;
            let clip_min_y = clip_min_y.round() as u32;
            let clip_max_x = clip_max_x.round() as u32;
            let clip_max_y = clip_max_y.round() as u32;

            let width = (clip_max_x - clip_min_x).max(1);
            let height = (clip_max_y - clip_min_y).max(1);

            let vertices = triangles.vertices.iter()
                .map(|v| {
                    WidgetVertex {
                        position: [v.pos.x, v.pos.y],
                        uv: [v.uv.x, v.uv.y],
                        color: [
                            v.color.r() as f32,
                            v.color.g() as f32,
                            v.color.b() as f32,
                            v.color.a() as f32 / 255.0,
                        ],
                    }
                })
                .collect::<Vec<_>>();

            Widget {
                vertices,
                indices: Some(triangles.indices),
                texture: self.texture.expect("texture for widget should be loaded"),
                clip_min_x, clip_min_y, width, height,
                ..Default::default()
            }
        }).collect()
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
