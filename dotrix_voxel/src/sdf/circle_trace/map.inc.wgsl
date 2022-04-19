/// This function takes a position in world space, consults the
/// sdf and returns the distance to the surface
///
/// This varient works on SDFs stored in 3D textures


// SDF for a box, b is half box size
fn sdBox(p: vec3<f32>,b: vec3<f32>) -> f32
{
  let q: vec3<f32> = abs(p) - b;
  return length(max(q,vec3<f32>(0.0))) + min(max(q.x,max(q.y,q.z)),0.0);
}

// Given a location in the texture of the kind x = (-0.5,0.5), y = (-0.5,0.5), z = (-0.5,0.5)
//  i.e. a coordinate inside a box of unit length center at the origin
// Return the interpolated value
fn tri_linear_interpolation(cube_p: vec4<f32>) -> f32 {
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

  return (
           f000*(1.-x)*(1.-y)*(1.-z)
          +f001*(1.-x)*(1.-y)*z
          +f010*(1.-x)*y     *(1.-z)
          +f011*(1.-x)*y     *z
          +f100*x     *(1.-y)*(1.-z)
          +f101*x     *(1.-y)*z
          +f110*x     *y     *(1.-z)
          +f111*x     *y     *z
        );

}

// Get distance to surface from a point in world space
fn map(p: vec3<f32>) -> f32
{
    let local_p: vec4<f32> = (u_sdf.inv_world_transform * vec4<f32>(p, 1.));
    let cube_p: vec4<f32> = (u_sdf.inv_cube_transform * local_p);

    let internal_dist = tri_linear_interpolation(cube_p);

    // Distance are built on the assumption that voxel size is one
    // we must correct that
    // If scale is non_uniform we can only provide a bound on the distance
    //
    let scale: vec3<f32> = u_sdf.world_scale.xyz;
    let min_scale: f32 = min(abs(scale.x), min(abs(scale.y), abs(scale.z)));
    let dist = internal_dist * min_scale;
    // Enclosing box used for clipping
    let enclosing_box: f32 = sdBox(local_p.xyz, (u_sdf.grid_dimensions.xyz * 1.00)/vec3<f32>(2.)) * min_scale;

    // Return intersection of voxel sdf and enclosing (clipping) box
    return max(enclosing_box, dist);
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
	let eps: vec3<f32> = abs(u_sdf.world_scale.xyz) * 0.05;

	return normalize
	(	vec3<f32>
		(	map(p + vec3<f32>(eps.x, 0., 0.)	) - map(p - vec3<f32>(eps.x, 0., 0.)),
			map(p + vec3<f32>(0., eps.y, 0.)	) - map(p - vec3<f32>(0., eps.y, 0.)),
			map(p + vec3<f32>(0., 0., eps.z)	) - map(p - vec3<f32>(0., 0., eps.z))
		)
	);
}
