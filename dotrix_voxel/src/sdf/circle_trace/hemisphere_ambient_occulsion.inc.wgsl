// This is a modified AO implementation.
// It attempt to probe the space around a point and determine
// How full that space is and therefore how occulded it is
//
// http://www.aduprat.com/portfolio/?page=articles/hemisphericalSDFAO
//
// It takes the standard concept of marching along the normal
// but expands it to march along multiple rays in a hemisphere
// like arrangment around the normal
//
// Uniform points on a hemisphere
// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
//
// The Hammersley function is used for uniform hemisphere points due to its
// Simplicilty on the gpu

struct AoResult {
  ao: f32;
};

struct AoInput {
  origin: vec3<f32>;
  direction: vec3<f32>;
  samples: u32;
  steps: u32;
  ao_step_size: f32;
};

let PI: f32 = 3.14159265358979;
fn radicalInverse_VdC(in_bits: u32) -> f32 {
    var bits: u32 = in_bits;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}
fn hammersley2d(i: u32, N: u32) -> vec2<f32> {
     return vec2<f32>(f32(i)/f32(N), radicalInverse_VdC(i));
}
fn hemisphereSample_uniform(u: f32, v: f32) -> vec3<f32> {
     let phi: f32 = v * 2.0 * PI;
     let cosTheta: f32 = 1.0 - u;
     let sinTheta: f32 = sqrt(1.0 - cosTheta * cosTheta);
     return vec3<f32>(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);
}

fn ambient_occlusion(input: AoInput) -> AoResult
{
    let nb_ite: u32 = input.samples;
    let nb_ite_inv: f32 = 1./f32(nb_ite);
    let rad: f32 = 1. - (1. * nb_ite_inv); //Hemispherical factor (self occlusion correction)

    var ao: f32 = 0.0;

    // Tangent space tranformation
    let a: vec3<f32> = vec3<f32>(0., 0., 1.);
    let b: vec3<f32> = input.direction;
    let v: vec3<f32> = cross(a,b);
    let s: f32 =  length(v);
    let I: mat3x3<f32> = mat3x3<f32>(vec3<f32>(1.,0.,0.), vec3<f32>(0.,1.,0.), vec3<f32>(0.,0.,1.));
    var R: mat3x3<f32>;
    if (abs(s) < 0.01) {
      R = I;
    } else {
      let c: f32 = dot(a, b);
      let sx: mat3x3<f32> = mat3x3<f32>(vec3<f32>(0.,v.z,-v.y), vec3<f32>(-v.z,0.,v.x), vec3<f32>(v.y,-v.x,0.));

      // R = I + sx + sx * sx * (1./(1. + c)); mat + mat addition is broken upstream https://github.com/gfx-rs/naga/issues/1527
      // Workaround start
      let ISx: mat3x3<f32> = mat3x3<f32>(I.x + sx.x, I.y + sx.y, I.z + sx.z);
      let sxsx: mat3x3<f32> = sx * sx * (1./(1. + c));

      R =  mat3x3<f32>(ISx.x + sxsx.x, ISx.y + sxsx.y, ISx.z + sxsx.z);
      // Workaround end
    }



    for( var i: u32 = 0u; i < nb_ite; i = i + 1u )
    {
        let hammersley: vec2<f32> = hammersley2d(i, nb_ite);
        let rd = hemisphereSample_uniform(hammersley.x, hammersley.y);

        // In tangent space
        let direction: vec3<f32> = R * rd;

        // Stepping on the ray
        var sum: f32 = 0.;
        var max_sum: f32 = 0.;
        for (var j: u32 = 0u; j < input.steps; j = j + 1u)
      	{
          let p: vec3<f32> = input.origin + direction * f32(j + 1u) * input.ao_step_size;
            sum     = sum     + 1. / pow(2., f32(j)) * max(map(p), 0.);
            max_sum = max_sum + 1. / pow(2., f32(j)) * f32(j + 1u) * input.ao_step_size;
      	}

        ao = ao + (sum / max_sum) / f32(nb_ite);
    }

    var ray_out: AoResult;
    ray_out.ao = clamp(ao, 0., 1.);
    return ray_out;
}
