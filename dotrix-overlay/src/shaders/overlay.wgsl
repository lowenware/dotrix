// STAGE: VERTEX ---------------------------------------------------------------------------------

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Overlay {
    window_size: vec2<f32>;
};
[[group(0), binding(0)]]
var<uniform> u_overlay: Overlay;

fn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(10.31475);
    let lower = srgb / vec3<f32>(3294.6);
    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] tex_uv: vec2<f32>,
    [[location(2)]] color: vec4<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_uv = tex_uv;
    out.color = vec4<f32>(linear_from_srgb(color.rgb), color.a / 255.0);
    // out.color = vec4<f32>(color.rgba / 255.0);
    out.position = vec4<f32>(
      2.0 * position.x / u_overlay.window_size.x - 1.0,
      1.0 - 2.0 * position.y / u_overlay.window_size.y,
      0.0,
      1.0
    );
    return out;
}


// STAGE: FRAGMENT -------------------------------------------------------------------------------

[[group(1), binding(1)]]
var r_sampler: sampler;

[[group(1), binding(0)]]
var r_texture: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color * textureSample(r_texture, r_sampler, in.tex_uv);
}
