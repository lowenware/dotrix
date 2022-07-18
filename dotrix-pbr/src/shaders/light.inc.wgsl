let MAX_LIGHTS_COUNT: u32 = {{ max_lights_count }};

struct LightCalcOutput {
  light_direction: vec3<f32>;
  radiance: vec3<f32>;
};

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
};

struct Light {
    camera_position: vec4<f32>;
    ambient: vec4<f32>;
    count: vec4<u32>;
    directional: [[stride(32)]] array<DirectionalLight, MAX_LIGHTS_COUNT>;
    point: [[stride(48)]] array<PointLight, MAX_LIGHTS_COUNT>;
    simple: [[stride(32)]] array<SimpleLight, MAX_LIGHTS_COUNT>;
    spot: [[stride(64)]] array<SpotLight, MAX_LIGHTS_COUNT>;
};

[[group({{ bind_group }}), binding({{ binding }})]]
var<uniform> u_light: Light;

fn calculate_directional(
    light: DirectionalLight,
) -> LightCalcOutput {
    let light_direction: vec3<f32> = normalize(-light.direction.xyz);

    var out: LightCalcOutput;
    out.light_direction = light_direction;
    out.radiance = light.color.rgb;
    return out;
}


fn calculate_point(
    light: PointLight,
    position: vec3<f32>,
) -> LightCalcOutput {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position);

    let light_distance: f32 = length(light.position.xyz - position.xyz);
    let attenuation: f32 = 1.0 / (
        light.attenuation.x
        + light.attenuation.y * light_distance
        + light.attenuation.z * (light_distance * light_distance)
    );

    var out: LightCalcOutput;
    out.light_direction = light_direction;
    out.radiance = light.color.rgb * attenuation;
    return out;
}

fn calculate_simple(
    light: SimpleLight,
    position: vec3<f32>,
) -> LightCalcOutput {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position.xyz);

    var out: LightCalcOutput;
    out.light_direction = light_direction;
    out.radiance = light.color.rgb;
    return out;
}


fn calculate_spot(
    light: SpotLight,
    position: vec3<f32>,
) -> LightCalcOutput {
    let light_direction: vec3<f32> = normalize(light.position.xyz - position.xyz);
    let theta: f32 = dot(light_direction, normalize(-light.direction.xyz));

    let epsilon: f32 = light.cut_off - light.outer_cut_off;
    let intensity: f32 = clamp((theta - light.outer_cut_off) / epsilon, 0.0, 1.0);

    var out: LightCalcOutput;
    out.light_direction = light_direction;
    out.radiance = light.color.rgb * intensity;
    return out;
}

// This will get the direction direction and intensity of
// the nth light towards a position
// If used in conjectuion with `get_light_count`
// It allows for more consistent iter code by providing
// A standard single data `LightCalcOutput` for any light
// regardless of type
fn calculate_nth_light_ray(
    in_camera_index: u32,
    position: vec3<f32>,
) -> LightCalcOutput {
  var camera_index: u32 = in_camera_index;
  // directional
  let dir_count = min(u32(u_light.count.x), MAX_LIGHTS_COUNT);
  if (camera_index < dir_count) {
    var light: DirectionalLight = u_light.directional[camera_index];
    return calculate_directional(light);
  }
  camera_index = camera_index - dir_count;
  // point
  let point_count = min(u32(u_light.count.y), MAX_LIGHTS_COUNT);
  if (camera_index < point_count) {
    var light: PointLight = u_light.point[camera_index];
    return calculate_point(light, position);
  }
  camera_index = camera_index - point_count;
  // simple
  let simple_count = min(u32(u_light.count.z), MAX_LIGHTS_COUNT);
  if (camera_index < simple_count) {
    var light: SimpleLight = u_light.simple[camera_index];
    return calculate_simple(light, position);
  }
  camera_index = camera_index - simple_count;
  // spot
  let spot_count = min(u32(u_light.count.w), MAX_LIGHTS_COUNT);
  if (camera_index < spot_count) {
    var light: SpotLight = u_light.spot[camera_index];
    return calculate_spot(light, position);
  }
  // Trying to access a non existant light
  var oob: LightCalcOutput;
  oob.light_direction = vec3<f32>(0.);
  oob.radiance = vec3<f32>(0.);
  return oob;
}

fn get_light_count() -> u32 {
  return u_light.count.x + u_light.count.y + u_light.count.z + u_light.count.w;
}

fn get_ambient() -> vec3<f32> {
  return u_light.ambient.xyz;
}
