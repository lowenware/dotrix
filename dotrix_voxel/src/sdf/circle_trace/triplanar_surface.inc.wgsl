//! This get the material properties at a point on the surface
//!
//! This uses triplanar rendering which means we blend a
//!   up front and right texture according to the normal
//!
//! Triplanar math is from https://www.volume-gfx.com/volume-rendering/triplanar-texturing/
//!
//! Normal math is from https://bgolus.medium.com/normal-mapping-for-a-triplanar-shader-10bf39dca05a
//!

struct Surface {
  normal: vec3<f32>;
  albedo: vec3<f32>;
  roughness: f32;
  metallic: f32;
  ao: f32;
};

fn average(input: vec4<f32>) -> f32 {
  return (input.x + input.y + input.z + input.w) / 4.;
}

// Get the surface using triplanar texturing
// This is the final materials values at a point
fn get_surface(world_pos: vec3<f32>, world_normal: vec3<f32>, material_id: u32, ddx: vec3<f32>, ddy: vec3<f32>) -> Surface {
  // Convert normal to local space
  let local_normal: vec3<f32> = normalize((u_sdf.inv_normal_transform * vec4<f32>(world_normal, 0.)).xyz);
  // Convert position to local space without scale
  // This is so that materials don't stretch over the voxel and a large voxel will have
  // more repeating material
  // TODO: Make this configurable
  let local_pos: vec3<f32> = (u_sdf.inv_world_transform * vec4<f32>(world_pos, 1.)).xyz * u_sdf.world_scale.xyz;

  // Tri planar parameters
  let delta: f32 = 1e-3; // Controls plateua where it is considered to be one plane
  let m: f32 = 1.; // Controls triplanar transision speed

  // Tile over 1x1x1 space
  let x: f32 = fract(local_pos.x);
  let y: f32 = fract(local_pos.y);
  let z: f32 = fract(local_pos.z);


  // Triplanar weights
  let b: vec3<f32> = normalize(vec3<f32>(
    pow(max(abs(local_normal.x) - delta, 0.), m),
    pow(max(abs(local_normal.y) - delta, 0.), m),
    pow(max(abs(local_normal.z) - delta, 0.), m)
  ));

  var material_right: Material = u_materials.materials[material_id];
  var material_top: Material = u_materials.materials[material_id];
  var material_front: Material = u_materials.materials[material_id];

  var out: Surface;

  // Albedo
  var albedo_right: vec4<f32>;
  var albedo_top: vec4<f32>;
  var albedo_front: vec4<f32>;
  if (material_right.albedo_id < 0) {
    albedo_right = material_right.albedo;
  } else {
    albedo_right = textureSampleGrad(material_textures, r_sampler, vec2<f32>(y,z), material_right.albedo_id, ddx.yz, ddy.yz);
  }
  if (material_top.albedo_id < 0) {
    albedo_top = material_top.albedo;
  } else {
    albedo_top = textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,z), material_top.albedo_id, ddx.xz, ddy.xz);
  }
  if (material_front.albedo_id < 0) {
    albedo_front = material_front.albedo;
  } else {
    albedo_front = textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,y), material_front.albedo_id, ddx.xy, ddy.xy);
  }
  out.albedo = albedo_right.xyz * b.x + albedo_top.xyz * b.y + albedo_front.xyz * b.z;

  // Roughness
  var roughness_right: f32;
  var roughness_top: f32;
  var roughness_front: f32;
  if (material_right.roughness_id < 0) {
    roughness_right = material_right.roughness;
  } else {
    roughness_right = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(y,z), material_right.roughness_id, ddx.yz, ddy.yz));
  }
  if (material_top.roughness_id < 0) {
    roughness_top = material_top.roughness;
  } else {
    roughness_top = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,z), material_top.roughness_id, ddx.xz, ddy.xz));
  }
  if (material_front.roughness_id < 0) {
    roughness_front = material_front.roughness;
  } else {
    roughness_front = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,y), material_front.roughness_id, ddx.xy, ddy.xy));
  }
  out.roughness = roughness_right * b.x + roughness_top * b.y + roughness_front * b.z;

  // Metallic
  var metallic_right: f32;
  var metallic_top: f32;
  var metallic_front: f32;
  if (material_right.metallic_id < 0) {
    metallic_right = material_right.metallic;
  } else {
    metallic_right = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(y,z), material_right.metallic_id, ddx.yz, ddy.yz));
  }
  if (material_top.metallic_id < 0) {
    metallic_top = material_top.metallic;
  } else {
    metallic_top = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,z), material_top.metallic_id, ddx.xz, ddy.xz));
  }
  if (material_front.metallic_id < 0) {
    metallic_front = material_front.metallic;
  } else {
    metallic_front = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,y), material_front.metallic_id, ddx.xy, ddy.xy));
  }
  out.metallic = metallic_right * b.x + metallic_top * b.y + metallic_front * b.z;

  // Ao
  var ao_right: f32;
  var ao_top: f32;
  var ao_front: f32;
  if (material_right.ao_id < 0) {
    ao_right = material_right.ao;
  } else {
    ao_right = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(y,z), material_right.ao_id, ddx.yz, ddy.yz));
  }
  if (material_top.ao_id < 0) {
    ao_top = material_top.ao;
  } else {
    ao_top = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,z), material_top.ao_id, ddx.xz, ddy.xz));
  }
  if (material_front.ao_id < 0) {
    ao_front = material_front.ao;
  } else {
    ao_front = average(textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,y), material_front.ao_id, ddx.xy, ddy.xy));
  }
  out.ao = ao_right * b.x + ao_top * b.y + ao_front * b.z;

  // Normal Map
  //
  // Using Whiteout blend normal map in local space
  var normal_right: vec3<f32>;
  var normal_top: vec3<f32>;
  var normal_front: vec3<f32>;
  if (material_right.normal_id < 0) {
    normal_right = vec3<f32>(0.,0.,1.);
  } else {
    normal_right = (textureSampleGrad(material_textures, r_sampler, vec2<f32>(y,z), material_right.normal_id, ddx.yz, ddy.yz).xyz - 0.5) * 2.;
  }

  if (material_top.normal_id < 0) {
    normal_right = vec3<f32>(0.,0.,1.);
  } else {
    normal_top = (textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,z), material_top.normal_id, ddx.xz, ddy.xz).xyz - 0.5) * 2.;
  }
  if (material_front.normal_id < 0) {
    normal_right = vec3<f32>(0.,0.,1.);
  } else {
    normal_front = (textureSampleGrad(material_textures, r_sampler, vec2<f32>(x,y), material_front.normal_id, ddx.xy, ddy.xy).xyz - 0.5) * 2.;
  }

  // Apply whiteout blend
  normal_right = vec3<f32>(normal_right.x+local_normal.z, normal_right.y+local_normal.y, abs(normal_right.z) * local_normal.x);
  normal_top = vec3<f32>(normal_top.x+local_normal.x, normal_top.y+local_normal.z, abs(normal_top.z) * local_normal.y);
  normal_front = vec3<f32>(normal_front.x+local_normal.x, normal_front.y+local_normal.y, abs(normal_front.z) * local_normal.z);

  // Swizzel into one normal
  let new_local_normal = normal_right.zyx * b.x + normal_top.xzy * b.y + normal_front.xyz * b.z;
  // let new_local_normal = normal_right.xyz * b.x + normal_top.xyz * b.y + normal_front.xyz * b.z;

  // Convert back to world space
  out.normal = normalize((u_sdf.normal_transform * vec4<f32>(new_local_normal, 0.)).xyz);

  return out;
}
