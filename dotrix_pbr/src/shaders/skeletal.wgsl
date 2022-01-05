// STAGE: VERTEX ---------------------------------------------------------------------------------

let MAX_JOINTS_COUNT: u32 = 32u;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_uv: vec2<f32>;
};


struct Renderer {
    proj_view: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> u_renderer: Renderer;


struct Model {
    transform: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> u_model: Model;

struct Joints {
    transform: [[stride(64)]] array<mat4x4<f32>, MAX_JOINTS_COUNT>;
};
[[group(1), binding(6)]]
var<uniform> u_joints: Joints;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_uv: vec2<f32>,
    [[location(3)]] weights: vec4<f32>,
    [[location(4)]] joints: vec4<u32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_uv = tex_uv;

    // at the moment of writing `Mul (Scalar, Matrix)` was not supported,
    // let skin_transform: mat4x4<f32> =
    //    weights.x * u_joints.transform[joints.x] +
    //    weights.y * u_joints.transform[joints.y] +
    //    weights.z * u_joints.transform[joints.z] +
    //    weights.w * u_joints.transform[joints.w];
    let skin_transform: mat4x4<f32> = mat4x4<f32>(
        weights.x * u_joints.transform[joints.x].x +
        weights.y * u_joints.transform[joints.y].x +
        weights.z * u_joints.transform[joints.z].x +
        weights.w * u_joints.transform[joints.w].x,

        weights.x * u_joints.transform[joints.x].y +
        weights.y * u_joints.transform[joints.y].y +
        weights.z * u_joints.transform[joints.z].y +
        weights.w * u_joints.transform[joints.w].y,

        weights.x * u_joints.transform[joints.x].z +
        weights.y * u_joints.transform[joints.y].z +
        weights.z * u_joints.transform[joints.z].z +
        weights.w * u_joints.transform[joints.w].z,

        weights.x * u_joints.transform[joints.x].w +
        weights.y * u_joints.transform[joints.y].w +
        weights.z * u_joints.transform[joints.z].w +
        weights.w * u_joints.transform[joints.w].w
    );

    out.normal = normalize(
        mat3x3<f32>(
            skin_transform.x.xyz,
            skin_transform.y.xyz,
            skin_transform.z.xyz
        ) * mat3x3<f32>(
            u_model.transform.x.xyz,
            u_model.transform.y.xyz,
            u_model.transform.z.xyz
        ) * normal
    );

    let pos = u_model.transform * skin_transform * vec4<f32>(position, 1.0);
    out.world_position = pos;
    out.position = u_renderer.proj_view * pos;
    return out;
}


// STAGE: FRAGMENT -------------------------------------------------------------------------------
struct Material {
    albedo: vec4<f32>;
    has_texture: u32;
    roughness: f32;
    metallic: f32;
    ao: f32;
};
[[group(1), binding(1)]]
var<uniform> u_material: Material;

[[group(1), binding(2)]]
var r_texture: texture_2d<f32>;

[[group(1), binding(3)]]
var r_roughness_texture: texture_2d<f32>;

[[group(1), binding(4)]]
var r_metallic_texture: texture_2d<f32>;

[[group(1), binding(5)]]
var r_ao_texture: texture_2d<f32>;

[[group(0), binding(1)]]
var r_sampler: sampler;

{{ include(light) }}

fn average(input: vec4<f32>) -> f32 {
  return (input.x + input.y + input.z + input.w) / 4.;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var albedo: vec4<f32>;
    var roughness: f32;
    var metallic: f32;
    var ao: f32;

    if ((u_material.has_texture & 1u) == 1u) {
        albedo = textureSample(r_texture, r_sampler, in.tex_uv);
        // Covert from sRGB to linear color space
        // (PBR based renderer expect linear)
        albedo = vec4<f32>(pow(albedo.rgb, vec3<f32>(2.2)), albedo.a);
    } else {
        albedo = u_material.albedo;
    }

    if ((u_material.has_texture & 2u) == 2u) {
        roughness = average(textureSample(r_roughness_texture, r_sampler, in.tex_uv));
    } else {
        roughness = u_material.roughness;
    }

    if ((u_material.has_texture & 4u) == 4u) {
        metallic = average(textureSample(r_metallic_texture, r_sampler, in.tex_uv));
    } else {
        metallic = u_material.metallic;
    }

    if ((u_material.has_texture & 8u) == 8u) {
        ao = average(textureSample(r_ao_texture, r_sampler, in.tex_uv));
    } else {
        ao = u_material.ao;
    }

    return calculate_lighting(
        in.world_position.xyz,
        in.normal.xyz,
        albedo.rgb,
        roughness,
        metallic,
        ao,
    );
}
