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

fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32
{
    let a: f32 = roughness*roughness;
    let a2: f32 = a*a;
    let NdotH: f32 = max(dot(N, H), 0.0);
    let NdotH2: f32 = NdotH*NdotH;

    let num: f32 = a2;
    var denom: f32 = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32
{
    let r: f32 = (roughness + 1.0);
    let k: f32 = (r*r) / 8.0;

    let num: f32   = NdotV;
    let denom: f32 = NdotV * (1.0 - k) + k;

    return num / denom;
}
fn GeometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32
{
    let NdotV: f32 = max(dot(N, V), 0.0);
    let NdotL: f32 = max(dot(N, L), 0.0);
    let ggx2: f32  = GeometrySchlickGGX(NdotV, roughness);
    let ggx1: f32  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

// Calulates the amount of light that refects (specular) and that which scatters (diffuse)
fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32>
{
    return F0 + (vec3<f32>(1.0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn pbr(
  light_out: LightCalcOutput,
  V: vec3<f32>, // Camera direction
  N: vec3<f32>, // normal
  F0: vec3<f32>, // surface reflection at zero incidence
  albedo: vec3<f32>, // scatter color in linear space
  metallic: f32, // Metallic (reflectance)
  roughness: f32 // Roughness (random scatter)
) -> vec3<f32> {
  let L: vec3<f32> = light_out.light_direction;
  let H: vec3<f32> = normalize(V + L);

  // cook-torrance brdf
  let NDF: f32 = DistributionGGX(N, H, roughness);
  let G: f32   = GeometrySmith(N, V, L, roughness);
  let F: vec3<f32>  = fresnelSchlick(max(dot(H, V), 0.0), F0);

  let kS: vec3<f32> = F;
  var kD: vec3<f32> = vec3<f32>(1.0) - kS;
  kD = kD * (1.0 - metallic);

  let numerator: vec3<f32> = NDF * G * F;
  let denominator: f32 = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
  let specular: vec3<f32>     = numerator / denominator;

  // get the outgoing radiance
  let NdotL: f32 = max(dot(N, L), 0.0);
  return (kD * albedo / PI + specular) * light_out.radiance * NdotL;
}

fn calculate_lighting(
    position: vec3<f32>,
    normal: vec3<f32>,
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32,
    ao: f32,
) -> vec4<f32> {
    let camera_position: vec3<f32> = u_light.camera_position.xyz;
    var light_color: vec3<f32> = vec3<f32>(0.);

    let N: vec3<f32> = normalize(normal);
    let V: vec3<f32> = normalize(camera_position - position);

    var F0: vec3<f32> = vec3<f32>(0.04);
    F0 = mix(F0, albedo, vec3<f32>(metallic));

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
        V,
        N,
        F0,
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
        V,
        N,
        F0,
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
        V,
        N,
        F0,
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
        V,
        N,
        F0,
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
