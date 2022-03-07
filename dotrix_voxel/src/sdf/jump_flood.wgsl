struct Intermediate {
  distances: array<f32>;
};

var<storage,read_write> intermediate: Intermediate;

fn value(index: u32) -> f32 {

}

fn index(coord: vec3<i32>) -> u32 {

}

fn value_at(coord: vec3<i32>) -> f32 {
  return value(index(coord));
}

// For a given pixel get its gradient in world space
fn gradient(coord: vec3<i32>) -> vec3<f32> {

}

fn is_out_of_bounds(coord: vec3<i32>) -> bool {

}

fn is_outside(coord: vec3<i32>) -> bool {
  return select(true, false, value_at(coord) >= 0.);
}

fn is_sameside(reference: bool, coord: vec3<i32>) {
  if (is_out_of_bounds(coord)) {
    return true;
  }
  return (is_outside(coord) == reference);
}

// Write location of current nearest seed for this pixel
// Written into the RGB channels
fn set_value(value: vec3<f32>, coord: vec3<i32>) {

}

/// Marks a cell as being invalid with no known data yet
fn set_invalid_at(coord: vec3<i32>) {
  set_value(vec3<f32>(1.0e999, 1.0e999, 1.0e999), coord);
}

/// Checks if it is has an invalid seed location
fn is_invalid(value: vec3<f32>) -> bool {
  return value == vec3<f32>(1.0e999, 1.0e999, 1.0e999);
}

/// Checks if pixel has an invalid seed location
fn is_invalid_at(coord: vec3<i32>) -> bool {
  return is_invalid(value_at(coord));
}

// For a given pixel tries to read the seed value,
// then compares to a reference seed distance
// and identifies if it is a better seed distance
fn is_seed_better(coord: vec3<i32>, reference_seed: vec3<f32>, origin: vec3<f32>) -> bool {
  if (is_out_of_bounds(coord)) {
    return false;
  }
  let new_seed: vec3<f32> = value_at(coord);
  if (is_invalid(new_seed)) {
    return false;
  }
  if (is_invalid(reference_seed)) {
    return true;
  }
  return distance(new_seed, origin) < distance(reference_seed, origin);
}

// For a given pixel get its origin in world space
fn origin(coord: vec3<i32>) -> vec3<f32> {

}

// Write final distance to isosurface
fn write_output(coord: vec3<i32>, dist: f32) {
}

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    // We calculate the distance from o
    // to the isosurface as represented
    // by the densities on x
    //  ---x---
    // |   |   |
    // x---o---x
    // |   |   |
    //  ---x---
    // Using the approximate jump flooding algorithm
    //

    let pixel_loc: vec3<i32> = vec3<i32>(
      i32(global_invocation_id[0]),
      i32(global_invocation_id[1]),
      i32(global_invocation_id[2]),
    );

    // First clear init cells with a null value
    let idx: u32 = index(pixel_loc);
    intermediate.distances[idx] = 1e99;

    // Now place seeds
    // If the density of o±1 crosses the root
    // then the isosurface is close by
    //
    let center_side: bool = is_outside(pixel_loc);
    var is_seed: bool = (
         !is_sameside(center_side, vec3<i32>(pixel_loc[0] + 1, pixel_loc[1], pixel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(pixel_loc[0] - 1, pixel_loc[1], pixel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(pixel_loc[0], pixel_loc[1] + 1, pixel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(pixel_loc[0], pixel_loc[1] - 1, pixel_loc[2]))
      || !is_sameside(center_side, vec3<i32>(pixel_loc[0], pixel_loc[1], pixel_loc[2] + 1))
      || !is_sameside(center_side, vec3<i32>(pixel_loc[0], pixel_loc[1], pixel_loc[2] - 1))
    );

    let pixel_origin: vec3<f32> = origin(pixel_loc);

    if (is_seed) {
      // We use the numerical gradient to approximate
      // it's distance to the isosurface
      //
      // This uses linear approximation
      // TODO: Test quadratic approximation

      // Gradient is the rate of change and the direction towards nearest isosurface
      let m: vec3<f32> = gradient(pixel_loc);
      let direction: vec3<f32> = normalize(m);

      // Just need to know how far to go in that direction
      // We assume linear where travelling one unit length of the gradient's direction
      // will reduce the value of the isosurface by m
      // How many m's doing we need?
      let approximate_distance: f32 = (value_at(pixel_loc) - 0.) / m;

      // Approximate location of nearest isosurface is as follows
      let approximate_location: vec3<f32> = pixel_origin + direction * approximate_distance;

      // Write the location of the closest seed into the pixel
      set_value(approximate_location, pixel_loc);
    }

    // Wait for seed to be set everywhere
    storageBarrier();
    workgroupBarrier();

    // Jump Flood Algorithm:
    //
    // n = number of pixels in largest dimension
    // Loop over ceil(log2(n)) times over the image i=[1, ceil(log2(n))]
    //  For n = 8, k = n/2, n/4, n/8
    //  For n = 16, k= n/2, n/4, n/8, n/16
    //  For n = 17, k= n/2, n/4, n/8, n/16, n/32
    //  k = n/(2^(i))
    let n: u32 = 8;
    var i: u32 = 1u;
    var k: u32 = n / pow(2u, i);

    while (k >= 1u) {
      var best_seed: vec3<f32> = value_at(pixel_loc);

      for (var dx = -1, dx<=1; dx = dx + 1) {
        for (var dy = -1, dy<=1; dy = dy + 1) {
          for (var dz = -1, dz<=1; dz = dz + 1) {
            if (dx == 0 && dy == 0 && dz == 0) {
              continue;
            }
            let check_coord: vec3<i32> = vec3<i32>(
              pixel_loc[0] - dx,
              pixel_loc[1] - dy,
              pixel_loc[2] - dz,
            );
            if (is_seed_better(check_coord, best_seed, pixel_origin))
              best_seed = value_at(check_coord);
            }
          }
        }
      }

      i =  i + 1u;
      k = n / pow(2u, i);

      // Wait for round to finish
      storageBarrier();
      workgroupBarrier();
    }

    // Jump flood complete write distance map
    write_output(pixel_loc, distance(pixel_origin, value_at(pixel_loc)));
}
