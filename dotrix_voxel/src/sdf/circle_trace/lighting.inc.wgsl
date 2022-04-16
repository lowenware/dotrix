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

struct GenericLight {
  position: vec4<f32>;
  direction: vec4<f32>;
  color: vec4<f32>;
  parameters: vec4<f32>;
  kind: u32; // 1 = DirectionalLight, 2 = PointLight, 3 = SimpleLight, 4 = SpotLight, 0 = None
};

struct Lights {
    generic_lights: array<GenericLight>;
};

[[group(0), binding(2)]]
var<storage, read> s_lights: Lights;

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

fn calculate_light_ray_for(
    camera_index: u32,
    position: vec3<f32>,
) -> LightCalcOutput {
  var generic_light: GenericLight = s_lights.generic_lights[camera_index];
  switch (generic_light.kind) {
    case 1: {
      var light: DirectionalLight;
      light.direction = generic_light.direction;
      light.color = generic_light.color;
      return calculate_directional(light);
    }
    case 2: {
      var light: PointLight;
      light.position = generic_light.position;
      light.color = generic_light.color;
      light.attenuation = generic_light.parameters;
      return calculate_point(light, position);
    }
    case 3: {
      var light: SimpleLight;
      light.position = generic_light.position;
      light.color = generic_light.color;
      return calculate_simple(light, position);
    }
    case 4: {
      var light: SpotLight;
      light.direction = generic_light.direction;
      light.position = generic_light.position;
      light.color = generic_light.color;
      light.cut_off = generic_light.parameters.x;
      light.outer_cut_off = generic_light.parameters.y;
      return calculate_spot(light, position);
    }
    default: {
      var out: LightCalcOutput;
      out.light_direction = vec3<f32>(0.);
      out.radiance = vec3<f32>(0.);
      return out;
    }
  }
}

// Ambient is stored in the last light
fn get_light_count() -> u32 {
  return arrayLength(&s_lights.generic_lights) - 1u;
}

fn get_ambient() -> vec3<f32> {
  let idx: u32 = u32(arrayLength(&s_lights.generic_lights)) - 1u;
  return s_lights.generic_lights[idx].color.xyz;
}
