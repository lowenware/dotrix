struct Camera {
  proj_view: mat4x4<f32>;
  static_camera_trans: mat4x4<f32>;
  pos: vec4<f32>;
  screen_resolution: vec2<u32>;
};
[[group(0), binding(0)]]
var<uniform> u_camera: Camera;

[[group(0), binding(1)]]
var r_sampler: sampler;

struct SdfData {
  // This transform scales the 1x1x1 cube so that it totally encloses the
  // voxels
  cube_transform: mat4x4<f32>;
  // World transform of the voxel grid
  world_transform: mat4x4<f32>;
  // Dimensions of the voxel
  grid_dimensions: vec3<f32>
};
[[group(1), binding(0)]]
var<uniform> u_sdf: SdfData;

[[group(1), binding(1)]]
var sdf_texture: texture_3d<f32>;


struct VertexOutput {
  [[builtin(position)]] position: vec4<f32>;
  [[location(0)]] world_position: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
) -> VertexOutput {
    let pos_4: vec4<f32>(position, 1.);
    let local_coords: vec4<f32> = u_sdf.cube_transform * pos_4;
    let world_coords: vec4<f32> = u_sdf.world_transform * local_coords;
    let clip_coords: vec4<f32> =  u_camera.proj_view * world_coords;

    var out: VertexOutput;
    out.position = clip_coords;
    out.world_position = world_coords.xyz;
}

// Convert clip space coordinates to pixel coordinates
fn clip_to_pixels(clip: vec2<f32>, resolution: vec2<f32>) -> vec2<u32> {
  let pixel_f32: vec2<f32> = (clip * resolution.y + resolution.xy ) / 2.;
  return vec2<u32>(u32(pixel_f32.x), u32(pixel_f32.y));
}

// Given pixel coordinates get the ray direction
fn get_ray_direction(pixel: vec2<u32>, resolution: vec2<f32>) -> vec3<f32> {
  let pixel_f32: vec2<f32> = vec2<f32>(f32(pixel.x), f32(pixel.y));
  let p: vec2<f32> =  (2.0 * pixel_f32 - resolution.xy)/(resolution.y);
  let z: f32 = -1. / tan(u_camera.fov * 0.5);
  let view_coordinate: vec4<f32> = vec4<f32>(p.x, p.y, z, 1.);
  let world_coordinate: vec4<f32> = u_camera.static_camera_trans * view_coordinate;

  return normalize(world_coordinate.xyz);
}

// Get distance to surface from a point
fn map(p: vec3<f32>) -> f32
{
    let d: f32 = {{ MAP }};

    return d;
}
// Get the material id at a point
fn map_material(p: vec3<f32>) -> u8
{
    let d: f32 = {{ MAP }};

    return d;
}

// Surface gradient (is the normal)
fn map_normal (p: vec3<f32>) -> vec3<f32>
{
  // return vec3<f32>(0.);
	let eps: f32 = 0.001;

	return normalize
	(	vec3<f32>
		(	map(p + vec3<f32>(eps, 0., 0.)	) - map(p - vec3<f32>(eps, 0., 0.)),
			map(p + vec3<f32>(0., eps, 0.)	) - map(p - vec3<f32>(0., eps, 0.)),
			map(p + vec3<f32>(0., 0., eps)	) - map(p - vec3<f32>(0., 0., eps))
		)
	);
}

// Use pixel based cones to get the size of the pizel
fn pixel_radius(t: f32, direction: vec3<f32>, direction_x: vec3<f32>, direction_y: vec3<f32>) -> f32 {
  let dx: f32 = length(t*(direction_x-direction));
  let dy: f32 = length(t*(direction_y-direction));
  return length(vec2<f32>(dx, dy)) * 0.4;
}

// Accelerated raymarching
// https://www.researchgate.net/publication/331547302_Accelerating_Sphere_Tracing
fn raymarch(t_in: f32, rd: vec3<f32>, rdx: vec3<f32>, rdy: vec3<f32>) -> f32 {
  let o: vec3<f32> = ro;
  let d: vec3<f32> = rd;
  let dx: vec3<f32> = rdx;
  let dy: vec3<f32> = rdy;

  let STEP_SIZE_REDUCTION: f32 = 0.95;
  let MAX_DISTANCE: f32 = t_in + length(u_sdf.grid_dimensions);
  let MAX_ITERATIONS: u32 = 235u;

  var t: f32 = t_in;
  var rp: f32 = 0.; // prev
  var rc: f32 = map(o + (t)*d);; // current
  var rn: f32 = t + MAX_DISTANCE * 2.0; // next (set to effectivly infinity)

  var di: f32 = 0.;

  for(var i: u32 = 0u; i < MAX_ITERATIONS && t < MAX_DISTANCE; i = i + 1u)
  {
    di = rc + STEP_SIZE_REDUCTION * rc * max( (di - rp + rc) / (di + rp - rc), 0.6);
    rn = map(o + (t + di)*d);
    if(di > rc + rn) {
      di = rc;
      rn = map(o + (t + di)*d);
    }
    if(rn < pixel_radius(t + di, d, dx, dy)) {
      return t + di;
    }
    t = t + di;
    rp = rc;
    rc = rn;
  }
  discard;
}

// AO
struct AoResult {
  ao: f32;
};

struct AoInput {
  origin: vec3<f32>;
  direction: vec3<f32>;
  samples: u32;
  steps: u32;
  ao_step_size: f32;
};

let PI: f32 = 3.14159265358979;

// Uniform points on a hemisphere
// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
fn radicalInverse_VdC(in_bits: u32) -> f32 {
    var bits: u32 = in_bits;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}
fn hammersley2d(i: u32, N: u32) -> vec2<f32> {
     return vec2<f32>(f32(i)/f32(N), radicalInverse_VdC(i));
}
fn hemisphereSample_uniform(u: f32, v: f32) -> vec3<f32> {
     let phi: f32 = v * 2.0 * PI;
     let cosTheta: f32 = 1.0 - u;
     let sinTheta: f32 = sqrt(1.0 - cosTheta * cosTheta);
     return vec3<f32>(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);
}

// Hemisphere based AO
// http://www.aduprat.com/portfolio/?page=articles/hemisphericalSDFAO
// Adapted to use hammersley2d sampling instead of random
// and to step along the ray
fn ambient_occlusion(input: AoInput) -> AoResult
{
    let nb_ite: u32 = input.samples;
    let nb_ite_inv: f32 = 1./f32(nb_ite);
    let rad: f32 = 1. - (1. * nb_ite_inv); //Hemispherical factor (self occlusion correction)

    var ao: f32 = 0.0;

    // Tangent space tranformation
    let a: vec3<f32> = vec3<f32>(0., 0., 1.);
    let b: vec3<f32> = input.direction;
    let v: vec3<f32> = cross(a,b);
    let s: f32 =  length(v);
    let I: mat3x3<f32> = mat3x3<f32>(vec3<f32>(1.,0.,0.), vec3<f32>(0.,1.,0.), vec3<f32>(0.,0.,1.));
    var R: mat3x3<f32>;
    if (abs(s) < 0.01) {
      R = I;
    } else {
      let c: f32 = dot(a, b);
      let sx: mat3x3<f32> = mat3x3<f32>(vec3<f32>(0.,v.z,-v.y), vec3<f32>(-v.z,0.,v.x), vec3<f32>(v.y,-v.x,0.));
      R = I + sx + sx * sx * (1./(1. + c));
    }



    for( var i: u32 = 0u; i < nb_ite; i = i + 1u )
    {
        let hammersley: vec2<f32> = hammersley2d(i, nb_ite);
        let rd = hemisphereSample_uniform(hammersley.x, hammersley.y);

        // In tangent space
        let direction: vec3<f32> = R * rd;

        // Stepping on the ray
        var sum: f32 = 0.;
        var max_sum: f32 = 0.;
        for (var j: u32 = 0u; j < input.steps; j = j + 1u)
      	{
          let p: vec3<f32> = input.origin + direction * f32(j + 1u) * input.ao_step_size;
            sum     = sum     + 1. / pow(2., f32(j)) * max(map(p), 0.);
            max_sum = max_sum + 1. / pow(2., f32(j)) * f32(j + 1u) * input.ao_step_size;
      	}

        ao = ao + (sum / max_sum) / f32(nb_ite);
    }

    var ray_out: AoResult;
    ray_out.ao = clamp(ao, 0., 1.);
    return ray_out;
}

// Shadows

struct SoftShadowResult {
  radiance: f32;
};

struct SoftShadowInput {
  origin: vec3<f32>;
  direction: vec3<f32>;
  max_iterations: u32;
  min_distance: f32;
  max_distance: f32;
  k: f32;
};

//
// // Csaba Bálint et al. / Accelerated Sphere Tracing
// Sebastian Aaltonen et al. Soft Shadows at his GDC presentation
fn softshadow (input: SoftShadowInput) -> SoftShadowResult
{
  let o: vec3<f32> = input.origin;
  let d: vec3<f32> = input.direction;

  var di: f32 = 0.;
  var t: f32 = input.min_distance;

  let STEP_SIZE_REDUCTION: f32 = 0.95;
  var rp: f32 = 0.; // prev
  var rc: f32 = 0.; // current large such that y=0.0 at first
  var rn: f32 = map(o + (t)*d); // next

  var radiance: f32 = 1.;

  for(var i: u32 = 0u; i < input.max_iterations && t < input.max_distance; i = i + 1u)
  {
    let y: f32 = rn*rn/(2.0*rc);
    let approx_distance: f32 = sqrt(rn*rn-y*y);
    radiance = min(radiance, input.k * approx_distance/max(0.0,t-y));

    di = rc + STEP_SIZE_REDUCTION * rc * max( (di - rp + rc) / (di + rp - rc), 0.6);
    rn = map(o + (t + di)*d);
    if(di > rc + rn)
    {
      di = rc;
      rn = map(o + (t + di)*d);
    }
    if(rn < 0.001) {
      var out: SoftShadowResult;
      out.radiance = 0.;
      return out;
    }
    t = t + di;

    rp = rc;
    rc = rn;
  }
  var out: SoftShadowResult;
  out.radiance = radiance;
  return out;
}

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

[[group(0), binding(2]]
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


struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
    [[builtin(frag_depth)]] depth: f32;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {

  let resolution: vec2<f32> = vec2<f32>(f32(u_camera.screen_resolution.x), f32(screen_resolution.y));
  let pixel_coords: vec2<u32> = clip_coords(in.position.xy, resolution);

  let ro: vec3<f32> = u_camera.pos;
  // This can also be achieved by using world coords but we
  // do it in pixels coords to get the pixel differentials
  let rd: vec3<f32> = get_ray_direction(pixel_coords.xy, resolution);
  let rdx: vec3<f32> = get_ray_direction(pixel_coords.xy + vec2<u32>(1u, 0u), resolution);
  let rdy: vec3<f32> = get_ray_direction(pixel_coords.xy + vec2<u32>(0u, 1u), resolution);

  // Current distance from camera to grid
  let t: f32 = length(in.world_position = ro);

  // March that ray
  let t_out: f32 = raymarch(t, rd, rdx, rdy);

  // Final position of the ray
  let pos: vec3<f32> = ro + rd*t_out;

  // Normal of the surface
  let nor: vec3<f32> = map_normal(pos);

  // AO
  var ray_in: AoInput;
  ray_in.origin = ro;
  ray_in.direction = nor;
  ray_in.samples = 32u;
  ray_in.steps = 8u;
  ray_in.ao_step_size = 0.01;

  let ao: f32 = 1. - clamp(0., .1, ambient_occlusion(ray_in).ao);

  // Shadows
  var total_radiance: vec3<f32> = vec3<f32>(0.);
  total_radiance = total_radiance + get_ambient();

  let light_count: u32 = get_light_count();
  for (var i: u32 = 0u; i<light_count; i = i + 1u) {
    let light_out: LightCalcOutput = calculate_light_ray_for(i, ro);
    let L: vec3<f32> = light_out.light_direction;

    // If perpendicular don't bother (numerically unstable)
    if (abs(dot(light_out.light_direction, N)) > 0.1  ) {
      var ray_in: SoftShadowInput;
      ray_in.origin = ro;
      ray_in.direction = light_out.light_direction;
      ray_in.max_iterations = 128u;
      ray_in.min_distance = 0.01;
      ray_in.max_distance = 100.;
      ray_in.k = 8.;

      let ray_out: SoftShadowResult = softshadow(ray_in);

      total_radiance = total_radiance + ray_out.radiance;
    }
  }
  total_radiance = clamp(vec3<f32>(0.), vec3<f32>(1.), total_radiance);

  // TODO: Work out how to bind textures effectivly
  // // Ray differntials
  // let dp_dxy: DpDxy = calcDpDxy( ro, rd, rdx, rdy, t, nor );
  //
  // // Material ID
  // let material_id: i32 = i32(map_material(pos));
  //
  // // Surface material
  // let sur: Material = get_material(pos, nor, dp_dxy.dposdx, dp_dxy.dposdy, material_id);
  //
  // // Lighting and PBR
  // let shaded: vec4<f32> = calculate_lighting(
  //     pos,
  //     sur.normal,
  //     sur.albedo.rgb,
  //     sur.roughness,
  //     sur.metallic,
  //     sur.ao,
  // );

  var out: FragmentOutput;
  out.color = shaded;
  let pos_clip: vec4<f32> = u_camera.proj_view * vec4<f32>(pos, 1.);
  out.frag_depth = pos_clip.z / pos_clip.w;
  return out;
}
