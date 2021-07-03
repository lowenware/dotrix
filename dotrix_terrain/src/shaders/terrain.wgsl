// STAGE: VERTEX ---------------------------------------------------------------------------------

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] world_position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_uv: vec2<f32>;
};


[[block]]
struct Renderer {
    proj_view: mat4x4<f32>;
};
[[group(0), binding(0)]]
var u_renderer: Renderer;


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_uv = tex_uv;
    out.normal = normalize((vec4<f32>(normal, 1.0)).xyz);
    let world_position: vec4<f32> = vec4<f32>(position, 1.0);
    out.world_position = world_position.xyz;
    out.position = u_renderer.proj_view * world_position;
    return out;
}


// STAGE: FRAGMENT -------------------------------------------------------------------------------

let MAX_LAYERS_COUNT: u32 = 16u;

[[group(1), binding(1)]]
var r_texture: texture_2d<f32>;

[[group(0), binding(1)]]
var r_sampler: sampler;

{{ include(light) }}

struct Layer {
    color: vec4<f32>;
    height: f32;
    blend: f32;
    unused: vec2<f32>;
};

[[block]]
struct Layers {
    count: vec4<u32>;
    list: [[stride(32)]] array<Layer, MAX_LAYERS_COUNT>;
};
[[group(0), binding(3)]]
var u_layers: Layers;

fn inverse_lerp(left: f32, right: f32, value: f32) -> f32 {
    return clamp((value - left) / (right - left), 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let texture_color: vec4<f32> = textureSample(r_texture, r_sampler, in.tex_uv / 0.5);
    var albedo_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    var i: u32 = 0u;
    var count: u32 = min(u32(u_layers.count.x), MAX_LAYERS_COUNT);

    // Terrain Types
    let max_height: f32 = 300.0;
    let height_blend: f32 = 0.02;
    let epsilon: f32 = 0.0001;
    let height_percent: f32 = inverse_lerp(0.0, max_height, in.world_position.y);

    // Apply terrain layers
    loop {
        if (!(i < count)) { break; }
        let half_height_blend = u_layers.list[i].blend / 2.0;
        let color_strength = inverse_lerp(
            -half_height_blend - epsilon,
            half_height_blend,
            height_percent - u_layers.list[i].height
        );

        albedo_color = albedo_color * (1.0 - color_strength) + u_layers.list[i].color * color_strength;
        continuing { i = i + 1u; } 
    }

    // Light
    let light_color: vec4<f32> = calculate_light(in.world_position.xyz, in.normal);

    return vec4<f32>(albedo_color.rgb * texture_color.rgb * light_color.rgb, 1.0);

    //mag: f32 = length(v_TexCoord-vec2(0.5));
    // o_Target = vec4(mix(result_color.xyz, vec3(0.0), mag*mag), 1.0);
}


