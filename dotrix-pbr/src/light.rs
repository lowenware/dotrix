//! Various implementations of light sources
use dotrix_core::ecs::{Const, Mut};
use dotrix_core::renderer::{Buffer, Pipeline, Texture};
use dotrix_core::{Camera, Color, Globals, Renderer, World};

use dotrix_math::{Deg, EuclideanSpace, Mat4, PerspectiveFov, Point3, Vec3};

const MAX_LIGHTS: usize = 10;

#[rustfmt::skip]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

/// Light component of different types and settings
pub enum Light {
    Ambient {
        /// Light source color
        color: Color,
        /// Light source intensity
        intensity: f32,
    },
    Directional {
        /// Light source color
        color: Color,
        /// Light source direction
        direction: Vec3,
        /// Light source intensity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
    },
    Simple {
        /// Light color
        color: Color,
        /// Light source position
        position: Vec3,
        /// Light intensity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
    },
    Point {
        /// Light color
        color: Color,
        /// Light source position
        position: Vec3,
        /// Light source intencity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
        /// Constant light
        constant: f32,
        /// Linear light
        linear: f32,
        /// Quadratic light
        quadratic: f32,
    },
    Spot {
        /// Light source color
        color: Color,
        /// Light source position
        position: Vec3,
        /// Light source direction
        direction: Vec3,
        /// Light source intensity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
        /// Light source cut off
        cut_off: f32,
        /// Light source outer cut off
        outer_cut_off: f32,
    },
}

impl Light {
    pub fn ambient() -> Self {
        Light::Ambient {
            color: Color::white(),
            intensity: 0.8,
        }
    }

    pub fn directional() -> Self {
        Light::Directional {
            enabled: true,
            direction: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
        }
    }

    pub fn simple() -> Self {
        Light::Simple {
            enabled: true,
            position: Vec3::new(0.0, 1000.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
        }
    }

    pub fn point() -> Self {
        Light::Point {
            enabled: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
            constant: 1.0,
            linear: 0.35,
            quadratic: 0.44,
        }
    }

    pub fn spot() -> Self {
        Light::Spot {
            enabled: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
            cut_off: 0.8,
            outer_cut_off: 0.65,
        }
    }
}

/// Lights global uniform controller
pub struct Lights {
    pub uniform: Buffer,

    pub proj_view: Buffer,
    pub shadows: Option<Texture>,
}

impl Lights {
    /// Integrates light support into shader
    /// The `source` shader code must contain `{{ include(light) }}` label and then
    /// `let light_color = calculate_light(world_position, normal);` can be called
    pub fn add_to_shader(source: &str, bind_group: usize, binding: usize) -> String {
        let bind_group = format!("{:?}", bind_group);
        let binding = format!("{:?}", binding);
        let lights_count = format!("{:?}u", MAX_LIGHTS);

        let light_code = include_str!("shaders/light.inc.wgsl");

        let light_code = str::replace(light_code, "{{ max_lights_count }}", &lights_count)
            .replace("{{ bind_group }}", &bind_group)
            .replace("{{ binding }}", &binding);

        source.replace("{{ include(light) }}", &light_code)
    }

    /// Enable shadows
    pub fn enable_shadows(&mut self) {
        self.shadows = Some(Texture::new_array("shadows").use_as_attachment());
    }

    /// Disable shadows
    pub fn disable_shadows(&mut self) {
        self.shadows = None;
    }
}

impl Default for Lights {
    fn default() -> Self {
        Self {
            uniform: Buffer::uniform("Lights Buffer"),
            proj_view: Buffer::uniform("ProjView Buffer"),
            shadows: None,
        }
    }
}

/// Shadow Component
#[derive(Debug, Default)]
pub struct Shadow {
    pub pipeline: Pipeline,
}

/// Lights startup system
pub fn startup(mut globals: Mut<Globals>) {
    globals.set(Lights::default());
}

/// Lights loading system
pub fn load(
    world: Const<World>,
    renderer: Const<Renderer>,
    mut globals: Mut<Globals>,
    camera: Const<Camera>,
) {
    if let Some(lights) = globals.get_mut::<Lights>() {
        let camera_position = camera.position();
        let mut uniform = Uniform {
            camera_position: [camera_position.x, camera_position.y, camera_position.z, 0.],
            ..Default::default()
        };

        let mut i = 0;
        for (light,) in world.query::<(&Light,)>() {
            uniform.store(light, i);
            i += 1;
        }

        renderer.load_buffer(&mut lights.uniform, bytemuck::cast_slice(&[uniform]));

        if let Some(shadows) = lights.shadows.as_mut() {
            let lights_count =
                uniform.count[0] + uniform.count[1] + uniform.count[2] + uniform.count[3];
            let shadow_size = 512;
            if shadows.count_layers() != lights_count {
                renderer.init_texture(shadows, shadow_size, shadow_size, Some(lights_count));
            }
        }
    }
}

/// Uniform structure for lights representation in shader
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Uniform {
    /// Camera position
    camera_position: [f32; 4],
    /// Light color
    ambient: [f32; 4],
    /// Slice with numbers of light sources
    count: [u32; 4],
    /// Directional lights
    directional: [DirectionalLight; MAX_LIGHTS],
    /// Point lights
    point: [PointLight; MAX_LIGHTS],
    /// Simple lights
    simple: [SimpleLight; MAX_LIGHTS],
    /// Spot lights
    spot: [SpotLight; MAX_LIGHTS],
    /// ProjView matrix from source of light position
    proj_views: [[[f32; 4]; 4]; 4 * MAX_LIGHTS],
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            camera_position: [0.0; 4],
            ambient: [0.0; 4],
            count: [0; 4],
            directional: [DirectionalLight::default(); MAX_LIGHTS],
            point: [PointLight::default(); MAX_LIGHTS],
            simple: [SimpleLight::default(); MAX_LIGHTS],
            spot: [SpotLight::default(); MAX_LIGHTS],
            proj_views: [[[0.0; 4]; 4]; 4 * MAX_LIGHTS],
        }
    }
}

unsafe impl bytemuck::Zeroable for Uniform {}
unsafe impl bytemuck::Pod for Uniform {}

impl Uniform {
    /// Stores data from Light component into the uniform structure
    pub fn store(&mut self, light: &Light, index: usize) {
        match light {
            Light::Ambient { color, intensity } => self.ambient = (*color * (*intensity)).into(),
            Light::Directional {
                color,
                direction,
                intensity,
                enabled,
            } => {
                let i = self.count[0] as usize;
                if *enabled && i < MAX_LIGHTS {
                    self.directional[i] = DirectionalLight {
                        direction: [direction.x, direction.y, direction.z, 1.0],
                        color: (*color * (*intensity)).into(),
                    };
                    self.count[0] = i as u32 + 1;
                }
                let pos = -100.0 * direction; // TODO: rework
                self.proj_views[index] = Self::get_mx(&pos);
            }
            Light::Point {
                color,
                position,
                intensity,
                enabled,
                constant,
                linear,
                quadratic,
            } => {
                let i = self.count[1] as usize;
                if *enabled && i < MAX_LIGHTS {
                    self.point[i] = PointLight {
                        position: [position.x, position.y, position.z, 1.0],
                        color: (*color * (*intensity)).into(),
                        a_constant: *constant,
                        a_linear: *linear,
                        a_quadratic: *quadratic,
                        ..Default::default()
                    };
                    self.count[1] = i as u32 + 1;
                }
                self.proj_views[index] = Self::get_mx(position);
            }
            Light::Simple {
                color,
                position,
                intensity,
                enabled,
            } => {
                let i = self.count[2] as usize;
                if *enabled && i < MAX_LIGHTS {
                    self.simple[i] = SimpleLight {
                        position: [position.x, position.y, position.z, 1.0],
                        color: (*color * (*intensity)).into(),
                    };
                    self.count[2] = i as u32 + 1;
                }
                self.proj_views[index] = Self::get_mx(position);
            }
            Light::Spot {
                color,
                position,
                direction,
                intensity,
                enabled,
                cut_off,
                outer_cut_off,
            } => {
                let i = self.count[3] as usize;
                if *enabled && i < MAX_LIGHTS {
                    self.spot[i] = SpotLight {
                        position: [position.x, position.y, position.z, 1.0],
                        direction: [direction.x, direction.y, direction.z, 1.0],
                        color: (*color * (*intensity)).into(),
                        cut_off: *cut_off,
                        outer_cut_off: *outer_cut_off,
                        ..Default::default()
                    };
                    self.count[3] = i as u32 + 1;
                }
                self.proj_views[index] = Self::get_mx(position);
            }
        };
    }

    fn get_mx(position: &Vec3) -> [[f32; 4]; 4] {
        let fov = 60.0; // NOTE: should be light specific
        let depth = 1.0..100.0;

        let mx_view = Mat4::look_at_rh(
            Point3::new(position.x, position.y, position.z),
            Point3::origin(),
            Vec3::unit_z(),
        );
        let projection = PerspectiveFov {
            fovy: Deg(fov).into(),
            aspect: 1.0,
            near: depth.start,
            far: depth.end,
        };
        let mx_correction = OPENGL_TO_WGPU_MATRIX;
        (mx_correction * Mat4::from(projection.to_perspective()) * mx_view).into()
    }
}

/// Directional light uniform data
#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct DirectionalLight {
    /// Light direction
    direction: [f32; 4],
    /// Light color
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for DirectionalLight {}
unsafe impl bytemuck::Pod for DirectionalLight {}

/// Point light uniform data
#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct PointLight {
    /// Light source position
    position: [f32; 4],
    /// Light color
    color: [f32; 4],
    /// Constant light attenuation
    a_constant: f32,
    /// Linear light attenuation
    a_linear: f32,
    /// Quadratic light attenuation
    a_quadratic: f32,
    a_unused: f32,
}

unsafe impl bytemuck::Zeroable for PointLight {}
unsafe impl bytemuck::Pod for PointLight {}

/// Simple light uniform data
#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct SimpleLight {
    /// Light source position
    position: [f32; 4],
    /// Light color
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for SimpleLight {}
unsafe impl bytemuck::Pod for SimpleLight {}

/// Spot Light uniform data
#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct SpotLight {
    /// Light source position
    position: [f32; 4],
    /// Light source direction
    direction: [f32; 4],
    /// Light source color
    color: [f32; 4],
    /// Light source cut off
    cut_off: f32,
    /// Light source outer cut off
    outer_cut_off: f32,
    /// structure padding
    padding: [f32; 2],
}

unsafe impl bytemuck::Zeroable for SpotLight {}
unsafe impl bytemuck::Pod for SpotLight {}
