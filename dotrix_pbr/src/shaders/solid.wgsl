// STAGE: VERTEX ---------------------------------------------------------------------------------

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] world_position: vec3<f32>;
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


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_uv = tex_uv;
    out.normal = normalize(mat3x3<f32>(
        u_model.transform.x.xyz,
        u_model.transform.y.xyz,
        u_model.transform.z.xyz,
    ) * normal);
    var pos: vec3<f32> = (u_model.transform * vec4<f32>(position, 1.0)).xyz;
    out.world_position = pos;
    out.position = u_renderer.proj_view * vec4<f32>(pos, 1.0);
    return out;
}


// STAGE: FRAGMENT -------------------------------------------------------------------------------
struct Material {
    albedo: vec4<f32>;
    has_texture: u32;
};
[[group(1), binding(1)]]
var<uniform> u_material: Material;

[[group(1), binding(2)]]
var r_texture: texture_2d<f32>;

[[group(0), binding(1)]]
var r_sampler: sampler;

{{ include(light) }}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var albedo_color: vec4<f32>;

    if (u_material.has_texture != 0u) {
        albedo_color = textureSample(r_texture, r_sampler, in.tex_uv);
    } else {
        albedo_color = u_material.albedo;
    }

    let light_color = calculate_light(in.world_position, in.normal);

    return albedo_color * light_color;
}
