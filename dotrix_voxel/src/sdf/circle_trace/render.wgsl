struct Camera {
  proj_view: mat4x4<f32>;
  static_camera_trans: mat4x4<f32>;
  pos: vec4<f32>;
  screen_resolution: vec2<f32>;
  fov: f32;
};
[[group(0), binding(0)]]
var<uniform> u_camera: Camera;

[[group(0), binding(1)]]
var r_sampler: sampler;

struct SdfData {
  // This transform scales the 1x1x1 cube so that it totally encloses the
  // voxels
  cube_transform: mat4x4<f32>;
  // Inverse cube_transform
  inv_cube_transform: mat4x4<f32>;
  // World transform of the voxel grid
  world_transform: mat4x4<f32>;
  // Inverse World transform of the voxel grid
  inv_world_transform: mat4x4<f32>;
  // Dimensions of the voxel
  voxel_dimensions: vec4<f32>;
  // Dimensions of the voxel
  grid_dimensions: vec4<f32>;
  // Scale in world space
  world_scale: vec4<f32>;
};
[[group(1), binding(0)]]
var<uniform> u_sdf: SdfData;

[[group(1), binding(1)]]
var sdf_texture: texture_3d<f32>;


struct VertexOutput {
  [[builtin(position)]] position: vec4<f32>;
  [[location(0)]] world_position: vec3<f32>;
  [[location(1)]] clip_coords: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
) -> VertexOutput {
    let pos_4: vec4<f32> = vec4<f32>(position, 1.);
    let local_coords: vec4<f32> = u_sdf.cube_transform * pos_4;
    let world_coords: vec4<f32> = u_sdf.world_transform * local_coords;
    let clip_coords: vec4<f32> =  u_camera.proj_view * world_coords;

    var out: VertexOutput;
    out.position = clip_coords;
    out.world_position = world_coords.xyz;
    out.clip_coords = clip_coords;
    return out;
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

// Use pixel based cones to get the size of the pizel
fn pixel_radius(t: f32, direction: vec3<f32>, direction_x: vec3<f32>, direction_y: vec3<f32>) -> f32 {
  let dx: f32 = length(t*(direction_x-direction));
  let dy: f32 = length(t*(direction_y-direction));
  return length(vec2<f32>(dx, dy)) * 0.1;
}

{% include "circle_trace/map.inc.wgsl" %}

{% include "circle_trace/accelerated_raytrace.inc.wgsl" %}

{% include "circle_trace/hemisphere_ambient_occulsion.inc.wgsl" %}

{% include "circle_trace/soft_shadows_closet_approach.inc.wgsl" %}

{% include "circle_trace/lighting.inc.wgsl" %}


struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
    [[builtin(frag_depth)]] depth: f32;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
  let debug: bool = true;
  let resolution: vec2<f32> = u_camera.screen_resolution.xy;
  let pixel_coords: vec2<u32> = vec2<u32>(u32(in.position.x), u32(resolution.y - in.position.y));

  let ro: vec3<f32> = u_camera.pos.xyz;

  // This can also be achieved by using world coords but we
  // do it in pixels coords to get the pixel differentials
  let rd: vec3<f32> = get_ray_direction(pixel_coords.xy, resolution);
  let rdx: vec3<f32> = get_ray_direction(pixel_coords.xy + vec2<u32>(1u, 0u), resolution);
  let rdy: vec3<f32> = get_ray_direction(pixel_coords.xy + vec2<u32>(0u, 1u), resolution);
  // let rd: vec3<f32> = normalize(in.world_position - ro); // Use world_coords instead

  // Current distance from camera to grid
  let t: f32 = length(in.world_position - ro);

  // March that ray
  let r_out: RaymarchOut = raymarch(t, ro, rd, rdx, rdy);
  let t_out: f32 = r_out.t;
  if (!debug && !r_out.success) {
     discard;
  }

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

    let intensity: f32 = dot(light_out.light_direction, nor);
    // If perpendicular don't bother (numerically unstable)
    if (abs(intensity) > 0.1  ) {
      var ray_in: SoftShadowInput;
      ray_in.origin = pos;
      ray_in.direction = light_out.light_direction;
      ray_in.max_iterations = 128u;
      ray_in.min_distance = 0.01;
      ray_in.max_distance = 100.;
      ray_in.k = 8.;

      let ray_out: SoftShadowResult = softshadow(ray_in);

      total_radiance = total_radiance + intensity * ray_out.radiance;
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
  if (r_out.success)  {
    out.color = vec4<f32>(total_radiance, 1.);
  } else {
    out.color = vec4<f32>(0.5,0.1,0.1,1.0);
  }

  let pos_clip: vec4<f32> = u_camera.proj_view * vec4<f32>(in.world_position, 1.);
  out.depth = pos_clip.z / pos_clip.w;
  return out;
}
