// This compute takes a voxel
// and write the init seed location for the
// jump flood algorithm
//
//

let ISO_SURFACE: f32 = 0.5;

// The density for the voxel should be stored in the r channel
[[group(0), binding(0)]]
var voxels: texture_3d<u32>;
// The rgb channels will be set to contain the nearest seed location
// of voxels that cross the root
[[group(0), binding(1)]]
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
fn voxel_value(seed_coord: vec3<f32>) -> f32 {
  let voxel_dim: vec3<i32> = textureDimensions(voxels) - vec3<i32>(1);
  let voxel_dim_f32: vec3<f32> = vec3<f32>(f32(voxel_dim.x), f32(voxel_dim.y), f32(voxel_dim.z));
  let seed_dim: vec3<i32> = textureDimensions(init_seeds) - vec3<i32>(1);
  let seed_dim_f32: vec3<f32> = vec3<f32>(f32(seed_dim.x), f32(seed_dim.y), f32(seed_dim.z));

  let coord: vec3<f32> = seed_coord / seed_dim_f32 * voxel_dim_f32;
  let i: i32 = clamp(i32(coord.x), 0, voxel_dim.x);
  let j: i32 = clamp(i32(coord.y), 0, voxel_dim.y);
  let k: i32 = clamp(i32(coord.z), 0, voxel_dim.z);
  let x: f32 = clamp(coord.x - f32(i), 0., 1.);
  let y: f32 = clamp(coord.y - f32(j), 0., 1.);
  let z: f32 = clamp(coord.z - f32(k), 0., 1.);

  let f000: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i,
        j,
        k,
    )
  ,0).r);
  let f001: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i,
        j,
        k + 1,
    )
  ,0).r);
  let f010: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i,
        j + 1,
        k,
    )
  ,0).r);
  let f011: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i,
        j + 1,
        k + 1,
    )
  ,0).r);
  let f100: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i + 1,
        j,
        k,
    )
  ,0).r);
  let f101: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i + 1,
        j,
        k + 1,
    )
  ,0).r);
  let f110: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i + 1,
        j + 1,
        k,
    )
  ,0).r);
  let f111: f32 = f32(textureLoad(voxels,
    vec3<i32>(
        i + 1,
        j + 1,
        k + 1,
    )
  ,0).r);

  return
        (
           f000*(1.-x)*(1.-y)*(1.-z)
          +f001*(1.-x)*(1.-y)*z
          +f010*(1.-x)*y     *(1.-z)
          +f011*(1.-x)*y     *z
          +f100*x     *(1.-y)*(1.-z)
          +f101*x     *(1.-y)*z
          +f110*x     *y     *(1.-z)
          +f111*x     *y     *z
        ) - ISO_SURFACE;
}

// For a given pixel get its pos in local seed space
fn seed_coord(seed_pixel: vec3<i32>) -> vec3<f32> {
  return vec3<f32>(f32(seed_pixel[0]),f32(seed_pixel[1]),f32(seed_pixel[2]));
}

fn approximate_root(seed_pixel_a: vec3<i32>, delta: vec3<i32>, current_best: ptr<function,f32>) {
  let seed_pixel_b: vec3<i32> = seed_pixel_a + delta;
  let seed_pos_a: vec3<f32> = seed_coord(seed_pixel_a);
  let seed_pos_b: vec3<f32> = seed_coord(seed_pixel_b);
  let voxel_value_a = voxel_value(seed_pos_a);
  let voxel_value_b = voxel_value(seed_pos_b);
  let weight: f32 = voxel_value_a/(voxel_value_a-voxel_value_b);

  if (weight<0. || weight > 1.) {
    return;
  }

  let root_pos: vec3<f32> = mix(seed_pos_a, seed_pos_b, weight);
  let dist: f32 = distance(seed_pos_a, root_pos);
  if (dist < *current_best) {
    set_seed_at(root_pos, seed_pixel_a);
    *current_best = dist;
  }
  return;
}

[[stage(compute), workgroup_size(8,8,4)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let seed_pixel: vec3<i32> = vec3<i32>(
      i32(global_invocation_id[0]),
      i32(global_invocation_id[1]),
      i32(global_invocation_id[2]),
    );

    var best_value: f32 = 4000.;
    set_seed_invalid_at(seed_pixel);
    approximate_root(seed_pixel, vec3<i32>(-1,-1,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0,-1,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1,-1,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>(-1, 0,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0, 0,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1, 0,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>(-1, 1,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0, 1,-1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1, 1,-1), &best_value);

    approximate_root(seed_pixel, vec3<i32>(-1,-1, 0), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0,-1, 0), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1,-1, 0), &best_value);
    approximate_root(seed_pixel, vec3<i32>(-1, 0, 0), &best_value);
    // approximate_root(seed_pixel, vec3<i32>( 0, 0, 0), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1, 0, 0), &best_value);
    approximate_root(seed_pixel, vec3<i32>(-1, 1, 0), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0, 1, 0), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1, 1, 0), &best_value);

    approximate_root(seed_pixel, vec3<i32>(-1,-1, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0,-1, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1,-1, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>(-1, 0, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0, 0, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1, 0, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>(-1, 1, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 0, 1, 1), &best_value);
    approximate_root(seed_pixel, vec3<i32>( 1, 1, 1), &best_value);
}
