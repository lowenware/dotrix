
struct Particle {
  position : vec4<f32>;
  velocity : vec4<f32>;
  color: vec4<f32>;
  life_time: f32;
};

struct Params {
  position : vec3<f32>;
  simulation_time : f32;
  gravity : f32;
  start_color: vec4<f32>;
  end_color: vec4<f32>;
};

struct Particles {
  particles : [[stride(64)]] array<Particle>;
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
  let time : f32 = uParams.simulation_time;
  let col_0 = uParams.start_color;
  let col_1 = uParams.end_color;

  let start_time: f32 = 0.;
  let life_time : f32 = sParticles.particles[index].life_time;

  let age: f32 = (time - start_time) % life_time;

  var pos : vec3<f32> = uParams.position.xyz;
  var vel : vec3<f32> = sParticles.particles[index].velocity.xyz;
  // Distance travelled
  pos = pos + age * vel.xyz;
  // Gravity
  pos = pos + vec3<f32>(0., gravity, 0.) * age*age;

  sParticles.particles[index].position = vec4<f32>(pos, 1.0);

  let age: f32 = time % life_time;
  sParticles.particles[index].color = mix(col_0, col_1, smoothStep(0., life_time, age));
}
