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

// SDF for a box, b is half box size
fn sdBox(p: vec3<f32>,b: vec3<f32>) -> f32
{
  let q: vec3<f32> = abs(p) - b;
  return length(max(q,vec3<f32>(0.0))) + min(max(q.x,max(q.y,q.z)),0.0);
}

// Get distance to surface from a point
fn map(p: vec3<f32>) -> f32
{
    let local_p: vec4<f32> = (u_sdf.inv_world_transform * vec4<f32>(p, 1.));
    let cube_p: vec4<f32> = (u_sdf.inv_cube_transform * local_p);

    let seed_dim: vec3<i32> = textureDimensions(sdf_texture) - vec3<i32>(1);
    let seed_dim_f32: vec3<f32> = vec3<f32>(f32(seed_dim.x), f32(seed_dim.y), f32(seed_dim.z));
    let pixel_coords: vec3<f32> = ((cube_p.xyz + vec3<f32>(0.5)) /2.) * seed_dim_f32 * 2.;

    var i: i32 = i32(pixel_coords.x);
    var j: i32 = i32(pixel_coords.y);
    var k: i32 = i32(pixel_coords.z);


    i = clamp(i, 0, seed_dim.x);
    j = clamp(j, 0, seed_dim.y);
    k = clamp(k, 0, seed_dim.z);
    let x: f32 = clamp(pixel_coords.x - f32(i), 0., 1.);
    let y: f32 = clamp(pixel_coords.y - f32(j), 0., 1.);
    let z: f32 = clamp(pixel_coords.z - f32(k), 0., 1.);

    let f000: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i,
          j,
          k,
      )
    ,0).r;
    let f001: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i,
          j,
          k + 1,
      )
    ,0).r;
    let f010: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i,
          j + 1,
          k,
      )
    ,0).r;
    let f011: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i,
          j + 1,
          k + 1,
      )
    ,0).r;
    let f100: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i + 1,
          j,
          k,
      )
    ,0).r;
    let f101: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i + 1,
          j,
          k + 1,
      )
    ,0).r;
    let f110: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i + 1,
          j + 1,
          k,
      )
    ,0).r;
    let f111: f32 = textureLoad(sdf_texture,
      vec3<i32>(
          i + 1,
          j + 1,
          k + 1,
      )
    ,0).r;

    // Distance are built on the assumption that voxel size is one
    // we must correct that
    // If scale is non_uniform we can only provide a bound on the distance
    //
    // We attempt to extract the scale from the transform matrix
    // This extraction only works if
    // - Matrix is orthonormal
    // - Matrix is constructed as Scale*Rotate*Translate
    // - Matrix is column vectors
    // - Scale is all positive
    //   - Cannot do negative scale without more information
    //   - In this case it would be best to just apply it
    //     from uniform data
    let scale: vec3<f32> = vec3<f32>(
      length(u_sdf.world_transform.x.xyz),
      length(u_sdf.world_transform.y.xyz),
      length(u_sdf.world_transform.z.xyz),
    );
    let min_scale: f32 = min(scale.x, min(scale.y, scale.z));
    let internal_dist: f32 = min_scale *
          (
             f000*(1.-x)*(1.-y)*(1.-z)
            +f001*(1.-x)*(1.-y)*z
            +f010*(1.-x)*y     *(1.-z)
            +f011*(1.-x)*y     *z
            +f100*x     *(1.-y)*(1.-z)
            +f101*x     *(1.-y)*z
            +f110*x     *y     *(1.-z)
            +f111*x     *y     *z
          );
      let enclosing_box: f32 = sdBox(local_p.xyz, (u_sdf.grid_dimensions.xyz * 1.001)/vec3<f32>(2.));
      return max(enclosing_box, internal_dist);
}
// Get the material id at a point
//
// Using nearest neighbour
fn map_material(p: vec3<f32>) -> u32
{
  let local_p: vec4<f32> = (u_sdf.inv_world_transform * vec4<f32>(p, 1.));
  let cube_p: vec4<f32> = (u_sdf.inv_cube_transform * local_p);

  let seed_dim: vec3<i32> = textureDimensions(sdf_texture) - vec3<i32>(1);
  let seed_dim_f32: vec3<f32> = vec3<f32>(f32(seed_dim.x), f32(seed_dim.y), f32(seed_dim.z));
  let pixel_coords: vec3<f32> = ((cube_p.xyz + vec3<f32>(0.5)) /2.) * seed_dim_f32 * 2.;

  let nearest_pos: vec3<f32> = round(pixel_coords);
  let nearest_coord: vec3<i32> = vec3<i32>(i32(nearest_pos.x), i32(nearest_pos.y), i32(nearest_pos.z));

  return u32(textureLoad(sdf_texture, nearest_coord, 0).g);
}

// Surface gradient (is the normal)
fn map_normal (p: vec3<f32>) -> vec3<f32>
{
	let eps: vec3<f32> = u_sdf.voxel_dimensions.xyz * 0.05;

	return normalize
	(	vec3<f32>
		(	map(p + vec3<f32>(eps.x, 0., 0.)	) - map(p - vec3<f32>(eps.x, 0., 0.)),
			map(p + vec3<f32>(0., eps.y, 0.)	) - map(p - vec3<f32>(0., eps.y, 0.)),
			map(p + vec3<f32>(0., 0., eps.z)	) - map(p - vec3<f32>(0., 0., eps.z))
		)
	);
}

// Use pixel based cones to get the size of the pizel
fn pixel_radius(t: f32, direction: vec3<f32>, direction_x: vec3<f32>, direction_y: vec3<f32>) -> f32 {
  let dx: f32 = length(t*(direction_x-direction));
  let dy: f32 = length(t*(direction_y-direction));
  return length(vec2<f32>(dx, dy)) * 0.1;
}

{{ RAYTRACE_ALGO }}

{{ AO_ALGO }}

{{ SHADOWS_ALGO }}

{{ LIGHTING_ALGO }}


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
