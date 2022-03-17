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
  let tex_dim: vec3<i32> = textureDimensions(init_seeds);
  return (
       coord[0] < 0
    || coord[1] < 0
    || coord[2] < 0
    || coord[0] >= tex_dim[0]
    || coord[1] >= tex_dim[1]
    || coord[2] >= tex_dim[2]
  );
}

// For a given voxel get its origin in local seed space
fn origin(coord: vec3<i32>) -> vec3<f32> {
  return vec3<f32>(f32(coord[0]),f32(coord[1]),f32(coord[2]));
}

// For a given pixel tries to read the seed value,
// then compares to a reference seed distance
// and identifies if it is a better seed distance
fn is_seed_better(origin_coord: vec3<i32>, delta: vec3<i32>, best_seed: ptr<function, vec3<f32>>) {
  let coord: vec3<i32> = origin_coord + delta;
  if (is_out_of_bounds(coord)) {
    return;
  }
  if (is_invalid_at(coord)) {
    return;
  }
  let new_seed: vec3<f32> = value_at(coord);
  let origin_pos: vec3<f32> = origin(origin_coord);
  if (is_invalid_at(origin_coord) || distance(new_seed, origin_pos) < distance(*best_seed, origin_pos)) {
    *best_seed = new_seed;
  }
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
    //  If seed found in neighbouring cell is better than current
    //  then use that one
    //
    //  This compute does only a single value of k
    //  it must be enqueued multiple times to complete the jump flood
    //  with a ping pong style buffer
    //
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

    is_seed_better(origin_coord, vec3<i32>( 0,-k,-k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k,-k,-k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>(-k, 0,-k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( 0, 0,-k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k, 0,-k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>(-k, k,-k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( 0, k,-k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k, k,-k), &best_seed);

    is_seed_better(origin_coord, vec3<i32>(-k,-k, 0), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( 0,-k, 0), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k,-k, 0), &best_seed);
    is_seed_better(origin_coord, vec3<i32>(-k, 0, 0), &best_seed);
    // is_seed_better(origin_coord, vec3<i32>( 0, 0, 0), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k, 0, 0), &best_seed);
    is_seed_better(origin_coord, vec3<i32>(-k, k, 0), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( 0, k, 0), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k, k, 0), &best_seed);

    is_seed_better(origin_coord, vec3<i32>(-k,-k, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( 0,-k, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k,-k, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>(-k, 0, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( 0, 0, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k, 0, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>(-k, k, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( 0, k, k), &best_seed);
    is_seed_better(origin_coord, vec3<i32>( k, k, k), &best_seed);

    set_value_at(origin_coord, best_seed);
}
