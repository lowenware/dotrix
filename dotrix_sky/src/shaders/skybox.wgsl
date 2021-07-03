// STAGE: VERTEX ---------------------------------------------------------------------------------

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] cube: vec3<f32>;
};

[[block]]
struct SkyBox {
    proj_view: mat4x4<f32>;
    scale: mat4x4<f32>;
};
[[group(0), binding(0)]]
var u_skybox: SkyBox;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.cube = position;
    out.position = u_skybox.proj_view * u_skybox.scale * vec4<f32>(position, 1.0);
    return out;
}


// STAGE: FRAGMENT -------------------------------------------------------------------------------

[[group(0), binding(1)]]
var r_sampler: sampler;
[[group(1), binding(0)]]
var r_texture: texture_cube<f32>;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let uv = vec3<f32>(in.cube.x, in.cube.yz);
    let texture_color = textureSample(r_texture, r_sampler, uv).rgb;
    return vec4<f32>(texture_color, 1.0);
}
