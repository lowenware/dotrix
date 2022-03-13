// Takes a jump flood result and computes the DF
//

// The density for the voxel should be stored in the r channel
// The material should be in the g channel
[[group(0), binding(0)]]
var voxels: texture_3d<f32>;

// The rgb channels will be set to contain the nearest seed location
[[group(0), binding(1)]]
var jump_flood: texture_3d<f32>;


// The r channel will contain the DF
// The g channel will copy the material ID from the voxel
[[group(0), binding(2)]]
var sdf: texture_storage_3d<rg32float,write>;


// For a given voxel get its origin in local space
fn origin(coord: vec3<i32>) -> vec3<f32> {
  return vec3<f32>(0.,0.,0.) + vec3<f32>(f32(coord[0]),f32(coord[1]),f32(coord[2])) * vec3<f32>(1.,1.,1.);
}

// Get the seed position from the jump flood
fn seed_position(coord: vec3<i32>) -> vec3<f32> {
  return textureLoad(jump_flood, coord, 0).rgb;
}

// Get the seed position from the jump flood
fn material(coord: vec3<i32>) -> f32 {
  return (textureLoad(voxels, coord, 0).g) * 256. ;
}

// Get the seed position from the jump flood
fn save_sdf(coord: vec3<i32>, dist: f32, material_id: f32) {
  textureStore(sdf, coord, vec4<f32>(dist, material_id, 0., 0.));
}

[[stage(compute), workgroup_size(8,8,4)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let origin_coord: vec3<i32> = vec3<i32>(
    i32(global_invocation_id[0]),
    i32(global_invocation_id[1]),
    i32(global_invocation_id[2]),
  );

  let origin_pos: vec3<f32> = origin(origin_coord);
  let seed_pos: vec3<f32> = seed_position(origin_coord);
  let dist: f32 = distance(origin_pos, seed_pos);
  let material_id: f32 = material(origin_coord);

  save_sdf(origin_coord, dist, material_id);
}
