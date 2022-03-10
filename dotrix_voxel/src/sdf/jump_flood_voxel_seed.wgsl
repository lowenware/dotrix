// This compute takes a voxel
// and write the init seed location for the
// jump flood algorithm
//

struct Data {
  // Voxel origin of voxel at the 0,0,0 position in world space
  origin: vec3<f32>;
  // Dimensions of a single voxel
  dimensions: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> data: Data;

// The density for the voxel should be stored in the r channel
[[group(0), binding(1)]]
var voxels: texture_3d<f32>;
// The rgb channels will be set to contain the nearest seed location
// of voxels that cross the root
[[group(0), binding(2)]]
var init_seeds: texture_storage_3d<rgba32float,write>;

// Write location of current nearest seed for this pixel
// Written into the RGB channels
fn set_seed_at(value: vec3<f32>, coord: vec3<i32>) {
  textureStore(init_seeds, coord, vec4<f32>(value, 0.));
}

/// Marks a cell as being invalid with no known data yet
fn set_seed_invalid_at(coord: vec3<i32>) {
  textureStore(init_seeds, coord, vec4<f32>(0.,0.,0., -1.));
}

// Get density from voxel
fn voxel_value_at(coord: vec3<i32>) -> f32 {
  return f32(textureLoad(voxels, coord, 0).x);
}

// For a given voxel get its origin in world space
fn origin(coord: vec3<i32>) -> vec3<f32> {
  return data.origin + vec3<f32>(f32(coord[0]),f32(coord[1]),f32(coord[2])) * data.dimensions;
}

fn is_out_of_bounds(coord: vec3<i32>) -> bool {
  return (
       coord[0] < 0
    || coord[1] < 0
    || coord[2] < 0
    || coord[0] >= i32(data.dimensions[0])
    || coord[1] >= i32(data.dimensions[1])
    || coord[2] >= i32(data.dimensions[2])
  );
}

// For a given voxel get its gradient in world space
fn gradient(coord: vec3<i32>) -> vec3<f32> {
  let x0: vec3<i32> = vec3<i32>(
    coord[0] + 1,
    coord[1],
    coord[2]
  );
  let x_plus: vec3<i32> = select(x0, coord, is_out_of_bounds(x0));

  let x1: vec3<i32> = vec3<i32>(
    coord[0] - 1,
    coord[1],
    coord[2]
  );
  let x_minus: vec3<i32> = select(x1, coord, is_out_of_bounds(x1));

  let y0: vec3<i32> = vec3<i32>(
    coord[0],
    coord[1] + 1,
    coord[2]
  );
  let y_plus: vec3<i32> = select(coord, y0, is_out_of_bounds(y0));

  let y1: vec3<i32> = vec3<i32>(
    coord[0],
    coord[1] - 1,
    coord[2]
  );
  let y_minus: vec3<i32> = select(coord, y1, is_out_of_bounds(y1));

  let z0: vec3<i32> = vec3<i32>(
    coord[0],
    coord[1],
    coord[2] + 1
  );
  let z_plus: vec3<i32> = select(coord, z0, is_out_of_bounds(z0));

  let z1: vec3<i32> = vec3<i32>(
    coord[0],
    coord[1],
    coord[2] - 1
  );
  let z_minus: vec3<i32> = select(coord, z1, is_out_of_bounds(z1));


  return vec3<f32>(
    (voxel_value_at(x_plus) - voxel_value_at(x_minus))/(origin(x_plus)[0] - origin(x_minus)[0]),
    (voxel_value_at(y_plus) - voxel_value_at(y_minus))/(origin(y_plus)[1] - origin(y_minus)[1]),
    (voxel_value_at(z_plus) - voxel_value_at(z_minus))/(origin(z_plus)[2] - origin(z_minus)[2]),
  );
}

fn is_outside(coord: vec3<i32>) -> bool {
  return select(true, false, voxel_value_at(coord) >= 0.);
}

fn is_sameside(reference: bool, coord: vec3<i32>) -> bool {
  if (is_out_of_bounds(coord)) {
    return true;
  }
  return (is_outside(coord) == reference);
}

[[stage(compute), workgroup_size(8,8,4)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    // We calculate the distance from o
    // to the isosurface as represented
    // by the densities on x
    //  ---x---
    // |   |   |
    // x---o---x
    // |   |   |
    //  ---x---
    //

    let voxel_loc: vec3<i32> = vec3<i32>(
      i32(global_invocation_id[0]),
      i32(global_invocation_id[1]),
      i32(global_invocation_id[2]),
    );

    // If the density of origin±1 crosses the root
    // then the isosurface is close by
    //
    let center_side: bool = is_outside(voxel_loc);
    var is_seed: bool = (
         !is_sameside(center_side, vec3<i32>(voxel_loc[0] + 1, voxel_loc[1], voxel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(voxel_loc[0] - 1, voxel_loc[1], voxel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(voxel_loc[0], voxel_loc[1] + 1, voxel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(voxel_loc[0], voxel_loc[1] - 1, voxel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(voxel_loc[0], voxel_loc[1], voxel_loc[2] + 1))
      || !is_sameside(center_side, vec3<i32>(voxel_loc[0], voxel_loc[1], voxel_loc[2] - 1))
    );

    let pixel_origin: vec3<f32> = origin(voxel_loc);

    if (is_seed) {
      // We use the numerical gradient to approximate
      // it's distance to the isosurface
      //
      // This uses linear approximation
      // TODO: Test quadratic approximation

      // Gradient is the rate of change and the direction towards nearest isosurface
      let m: vec3<f32> = gradient(voxel_loc);
      let direction: vec3<f32> = normalize(m);

      // Just need to know how far to go in that direction
      // We assume linear where travelling one unit length of the gradient's direction
      // will reduce the value of the isosurface by m
      // How many m's doing we need?
      let v0: f32 = voxel_value_at(voxel_loc);
      let V0: vec3<f32> = vec3<f32>(v0);
      let approximate_distance: vec3<f32> = V0 / m;

      // Approximate location of nearest isosurface is as follows
      let approximate_location: vec3<f32> = pixel_origin + direction * approximate_distance;

      // Write the location of the closest seed into the pixel
      set_seed_at(approximate_location, voxel_loc);
    } else {
      set_seed_invalid_at(voxel_loc);
    }
}
