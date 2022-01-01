// STAGE: VERTEX ---------------------------------------------------------------------------------
struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] local_position: vec3<f32>;
};

struct Renderer {
    proj_view: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> u_renderer: Renderer;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.local_position = position;
    out.position = u_renderer.proj_view * vec4<f32>(position, 1.0);
    return out;
}

// STAGE: FRAGMENT -------------------------------------------------------------------------------
struct Gradient {
  zenith_color: vec4<f32>;
  nadir_color: vec4<f32>;
};
[[group(0), binding(1)]]
var<uniform> u_skybox: Gradient;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let normalized_direction: vec3<f32> = normalize(in.local_position);
    let nadir_color: vec3<f32> =  u_skybox.nadir_color.xyz;

    let zenith_color: vec3<f32> = u_skybox.zenith_color.xyz;

    let mix_amount: vec3<f32> = vec3<f32>(smoothStep(-1., 1., normalized_direction.y));
    let horizon_color: vec3<f32> = mix(nadir_color, zenith_color, mix_amount);

    return vec4<f32>(horizon_color , 1.0);
}
