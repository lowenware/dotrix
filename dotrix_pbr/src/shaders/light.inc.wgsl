// Most of this comes from https://learnopengl.com/PBR/Lighting
let MAX_LIGHTS_COUNT: u32 = {{ max_lights_count }};
let PI: f32 = 3.14159;

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
    normal: vec3<f32>,
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
    normal: vec3<f32>,
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
    normal: vec3<f32>,
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
    normal: vec3<f32>,
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

fn distribution_ggx(normal: vec3<f32>, halfway: vec3<f32>, roughness: f32) -> f32
{
    let a: f32 = roughness*roughness;
    let a2: f32 = a*a;
    let n_dot_h: f32 = max(dot(normal, halfway), 0.0);
    let n_dot_h_2: f32 = n_dot_h*n_dot_h;

    let num: f32 = a2;
    var denom: f32 = (n_dot_h_2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32
{
    let r: f32 = (roughness + 1.0);
    let k: f32 = (r*r) / 8.0;

    let num: f32   = n_dot_v;
    let denom: f32 = n_dot_v * (1.0 - k) + k;

    return num / denom;
}
fn geometry_smith(normal: vec3<f32>, camera_direction: vec3<f32>, light_direction: vec3<f32>, roughness: f32) -> f32
{
    let n_dot_v: f32 = max(dot(normal, camera_direction), 0.0);
    let n_dot_l: f32 = max(dot(normal, light_direction), 0.0);
    let ggx2: f32  = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1: f32  = geometry_schlick_ggx(n_dot_l, roughness);

    return ggx1 * ggx2;
}

// Calulates the amount of light that refects (specular) and that which scatters (diffuse)
fn calculate_fresnel_schlick(cos_theta: f32, fresnel_schlick_0: vec3<f32>) -> vec3<f32>
{
    return fresnel_schlick_0 + (vec3<f32>(1.0) - fresnel_schlick_0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn pbr(
  light_out: LightCalcOutput,
  camera_direction: vec3<f32>, // Camera direction
  normal: vec3<f32>, // normal
  fresnel_schlick_0: vec3<f32>, // surface reflection at zero incidence
  albedo: vec3<f32>, // scatter color in linear space
  metallic: f32, // Metallic (reflectance)
  roughness: f32 // Roughness (random scatter)
) -> vec3<f32> {
  let light_direction: vec3<f32> = light_out.light_direction;
  let halfway: vec3<f32> = normalize(camera_direction + light_direction);

  // cook-torrance brdf
  let normal_distribution_function: f32 = distribution_ggx(normal, halfway, roughness);
  let geometry_function: f32   = geometry_smith(normal, camera_direction, light_direction, roughness);
  let fresnel_schlick: vec3<f32>  = calculate_fresnel_schlick(max(dot(halfway, camera_direction), 0.0), fresnel_schlick_0);

  let reflection_specular_fraction: vec3<f32> = fresnel_schlick;
  var refraction_diffuse_fraction: vec3<f32> = vec3<f32>(1.0) - reflection_specular_fraction; // refraction/diffuse  fraction
  refraction_diffuse_fraction = refraction_diffuse_fraction * (1.0 - metallic);

  let numerator: vec3<f32> = normal_distribution_function * geometry_function * fresnel_schlick;
  let denominator: f32 = 4.0 * max(dot(normal, camera_direction), 0.0) * max(dot(normal, light_direction), 0.0) + 0.0001;
  let specular: vec3<f32>     = numerator / denominator;

  // get the outgoing radiance
  let n_dot_l: f32 = max(dot(normal, light_direction), 0.0);
  return (refraction_diffuse_fraction * albedo / PI + specular) * light_out.radiance * n_dot_l;
}

fn calculate_lighting(
    position: vec3<f32>,
    normal_in: vec3<f32>,
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32,
    ao: f32,
) -> vec4<f32> {
    let camera_position: vec3<f32> = u_light.camera_position.xyz;
    var light_color: vec3<f32> = vec3<f32>(0.);

    let normal: vec3<f32> = normalize(normal_in );
    let camera_direction: vec3<f32> = normalize(camera_position - position);

    var fresnel_schlick_0: vec3<f32> = vec3<f32>(0.04);
    fresnel_schlick_0 = mix(fresnel_schlick_0, albedo, vec3<f32>(metallic));

    // Directions light
    var i: u32 = 0u;
    var count: u32 = min(u32(u_light.count.x), MAX_LIGHTS_COUNT);
    for (i = 0u; i< count; i = i + 1u) {
      let light_result = calculate_directional(
          u_light.directional[i],
          normal
      );
      light_color = light_color + pbr(
        light_result,
        camera_direction,
        normal,
        fresnel_schlick_0,
        albedo,
        metallic,
        roughness
      );
    }
    // Point light
    count = min(u32(u_light.count.y), MAX_LIGHTS_COUNT);
    for (i = 0u; i< count; i = i + 1u) {
      let light_result = calculate_point(
          u_light.point[i],
          position,
          normal
      );
      light_color = light_color + pbr(
        light_result,
        camera_direction,
        normal,
        fresnel_schlick_0,
        albedo,
        metallic,
        roughness
      );
    }
    // Simple light
    count = min(u32(u_light.count.z), MAX_LIGHTS_COUNT);
    for (i = 0u; i< count; i = i + 1u) {
      let light_result = calculate_simple(
          u_light.simple[i],
          position,
          normal
      );
      light_color = light_color + pbr(
        light_result,
        camera_direction,
        normal,
        fresnel_schlick_0,
        albedo,
        metallic,
        roughness
      );
    }
    // Spot light
    count = min(u32(u_light.count.w), MAX_LIGHTS_COUNT);
    for (i = 0u; i< count; i = i + 1u) {
      let light_result = calculate_spot(
          u_light.spot[i],
          position,
          normal
      );
      light_color = light_color + pbr(
        light_result,
        camera_direction,
        normal,
        fresnel_schlick_0,
        albedo,
        metallic,
        roughness
      );
    }

    // Ambient
    let ambient = u_light.ambient.xyz * albedo * ao;
    light_color = light_color + ambient;

    // Gamma correct
    light_color = light_color / (light_color + vec3<f32>(1.0));
    light_color = pow(light_color, vec3<f32>(1.0/2.2));

    return vec4<f32>(light_color, 1.0);
}
