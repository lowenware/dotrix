//! This get the material properties at a point on the surface
//!
//! This uses trilinear rendering which means we blend a
//!   up front and right texture according to the normal
//!

struct Surface {
  normal: vec3<f32>;
  albedo: vec3<f32>;
  roughness: f32;
  metallic: f32;
  ao: f32;
}

// Get the surface using trilinear texturing
fn get_surface(pos: vec3<f32>, nor: vec3<f32>, material_id: u32) -> Surface {

}
