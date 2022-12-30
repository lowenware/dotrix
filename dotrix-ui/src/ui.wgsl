struct VertexOutput {
    @location(0) tex_uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Overlay {
    screen_width: f32,
    screen_height: f32,
    max_radius: f32,
    _padding: u32,
};
@group(0) @binding(0) var<uniform> u_overlay: Overlay;

fn color_from_u32(value: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((value >> 24u) & 255u),
        f32((value >> 16u) & 255u),
        f32((value >> 8u) & 255u),
        f32(value & 255u),
    ) / 255.0;
}

fn position_from_screen(position: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(
        2.0 * position.x / u_overlay.screen_width - 1.0,
        1.0 - 2.0 * position.y / u_overlay.screen_height,
        0.0,
        1.0,
    );
}

@vertex
fn vs_main(
    @builtin(vertex_index) my_index: u32,
    @location(0) a_position: vec2<f32>,
    @location(1) a_tex_uv: vec2<f32>,
    @location(2) a_color: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_uv = a_tex_uv;
    out.color = color_from_u32(a_color);
    /*
    var z: f32 = 0.0;
    if my_index > 6u {
        z = 1.0;
        out.color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } else {
        out.color = vec4<f32>(1.0, 0.0, 1.0, 0.0);
        
    }
    */
    out.position = position_from_screen(a_position);
    return out;
}

// @group(1) @binding(0) var t_texture: texture_2d<f32>;
// @group(1) @binding(1) var s_sampler: sampler;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    var color = vec4<f32>(in.color.rgba);

    //if in.position.x > 380.0  && in.position.x < 440.0 {
    //    color = vec4<f32>(in.color.rgb, 0.3);
        // return vec4<f32>(in.color.rga, 0.3);
    //} 
    // We always have an sRGB aware texture at the moment.
    // let tex_linear = textureSample(r_tex_color, r_tex_sampler, in.tex_coord);
    // let tex_gamma = gamma_from_linear_rgba(tex_linear);
    // let out_color_gamma = in.color * tex_gamma;
    // return vec4<f32>(in.color.rgb, 1.0);
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    return color;
}
