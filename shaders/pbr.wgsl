struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct Global {
    /// x: number_of_lights, y: shadows_enabled, z,w: reserved
    config: vec4<u32>,
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    light_proj_view: mat4x4<f32>,
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
    /// ProjView matrix
    proj_view: mat4x4<f32>,
    /// rgb, a:unused
    color: vec4<f32>,
    /// xyz, if w < 1.0 { pos } else { dir } 
    pos_dir: vec4<f32>,
    /// xyz, w:unused
    stream: vec4<f32>,
    /// x:constant, y:linear, z:quadratic, w:unused
    blur: vec4<f32>,
    /// x:cut_off_inner, y:outer_cut_off, zw: unused
    cut_off: vec4<f32>,
    /// x:shadow, yzw: unused
    options: vec4<u32>,
}

@group(0)
@binding(0)
var<uniform> u_global: Global;

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

@group(1)
@binding(0)
var t_shadow: texture_depth_2d_array;
@group(1)
@binding(1)
var sampler_shadow: sampler_comparison;

@vertex
fn vs_main_shadows(
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_id: u32,
) -> @builtin(position) vec4<f32> {
    let idx: vec4<u32> = s_instances[instance_id].idx;
    let model = s_transform[idx.x].model;

    return u_global.light_proj_view * model * vec4<f32>(position, 1.0);
}

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
    result.position = u_global.proj * u_global.view * world_position;
    result.world_position = world_position;
    result.world_normal = world_normal;

    return result;
}

fn fetch_shadow(light_idx: u32, homogeneous_coords: vec4<f32>) -> f32 {
    // let pcf_count: i32 = 2;
    // let total_texels: f32 = (pcf_count * 2.0 + 1.0) * (pcf_count * 2.0 + 1.0);
    // let map_size: f32 = 512.0;
    // let texel_size = 1.0 / map_size;
    // let total = 0.0;

    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }
    // compensate for the Y-flip difference between the NDC and texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    
    // do the lookup, using HW PCF and comparison
    //for (var u = -pcf_count; u <= pcf_count; u += 1u) {
    //    for (var v = -pcf_count; v <= pcf_count; v += 1u) {
    //
    //    }
    //}
    

    return textureSampleCompareLevel(t_shadow, sampler_shadow, light_local, i32(light_idx), homogeneous_coords.z * proj_correction);
}


@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);
    let number_of_lights = u_global.config.x;
    let normal = normalize(vertex.world_normal);
    var shadow_idx: u32 = 0u;
    
    for (var i = 0u; i < number_of_lights; i +=1u) {
        let light_color = vec3<f32>(s_light[i].color.rgb);
        let pos_dir = vec4<f32>(s_light[i].pos_dir);
        var shadow: f32 = 1.0;

        var light_direction: vec3<f32>;
        if (pos_dir.w < 1.0) {
            light_direction = vec3<f32>(-pos_dir.xyz);
        } else {
            light_direction = normalize(pos_dir.xyz - vertex.world_position.xyz);
        }

        let drop_shadow: u32 = s_light[i].options.x;

        if (drop_shadow == 1u) {
            shadow = fetch_shadow(shadow_idx, s_light[i].proj_view * vertex.world_position);
            shadow_idx += 1u;
        }

        let diffuse = max(0.0, dot(normal, light_direction));
        color += shadow * diffuse * light_color;
    }

    return vec4<f32>(color, 1.0) * vertex.color;
}
