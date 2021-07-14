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
    let world_position: vec4<f32> = u_model.transform * vec4<f32>(position, 1.0);
    out.world_position = world_position.xyz;
    out.position = u_renderer.proj_view * world_position;
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


[[group(0), binding(3)]]
var r_texture: texture_2d<f32>;

[[group(0), binding(4)]]
var r_sampler: sampler;

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
var u_light: Light;


fn calculate_directional_light(
    light: DirectionalLight,
    color: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(-light.direction.xyz);
    let diffuse: f32 = max(0.0, dot(normal, light_direction));
    return color + diffuse * light.color.rgb;
}


fn calculate_point_light(
    light: PointLight,
    color: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position);
    let diffuse: f32 = max(0.0, dot(normal, light_direction));

    let light_distance: f32 = length(light.position.xyz - position.xyz);
    let attenuation: f32 = 1.0 / (
        light.a_constant
        + light.a_linear * light_distance
        + light.a_quadratic * (light_distance * light_distance)
    );

    return color + (diffuse * light.color.rgb) * attenuation;
}


fn calculate_simple_light(
    light: SimpleLight,
    color: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position);
    let diffuse: f32 = max(0.0, dot(normal, light_direction));

    return color + diffuse * light.color.rgb;
}


fn calculate_spot_light(
    light: SpotLight,
    color: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position);
    let theta: f32 = dot(light_direction, normalize(-light.direction.xyz));

    let epsilon: f32 = light.cut_off - light.outer_cut_off;
    let intensity: f32 = clamp((theta - light.outer_cut_off) / epsilon, 0.0, 1.0);

    let diffuse: f32 = max(0.0, dot(normal, light_direction));
    return color + (diffuse * light.color.xyz) * intensity;
}


fn inverse_lerp(left: f32, right: f32, value: f32) -> f32 {
    return clamp((value - left) / (right - left), 0.0, 1.0);
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let texture_color: vec4<f32> = textureSample(r_texture, r_sampler, in.tex_uv / 0.5);
    var albedo_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    var light_color: vec3<f32> = u_light.ambient.xyz;

    // Terrain Types
    let max_height: f32 = 300.0;
    let height_blend: f32 = 0.02;
    let epsilon: f32 = 0.0001;
    let height_percent: f32 = inverse_lerp(0.0, max_height, in.world_position.y);

    let water_base: f32 = -1.0;
    let water_color: vec4<f32> = vec4<f32>(0.0, 0.538, 1.0, 1.0);

    let sand_base: f32 = 0.020;
    let sand_color: vec4<f32> = vec4<f32>(0.923, 0.832, 0.472, 1.0);

    let grass_base: f32 = 0.035;
    let grass_color: vec4<f32> = vec4<f32>(0.173, 0.622, 0.235, 1.0);

    let stone_base: f32 = 0.20;
    let stone_color: vec4<f32> = vec4<f32>(0.193, 0.188, 0.201, 1.0);

    let snow_base: f32 = 0.4;
    let snow_color: vec4<f32> = vec4<f32>(0.813, 0.936, 0.999, 1.0);

    // loop
    var color_strength: f32 = inverse_lerp(
        -height_blend / 2.0 - epsilon,
        height_blend / 2.0,
        height_percent - water_base
    );
    albedo_color = albedo_color * (1.0 - color_strength) + water_color * color_strength;

    color_strength = inverse_lerp(
        -height_blend / 2.0 - epsilon,
        height_blend / 2.0,
        height_percent -sand_base 
    );
    albedo_color = albedo_color * (1.0 - color_strength) + sand_color * color_strength;

    color_strength = inverse_lerp(
        -height_blend / 2.0 - epsilon,
        height_blend / 2.0,
        height_percent -grass_base 
    );
    albedo_color = albedo_color * (1.0 - color_strength) + grass_color * color_strength;

    color_strength = inverse_lerp(
        -height_blend / 2.0 - epsilon,
        height_blend / 2.0,
        height_percent -stone_base 
    );
    albedo_color = albedo_color * (1.0 - color_strength) + stone_color * color_strength;

    color_strength = inverse_lerp(
        -height_blend / 2.0 - epsilon,
        height_blend / 2.0,
        height_percent -snow_base 
    );
    albedo_color = albedo_color * (1.0 - color_strength) + snow_color * color_strength;

    // Light
    var i: u32;
    var count: u32;

    i = 0u;
    count = min(u32(u_light.count.x), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) { break; }

        light_color = calculate_directional_light( u_light.directional[i], light_color, in.world_position, in.normal);

        continuing { i = i + 1u; } 
    }

    i = 0u;
    count = min(u32(u_light.count.y), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) { break; }

        light_color = calculate_point_light( u_light.point[i], light_color, in.world_position, in.normal);

        continuing { i = i + 1u; }
    }

    i = 0u;
    count = min(u32(u_light.count.z), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) { break; }

        light_color = calculate_simple_light( u_light.simple[i], light_color, in.world_position, in.normal);

        continuing { i = i + 1u; } 
    }

    i = 0u;
    count = min(u32(u_light.count.w), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) { break; }

        light_color = calculate_spot_light( u_light.spot[i], light_color, in.world_position, in.normal);

        continuing { i = i + 1u; } 
    }


  return vec4<f32>(albedo_color.rgb * texture_color.rgb * light_color, 1.0);

    //mag: f32 = length(v_TexCoord-vec2(0.5));
    // o_Target = vec4(mix(result_color.xyz, vec3(0.0), mag*mag), 1.0);
}


