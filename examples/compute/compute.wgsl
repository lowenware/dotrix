
struct Particle {
  position : vec4<f32>;
  velocity : vec4<f32>;
};

struct Params {
  position : vec3<f32>;
  timeDelta : f32;
  gravity : f32;
};

struct Particles {
  particles : [[stride(32)]] array<Particle>;
};

[[group(0), binding(0)]] var<uniform> uParams : Params;
[[group(0), binding(1)]] var<storage, read_write> sParticles : Particles;


[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let total = arrayLength(&sParticles.particles);
  let index = global_invocation_id.x;
  if (index >= total) {
    return;
  }
  let gravity : f32 = uParams.gravity;
  let timeDelta : f32 = uParams.timeDelta;

  let pos : vec3<f32> = sParticles.particles[index].position.xyz;
  let vel : vec3<f32> = sParticles.particles[index].velocity.xyz;
  let time : f32 = sParticles.particles[index].velocity.w + timeDelta;
  let offset : vec3<f32> = vec3<f32>(vel.x, vel.y + gravity * time, vel.z);
  // Write back
  if (pos.y < 0.0) {
    // reset particle
    sParticles.particles[index].position = vec4<f32>(uParams.position, 1.0);
    sParticles.particles[index].velocity = vec4<f32>(vel, 0.0);
  } else {
    sParticles.particles[index].position = vec4<f32>(pos + offset, 1.0);
    sParticles.particles[index].velocity = vec4<f32>(vel, time);
  }
}
