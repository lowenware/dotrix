// Soft shadows as suggested by
// Sebastian Aaltonen et al. Soft Shadows at his GDC presentation
//
// Marches a ray from a point towards a single light
// If that ray hits the surface then radience will be none
//
// If that ray goes near but does not hit a surface then we are in soft (partial)
// shadow
//
// Calcualation of near surface it simple because of the SDF map
//
// Sebastian Aaltonen et al. provides an improved implementation of nearmiss
// Soft shadows with fewer artifacts that more accuratly works out the closest
// approach point of a ray
//
//
struct SoftShadowResult {
  radiance: f32;
};

struct SoftShadowInput {
  origin: vec3<f32>;
  direction: vec3<f32>;
  max_iterations: u32;
  min_distance: f32;
  max_distance: f32;
  k: f32;
};

fn softshadow (input: SoftShadowInput) -> SoftShadowResult
{
  let o: vec3<f32> = input.origin;
  let d: vec3<f32> = input.direction;

  var di: f32 = 0.;
  var t: f32 = input.min_distance;

  let STEP_SIZE_REDUCTION: f32 = 0.95;
  var rp: f32 = 0.; // prev
  var rc: f32 = 0.; // current large such that y=0.0 at first
  var rn: f32 = map(o + (t)*d); // next

  var radiance: f32 = 1.;

  for(var i: u32 = 0u; i < input.max_iterations && t < input.max_distance; i = i + 1u)
  {
    let y: f32 = rn*rn/(2.0*rc);
    let approx_distance: f32 = sqrt(rn*rn-y*y);
    radiance = min(radiance, input.k * approx_distance/max(0.0,t-y));

    di = rc + STEP_SIZE_REDUCTION * rc * max( (di - rp + rc) / (di + rp - rc), 0.6);
    rn = map(o + (t + di)*d);
    if(di > rc + rn)
    {
      di = rc;
      rn = map(o + (t + di)*d);
    }
    // if(rn < 0.001) {
    //   var out: SoftShadowResult;
    //   out.radiance = 0.;
    //   return out;
    // }
    t = t + di;

    rp = rc;
    rc = rn;
  }
  var out: SoftShadowResult;
  out.radiance = radiance;
  return out;
}
