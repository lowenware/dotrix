// STAGE: VERTEX ---------------------------------------------------------------------------------

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] tex_uv: vec2<f32>;
};


[[block]]
struct Renderer {
    proj_view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var u_renderer: Renderer;


[[block]]
struct Model {
    transform: mat4x4<f32>;
};

[[group(0), binding(1)]]
var u_model: Model;


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_uv = tex_uv;
    out.normal = normalize((u_model.transform * vec4<f32>(normal, 1.0)).xyz);
    out.position = u_renderer.proj_view * u_model.transform * vec4<f32>(position, 1.0);
    return out;
}


// STAGE: FRAGMENT -------------------------------------------------------------------------------

let MAX_LIGHTS_COUNT: u32 = 10u;

struct DirectionalLight {
    direction: vec4<f32>;
    color: vec4<f32>;
};

struct PointLight {
    position: vec4<f32>;
    color: vec4<f32>;

    // attenuation
    a_constant: f32;
    a_linear: f32;
    a_quadratic: f32;
    unused: f32;
};

struct SimpleLight {
    position: vec4<f32>;
    color: vec4<f32>;
};

struct SpotLight {
    position: vec4<f32>;
    direction: vec4<f32>;
    color: vec4<f32>;

    cut_off: f32;
    outer_cut_off: f32;
    unused: vec2<f32>;
};

[[block]]
struct Light {
    ambient: vec4<f32>;
    count: vec4<u32>;
    directional: [[stride(32)]] array<DirectionalLight, 10u>;
    point: [[stride(48)]] array<PointLight, 10u>;
    simple: [[stride(32)]] array<SimpleLight, 10u>;
    spot: [[stride(64)]] array<SpotLight, 10u>;
};
[[group(0), binding(5)]]
var r_light: Light;

[[group(0), binding(3)]]
var r_color: texture_2d<f32>;

[[group(0), binding(4)]]
var r_sampler: sampler;




fn calculate_simple_light(
    light: SimpleLight,
    color: vec3<f32>,
    position: vec4<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {

    var light_color: vec3<f32> = color;

    var light_direction: vec3<f32> = normalize(light.position.xyz - position.xyz);
    var diffuse: f32 = max(0.0, dot(normal, light_direction));

    light_color = light_color + diffuse * light.color.rgb;
    return light_color;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var result_color: vec4<f32> = textureSample(r_color, r_sampler, in.tex_uv);

    var light_color: vec3<f32> = r_light.ambient.xyz;
    var i: u32 = 0u;

    // for (int i = 0; i < int(lights_count.x) && i < MAX_LIGHTS; i++) {
    //     light_color = CalculateDirLight(dir_lights[i], light_color, normal);
    // }

    // for (int i = 0; i < int(lights_count.y) && i < MAX_LIGHTS; i++) {
    //     light_color = CalculatePointLight(point_lights[i], light_color, normal);
    // }

    var count: u32 = min(u32(r_light.count.z), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) {
            break;
        }

        light_color = calculate_simple_light(
            r_light.simple[i],
            light_color,
            in.position,
            in.normal
        );

        continuing {
            i = i + 1u;
        } 
    }

    // for (int i = 0; i < int(lights_count.w) && i < MAX_LIGHTS; i++) {
    //     light_color = CalculateSpotLight(spot_lights[i], light_color, normal);
    // }


    result_color = result_color * vec4<f32>(light_color.xyz, 1.0);

    //mag: f32 = length(v_TexCoord-vec2(0.5));
    return result_color;
    // o_Target = vec4(mix(result_color.xyz, vec3(0.0), mag*mag), 1.0);
}


