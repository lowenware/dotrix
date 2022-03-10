// This compute applies the jump flood algorithm
//
// The algorithm is a fast (approximate) method
// for voronoi diagrams and distance transforms
//
// It is O(log(n))
//
// This algorihm should be called as a ping pong buffer
// Each call should decrease k until k==1
// while swapping the input/output texture buffers
//
// Texture buffers are of kind:
// r,g,b,a where r,g,b are the xyz values of the nearest seed
// and a is used as a flag for invalid seed when a<0

struct Data {
  // Voxel origin of voxel at the 0,0,0 position in world space
  origin: vec3<f32>;
  // Dimensions of a single voxel
  dimensions: vec3<f32>;
  // The current iterations step size must be >=1
  k: i32;
};

[[group(0), binding(0)]]
var<uniform> data: Data;

// The previous run seed values stored in each pixel
[[group(0), binding(1)]]
var init_seeds: texture_3d<f32>;

// The next run's seed values
[[group(0), binding(2)]]
var out_seeds: texture_storage_3d<rgba32float,write>;

fn value_at(coord: vec3<i32>) -> vec3<f32> {
  return textureLoad(init_seeds, coord, 0).rgb;
}

// Write location of current nearest seed for this pixel
// Written into the RGB channels
fn set_value_at(coord: vec3<i32>, value: vec3<f32>) {
  textureStore(out_seeds, coord, vec4<f32>(value, 0.));
}

/// Checks if it is has an invalid seed location
fn is_invalid_at(coord: vec3<i32>) -> bool {
  return textureLoad(init_seeds, coord, 0).a < 0.;
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

// For a given voxel get its origin in world space
fn origin(coord: vec3<i32>) -> vec3<f32> {
  return data.origin + vec3<f32>(f32(coord[0]),f32(coord[1]),f32(coord[2])) * data.dimensions;
}

// For a given pixel tries to read the seed value,
// then compares to a reference seed distance
// and identifies if it is a better seed distance
fn is_seed_better(coord: vec3<i32>, origin_coord: vec3<i32>) -> bool {
  if (is_out_of_bounds(coord)) {
    return false;
  }
  if (is_invalid_at(coord)) {
    return false;
  }
  if (is_invalid_at(origin_coord)) {
    return true;
  }

  let new_seed: vec3<f32> = value_at(coord);
  let reference_seed: vec3<f32> = value_at(origin_coord);
  let origin_coord: vec3<f32> = origin(origin_coord);
  return distance(new_seed, origin_coord) < distance(reference_seed, origin_coord);
}


[[stage(compute), workgroup_size(8,8,4)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    // Jump Flood Algorithm:
    //
    // n = number of pixels in largest dimension
    // Loop over ceil(log2(n)) times over the image i=[1, ceil(log2(n))]
    //  For n = 8, k = n/2, n/4, n/8
    //  For n = 16, k= n/2, n/4, n/8, n/16
    //  For n = 17, k= n/2, n/4, n/8, n/16, n/32
    //  k = n/(2^(i))
    //
    //  Look in all seeds at location of origin±k
    //  If seed found in neighbouring cell is better than current use that one
    var k: i32 = data.k;
    if (k<1) {
      return;
    }

    let origin_coord: vec3<i32> = vec3<i32>(
      i32(global_invocation_id[0]),
      i32(global_invocation_id[1]),
      i32(global_invocation_id[2]),
    );

    var best_seed: vec3<f32> = value_at(origin_coord);

    for (var dx: i32 = -1; dx<=1; dx = dx + 1) {
      for (var dy: i32 = -1; dy<=1; dy = dy + 1) {
        for (var dz: i32 = -1; dz<=1; dz = dz + 1) {
          if (dx == 0 && dy == 0 && dz == 0) {
            continue;
          }
          let check_coord: vec3<i32> = vec3<i32>(
            origin_coord[0] - dx*k,
            origin_coord[1] - dy*k,
            origin_coord[2] - dz*k,
          );
          if (is_seed_better(check_coord, origin_coord)) {
            best_seed = value_at(check_coord);
          }
        }
      }
    }

    set_value_at(origin_coord, best_seed);
}
