struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct Meta {
    /// x: number_of_lights, y: shadows_enabled, z,w: reserved
    config: vec4<u32>,
}

struct Camera {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
}

struct Transform {
    model: mat4x4<f32>,
}

struct Instance {
    idx: vec4<u32>, // x - transform, y - material, z/w - reserved
}

struct Material {
    /// Albedo color RGBA
    color: vec4<f32>,
    /// x: ambient_occlusion, y:metallic, z:roughness, w:reserve
    options: vec4<f32>,
    /// x: ambient_occlusion, y:metallic, z:normal, w:roughness
    maps_1: vec4<u32>,
    /// x: color map, yzw: reserved
    maps_2: vec4<u32>,
}

struct Light {
    /// rgb, a: intensity
    color: vec4<f32>,
    /// x: enabled, y: mode, z: shadow, w: reserved
    options: vec4<u32>,
    /// Mode::Ambient -> _
    /// Mode::Directional -> xyz: direction
    /// Mode::Simple -> xyz: position
    /// Mode::Point -> xyz: position
    /// Mode::Spot -> xyz: position, w: cut_off
    mode_options_1: vec4<f32>,
    /// Mode::Ambient -> _
    /// Mode::Directional -> _
    /// Mode::Simple -> _
    /// Mode::Point -> x: constant, y: linear, z: quadratic
    /// Mode::Spot -> xyz: direction, w: outer_cut_off
    mode_options_2: vec4<f32>
}

@group(0)
@binding(0)
var<uniform> u_meta: Meta;

@group(0)
@binding(1)
var<uniform> u_camera: Camera;

@group(0)
@binding(2)
var<storage> s_instances: array<Instance>;

@group(0)
@binding(3)
var<storage> s_transform: array<Transform>;

@group(0)
@binding(4)
var<storage> s_materials: array<Material>;

@group(0)
@binding(5)
var<storage> s_light: array<Light>;

@vertex
fn vs_main_solid(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texUV: vec2<f32>,
    @builtin(instance_index) instance_id: u32,
) -> VertexOutput {
    var result: VertexOutput;

    let idx: vec4<u32> = s_instances[instance_id].idx;
    let model = s_transform[idx.x].model;
    let material = s_materials[idx.y]; 

    let world_position = model * vec4<f32>(position, 1.0);
    let world_normal = mat3x3<f32>(model.x.xyz, model.y.xyz, model.z.xyz) * normal;

    result.color = material.color;
    result.position = u_camera.proj * u_camera.view * world_position;
    result.world_position = world_position;
    result.world_normal = world_normal;

    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);
    let number_of_lights = u_meta.config.x;
    let normal = normalize(vertex.world_normal);
    
    for (var i = 0u; i < number_of_lights; i +=1u) {
        let light_color = vec3<f32>(s_light[i].color.rgb) * s_light[i].color.a;
        let light_mode = s_light[i].options.y;
        if (light_mode == 0u) { // Ambient
            color += light_color;
        } else if (light_mode == 2u) { // Simple
            let light_position = vec3<f32>(s_light[i].mode_options_1.xyz);
            let light_direction = normalize(light_position.xyz - vertex.world_position.xyz);
            let diffuse = max(0.0, dot(normal, light_direction));
            color += diffuse * light_color;
        }
    }

    return vec4<f32>(color, 1.0) * vertex.color;
}
