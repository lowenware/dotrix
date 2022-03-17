// Takes a jump flood result and computes the SDF
//
// Jump flood has seeds in jump flood space
// SDF should have distance in voxel space
// Conversion is done where appropiate
//

let ISO_SURFACE: f32 = 0.5;

// The density for the voxel should be stored in the r channel
// The material should be in the g channel
[[group(0), binding(0)]]
var voxels: texture_3d<u32>;

// The rgb channels will be set to contain the nearest seed location
[[group(0), binding(1)]]
var jump_flood: texture_3d<f32>;


// The r channel will contain the DF
// The g channel will copy the material ID from the voxel
[[group(0), binding(2)]]
var sdf: texture_storage_3d<rg32float,write>;


// For a given jump_flood coord get its coord in local voxel space
fn seed_coord_to_voxel_pos(coord: vec3<i32>) -> vec3<f32> {
  let seed_dim: vec3<i32> = textureDimensions(jump_flood) - vec3<i32>(1);
  let voxel_dim: vec3<i32> = textureDimensions(voxels) - vec3<i32>(1);
  let seed_dim_f32: vec3<f32> = vec3<f32>(f32(seed_dim.x), f32(seed_dim.y), f32(seed_dim.z));
  let voxel_dim_f32: vec3<f32> = vec3<f32>(f32(voxel_dim.x), f32(voxel_dim.y), f32(voxel_dim.z));
  return vec3<f32>(f32(coord[0]),f32(coord[1]),f32(coord[2])) * voxel_dim_f32 / seed_dim_f32;
}

// Get the seed position from the jump flood in local voxel space
fn seed_position(coord: vec3<i32>) -> vec3<f32> {
  let seed_dim: vec3<i32> = textureDimensions(jump_flood) - vec3<i32>(1);
  let voxel_dim: vec3<i32> = textureDimensions(voxels) - vec3<i32>(1);
  let seed_dim_f32: vec3<f32> = vec3<f32>(f32(seed_dim.x), f32(seed_dim.y), f32(seed_dim.z));
  let voxel_dim_f32: vec3<f32> = vec3<f32>(f32(voxel_dim.x), f32(voxel_dim.y), f32(voxel_dim.z));
  return textureLoad(jump_flood, coord, 0).rgb * voxel_dim_f32 / seed_dim_f32;
}

// Get density from voxel using voxel coord
// - Used to work out if a point is inside or not
fn voxel_value(coord: vec3<f32>) -> f32 {
  let voxel_dim: vec3<i32> = textureDimensions(voxels) - vec3<i32>(1);
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

// Get the material from the voxel
// using nearest neighbour
fn material(coord: vec3<f32>) -> u32 {
  let nearest_pos: vec3<f32> = round(coord);
  let nearest_coord: vec3<i32> = vec3<i32>(i32(nearest_pos.x), i32(nearest_pos.y), i32(nearest_pos.z));
  return textureLoad(voxels, nearest_coord, 0).g;
}

// Save the distance field value into the SDF texture
fn save_sdf(coord: vec3<i32>, dist: f32, material_id: f32) {
  textureStore(sdf, coord, vec4<f32>(dist, material_id, 0., 0.));
}

// Check if a position in inside or outside the surface
fn is_outside(pos_voxel: vec3<f32>) -> bool {
  // return true;
  return voxel_value(pos_voxel) > 1e-4;
}

[[stage(compute), workgroup_size(8,8,4)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let coord_seed: vec3<i32> = vec3<i32>(
    i32(global_invocation_id[0]),
    i32(global_invocation_id[1]),
    i32(global_invocation_id[2]),
  );

  let pos_voxel: vec3<f32> = seed_coord_to_voxel_pos(coord_seed);
  let seed_pos_voxel: vec3<f32> = seed_position(coord_seed);
  let dist: f32 = select(1.,-1., is_outside(pos_voxel)) * distance(pos_voxel, seed_pos_voxel);
  let material_id: f32 = f32(material(pos_voxel));

  save_sdf(coord_seed, dist, material_id);
}
