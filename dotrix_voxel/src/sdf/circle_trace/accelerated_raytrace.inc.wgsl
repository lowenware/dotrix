// Given:
// A `map` function that gives the distance to the sufrace from any point
// A direction (`rd`) and origin (`ro`) that represents the ray to be marched
// An intial distance already travelled (`t_in`)
// Ray differentails that represent the direction of neighbouring rays (`rdx`, `rdy`)
// Then:
// Compute the point at which this ray intersects the surface
//
// This implementation uses
// Accelerated raymarching
// https://www.researchgate.net/publication/331547302_Accelerating_Sphere_Tracing
// Which attempts to overstep on the ray in order to reduce the number of steps marched
// on the ray
//
// The function `pixel_radius` is used for an early exit, given the directions and
// the distance traveled it computes the approixmate size of a pixel on screen
// If the distance to the surface is less then the returned pixel size then we
// Stop marching
struct RaymarchOut {
  t: f32;
  success: bool;
};

fn raymarch(t_in: f32, ro: vec3<f32>, rd: vec3<f32>, rdx: vec3<f32>, rdy: vec3<f32>) -> RaymarchOut {
  let o: vec3<f32> = ro;
  let d: vec3<f32> = rd;
  let dx: vec3<f32> = rdx;
  let dy: vec3<f32> = rdy;

  let STEP_SIZE_REDUCTION: f32 = 0.95;
  let MAX_DISTANCE: f32 = t_in + length(u_sdf.grid_dimensions.xyz * abs(u_sdf.world_scale.xyz));
  let MAX_ITERATIONS: u32 = 128u;

  var t: f32 = t_in;
  var rp: f32 = 0.; // prev
  var rc: f32 = map(o + (t)*d);; // current
  var rn: f32 = t + MAX_DISTANCE * 2.0; // next (set to effectivly infinity)

  var di: f32 = 0.;

  var out: RaymarchOut;
  out.success = false;

  for(var i: u32 = 0u; i < MAX_ITERATIONS && t < MAX_DISTANCE; i = i + 1u)
  {
    di = rc + STEP_SIZE_REDUCTION * rc * max( (di - rp + rc) / (di + rp - rc), 0.6);
    rn = map(o + (t + di)*d);
    if(di > rc + rn) {
      di = rc;
      rn = map(o + (t + di)*d);
    }
    t = t + di;
    out.t = t;
    if(rn < pixel_radius(t, d, dx, dy)) {
      out.success = true;
      return out;
    }

    rp = rc;
    rc = rn;
  }

  return out;
}
