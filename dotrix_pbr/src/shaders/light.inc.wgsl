let MAX_LIGHTS_COUNT: u32 = {{ max_lights_count }};

struct DirectionalLight {
    direction: vec4<f32>;
    color: vec4<f32>;
};

struct PointLight {
    position: vec4<f32>;
    color: vec4<f32>;
    attenuation: vec4<f32>;
    // attenuation
    // a_constant: f32;
    // a_linear: f32;
    // a_quadratic: f32;
    // unused: f32;
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
    directional: [[stride(32)]] array<DirectionalLight, MAX_LIGHTS_COUNT>;
    point: [[stride(48)]] array<PointLight, MAX_LIGHTS_COUNT>;
    simple: [[stride(32)]] array<SimpleLight, MAX_LIGHTS_COUNT>;
    spot: [[stride(64)]] array<SpotLight, MAX_LIGHTS_COUNT>;
};

[[group({{ bind_group }}), binding({{ binding }})]]
var u_light: Light;

fn calculate_directional(
    light: DirectionalLight,
    color: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(-light.direction.xyz);
    let diffuse: f32 = max(0.0, dot(normal, light_direction));
    return color + diffuse * light.color.rgb;
}


fn calculate_point(
    light: PointLight,
    color: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position);
    let diffuse: f32 = max(0.0, dot(normal, light_direction));

    let light_distance: f32 = length(light.position.xyz - position.xyz);
    let attenuation: f32 = 1.0 / (
        light.attenuation.x
        + light.attenuation.y * light_distance
        + light.attenuation.z * (light_distance * light_distance)
    );

    return color + (diffuse * light.color.rgb * attenuation);
}


fn calculate_simple(
    light: SimpleLight,
    color: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position.xyz);
    let diffuse: f32 = max(0.0, dot(normal, light_direction));

    return color + diffuse * light.color.rgb;
}


fn calculate_spot(
    light: SpotLight,
    color: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec3<f32> {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position.xyz);
    let theta: f32 = dot(light_direction, normalize(-light.direction.xyz));

    let epsilon: f32 = light.cut_off - light.outer_cut_off;
    let intensity: f32 = clamp((theta - light.outer_cut_off) / epsilon, 0.0, 1.0);

    let diffuse: f32 = max(0.0, dot(normal, light_direction));
    return color + (diffuse * light.color.xyz) * intensity;
}


fn calculate_light(
    position: vec3<f32>,
    normal: vec3<f32>,
) -> vec4<f32> {

    var light_color: vec3<f32> = u_light.ambient.xyz;
    var i: u32 = 0u;
    var count: u32 = min(u32(u_light.count.x), MAX_LIGHTS_COUNT);

    loop {
        if (!(i < count)) { break; }

        light_color = calculate_directional(
            u_light.directional[i],
            light_color,
            normal
        );
        continuing { i = i + 1u; } 
    }

    i = 0u;
    count = min(u32(u_light.count.y), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) { break; }

        light_color = calculate_point(
            u_light.point[i],
            light_color,
            position,
            normal
        );

        continuing { i = i + 1u; }
    }

    i = 0u;
    count = min(u32(u_light.count.z), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) { break; }

        light_color = calculate_simple(
            u_light.simple[i],
            light_color,
            position,
            normal
        );

        continuing { i = i + 1u; } 
    }

    i = 0u;
    count = min(u32(u_light.count.w), MAX_LIGHTS_COUNT);
    loop {
        if (!(i < count)) { break; }

        light_color = calculate_spot(
            u_light.spot[i],
            light_color,
            position,
            normal
        );

        continuing { i = i + 1u; } 
    }

    return vec4<f32>(light_color, 1.0);
}

