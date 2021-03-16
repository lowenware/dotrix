use dotrix::{
    Dotrix, Display,
    assets::{ Id, Mesh, Wires },
    components::{ Model, WireFrame },
    input::{ Button, State as InputState },
    ecs::{ Const, Context, Mut, RunLevel, System },
    egui::{
        Egui,
        DragValue,
        Grid,
        Window
    },
    input::{ Mapper },
    math::{ Point3, Vec3 },
    renderer::{ SimpleLight, Transform },
    services::{ Assets, Camera, Renderer, Frame, Input, World },
    systems::{ overlay_update, world_renderer },
    terrain::{ MarchingCubes },
};

use noise::{ NoiseFn, Fbm, MultiFractal };
use std::f32::consts::PI;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Action {}

fn main() {
    Dotrix::application("Marching Cubes")
        .with_display(Display {
            clear_color: [0.02; 4],
            fullscreen: false,
        })
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(overlay_update))
        .with_system(System::from(ui))
        .with_system(System::from(camera_control))
        .with_system(System::from(spawn))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_service(Assets::new())
        .with_service(Frame::new())
        .with_service(Camera {
            distance: 8.0,
            y_angle: PI / 2.0,
            xz_angle: 0.1,
            target: Point3::new(1.0, 2.0, 0.0),
            ..Default::default()
        })
        .with_service(Settings::default())
        .with_service(Density::new())
        .with_service(World::new())
        .with_service(Input::new(Box::new(Mapper::<Action>::new())))
        .run();
}

fn startup(
    mut assets: Mut<Assets>,
    mut renderer: Mut<Renderer>,
    mut world: Mut<World>,
) {
    renderer.add_overlay(Box::new(Egui::default()));
    assets.store_as(Wires::cube([0.0, 1.0, 0.0]), "wires");
    assets.import("examples/marching_cubes/surface.png");

    world.spawn(Some((SimpleLight{
        position: Vec3::new(0.0, 100.0, 0.0), ..Default::default()
    },)));
}

const DENSITY_MAP_WIDTH: usize = 3;
const DENSITY_MAP_DEPTH: usize = 3;
const DENSITY_MAP_HEIGHT: usize = 5;

struct Density {
    changed: bool,
    iteration: usize,
    map: [[[f32; DENSITY_MAP_DEPTH]; DENSITY_MAP_HEIGHT]; DENSITY_MAP_WIDTH],
}

impl Density {
    fn new() -> Self {
        Self {
            map: Self::flat(),
            iteration: 0,
            changed: true
        }
    }

    fn flat() -> [[[f32; DENSITY_MAP_DEPTH]; DENSITY_MAP_HEIGHT]; DENSITY_MAP_WIDTH] {
        let mut map = [[[0.0; DENSITY_MAP_DEPTH]; DENSITY_MAP_HEIGHT]; DENSITY_MAP_WIDTH];
        for by_x in map.iter_mut().take(DENSITY_MAP_WIDTH) {
            for (y, item) in by_x.iter_mut().enumerate().take(DENSITY_MAP_HEIGHT) {
                let value = ((DENSITY_MAP_HEIGHT as i32) / 2 - y as i32) as f32;
                *item = [value; DENSITY_MAP_DEPTH];
            }
        }
        map
    }

    fn noise(iteration: usize) -> [[[f32; DENSITY_MAP_DEPTH]; DENSITY_MAP_HEIGHT]; DENSITY_MAP_WIDTH] {
        let mut map = [[[0.0; DENSITY_MAP_DEPTH]; DENSITY_MAP_HEIGHT]; DENSITY_MAP_WIDTH];
        let scale = 2.0;
        let amplitude = 4.0;
        let noise = Fbm::new();
        let noise = noise.set_octaves(8);
        let noise = noise.set_frequency(1.1);
        let noise = noise.set_lacunarity(4.5);
        let noise = noise.set_persistence(0.1);
        for (x, by_x) in map.iter_mut().enumerate().take(DENSITY_MAP_WIDTH) {
            let xf = (x + iteration) as f64 / scale;
            for (y, by_xy) in by_x.iter_mut().enumerate().take(DENSITY_MAP_HEIGHT) {
                let yf = y as f64 / scale;
                for (z, item) in by_xy.iter_mut().enumerate().take(DENSITY_MAP_DEPTH) {
                    let zf = z as f64 / scale;
                    *item = (amplitude * noise.get([xf, zf]) - yf) as f32;
                }
            }
        }
        map
    }

    fn from_flat(&mut self) {
        self.map = Self::flat();
        self.changed = true;
    }

    fn from_noise(&mut self) {
        self.map = Self::noise(self.iteration);
        self.iteration += 1;
        self.changed = true;
    }

    fn interpolate(&mut self) {
        let y = DENSITY_MAP_HEIGHT / 2;
        self.map[0][y][1] = self.map[0][y][0] + (self.map[0][y][2] - self.map[0][y][0]) / 2.0;
        self.map[2][y][1] = self.map[0][y][0] + (self.map[2][y][2] - self.map[2][y][0]) / 2.0;

        self.map[1][y][0] = self.map[0][y][0] + (self.map[2][y][0] - self.map[0][y][0]) / 2.0;
        self.map[1][y][1] = self.map[0][y][1] + (self.map[2][y][1] - self.map[0][y][1]) / 2.0;
        self.map[1][y][2] = self.map[0][y][2] + (self.map[2][y][2] - self.map[0][y][2]) / 2.0;
    }
}

pub struct Settings {
    show_top: bool,
    show_bottom: bool,
    high_lod_top: bool,
    high_lod_bottom: bool,
    zoom_speed: f32,
    rotate_speed: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_top: true,
            show_bottom: false,
            high_lod_top: false,
            high_lod_bottom: false,
            zoom_speed: 1.0,
            rotate_speed: 1.0,
        }
    }
}

fn ui(
    mut settings: Mut<Settings>,
    renderer: Const<Renderer>,
    mut density: Mut<Density>,
) {
    let high_lod = settings.high_lod_top || settings.high_lod_bottom;
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    Window::new("Density")
        .show(&egui.ctx, |ui| {

        Grid::new("Density")
            .striped(true)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {

            for y in (0..DENSITY_MAP_HEIGHT).rev() {

                if !high_lod && y % 2 != 0 {
                    continue;
                }
                ui.label(format!("Y: {}, (X;Z)", y));
                for z in 0..DENSITY_MAP_DEPTH {
                    for x in 0..DENSITY_MAP_WIDTH {
                        if high_lod || (x % 2 == 0 && z % 2 == 0) {
                            let value = density.map[x][y][z];
                            ui.add(
                                DragValue::f32(&mut density.map[x][y][z])
                                    .prefix(format!("({};{}): ", x, z))
                                    .speed(0.01)
                            );
                            if (value - density.map[x][y][z]).abs() > 0.009 {
                                density.changed = true;
                            }
                        }
                    }
                }
                ui.end_row();
            }
        });
    });

    Window::new("Controls")
        .show(&egui.ctx, |ui| {

        Grid::new("Density")
            .striped(true)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {

            ui.label("Rotation speed");
            ui.add(
                DragValue::f32(&mut settings.rotate_speed).speed(0.01)
            );
            ui.end_row();

            ui.label("Zoom speed");
            ui.add(
                DragValue::f32(&mut settings.zoom_speed).speed(0.01)
            );
            ui.end_row();

            ui.label("Top Node");
            ui.checkbox(&mut settings.show_top, "Show");
            ui.checkbox(&mut settings.high_lod_top, "High LOD");
            ui.end_row();

            ui.label("Bottom Node");
            ui.checkbox(&mut settings.show_bottom, "Show");
            ui.checkbox(&mut settings.high_lod_bottom, "High LOD");
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("Flat").clicked {
                density.from_flat();
            }
            if ui.button("Noise").clicked {
                density.from_noise();
            }
            if ui.button("Interpolate").clicked {
                density.interpolate();
            }
        });
    });

    if settings.rotate_speed < 0.01 {
        settings.rotate_speed = 0.01;
    } else if settings.rotate_speed > 100.0 {
        settings.rotate_speed = 100.0;
    }

    if settings.zoom_speed < 0.01 {
        settings.zoom_speed = 0.01;
    } else if settings.zoom_speed > 100.0 {
        settings.zoom_speed = 100.0;
    }
}

#[derive(Default)]
struct Chunk {
    mesh: Option<Id<Mesh>>,
    x: usize,
    y: usize,
    z: usize,
    lod: usize,
    enabled: bool,
}

#[derive(Default)]
struct Polygons {
    chunks: Vec<Chunk>,
}

struct Index(usize);

fn spawn(
    mut ctx: Context<Polygons>,
    mut assets: Mut<Assets>,
    mut world: Mut<World>,
    mut density: Mut<Density>,
    settings: Const<Settings>,
) {
    if ctx.chunks.is_empty() {
        // add lower lods
        for y in 0..2 {
            ctx.chunks.push(Chunk {
                lod: 1,
                x: 0,
                y: y * 2,
                z: 0,
                ..Default::default()
            });
        }
        // add higher lods
        for x in 0..(DENSITY_MAP_WIDTH - 1) {
            for y in 0..(DENSITY_MAP_HEIGHT - 1) {
                for z in 0..(DENSITY_MAP_DEPTH - 1) {
                    ctx.chunks.push(Chunk {
                        lod: 2,
                        x,
                        y,
                        z,
                        ..Default::default()
                    });
                }
            }
        }
    }

    for (i, chunk) in ctx.chunks.iter_mut().enumerate() {
        let enabled = chunk.enabled;

        chunk.enabled = true;
        if chunk.y < DENSITY_MAP_HEIGHT / 2 {
            if !settings.show_bottom || (chunk.lod == 2) != settings.high_lod_bottom {
                chunk.enabled = false;
            }
        } else if !settings.show_top || (chunk.lod == 2) != settings.high_lod_top {
            chunk.enabled = false;
        }

        if chunk.enabled == enabled && !density.changed {
            continue;
        }

        let query = world.query::<(&mut Model, &mut WireFrame, &Index)>();
        for (model, wireframe, index) in query {
            if index.0 == i {
                model.disabled = !chunk.enabled;
                wireframe.disabled = !chunk.enabled;
            }
        }

        println!("({}, {}, {}): {} -> {}", chunk.x, chunk.y, chunk.z, chunk.lod, chunk.enabled);

        let positions = polygonize(&density, chunk.lod, chunk.x, chunk.y, chunk.z);
        let len = positions.len();
        let uvs = Some(vec![[1.0, 0.0]; len]);

        if let Some(mesh_id) = chunk.mesh {
            let mesh = assets.get_mut(mesh_id).unwrap();
                mesh.positions = positions;
                mesh.uvs = uvs;
                mesh.normals.take();
                mesh.calculate();
                mesh.unload();
        } else {
            let mut mesh = Mesh {
                    positions,
                    uvs,
                    ..Default::default()
                };
            mesh.calculate();

            let mesh = assets.store(mesh);

            let texture = assets.register("surface");
            let scale = if chunk.lod == 1 { 2.0 } else { 1.0 };

            let transform = Transform {
                translate: Vec3::new(chunk.x as f32, chunk.y as f32, chunk.z as f32),
                scale: Vec3::new(scale, scale, scale),
                ..Default::default()
            };
            let wires = assets.find("wires").expect("wires to be loaded");
            let wires_transform = Transform {
                translate: Vec3::new(
                    chunk.x as f32 + scale / 2.0,
                    chunk.y as f32 + scale / 2.0,
                    chunk.z as f32 + scale / 2.0
                ),
                scale: Vec3::new(scale / 2.0, scale / 2.0, scale / 2.0),
                ..Default::default()
            };

            world.spawn(
                Some((
                    Model {
                        mesh,
                        texture,
                        transform,
                        disabled: !chunk.enabled,
                        ..Default::default()
                    },
                    WireFrame {
                        wires,
                        transform: wires_transform,
                        disabled: !chunk.enabled,
                        ..Default::default()
                    },
                    Index(i),
                ))
            );

            chunk.mesh = Some(mesh);
        }
    }
    density.changed = false;
}

fn polygonize(density: &Density, lod: usize, x0: usize, y0: usize, z0: usize) -> Vec<[f32; 3]> {

    let mc = MarchingCubes {
        size: 1,
        ..Default::default()
    };
    let scale = 2 / lod;

    let (positions, _) = mc.polygonize(|x, y, z| {
        density.map[x * scale + x0][y * scale + y0][z * scale + z0]
    });

    positions
}

pub fn camera_control(
    mut camera: Mut<Camera>,
    input: Const<Input>,
    frame: Const<Frame>,
    settings: Const<Settings>,
) {
    let rotate_speed = settings.rotate_speed * PI / 10.0;
    let zoom_speed = settings.zoom_speed * 10.0;

    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    let distance = camera.distance - zoom_speed * mouse_scroll * time_delta;
    camera.distance = if distance > -1.0 { distance } else { -1.0 };

    if input.button_state(Button::MouseRight) == Some(InputState::Hold) {
        camera.y_angle += mouse_delta.x * rotate_speed * time_delta;

        let xz_angle = camera.xz_angle + mouse_delta.y * rotate_speed * time_delta;
        let half_pi = PI / 2.0;

        camera.xz_angle = if xz_angle >= half_pi {
            half_pi - 0.01
        } else if xz_angle <= -half_pi {
            -half_pi + 0.01
        } else {
            xz_angle
        };
    }

    camera.set_view();
}
