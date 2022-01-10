struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

struct Renderer {
  proj_view: mat4x4<f32>;
};

struct Particle {
  position : vec4<f32>;
  velocity : vec4<f32>;
  color: vec4<f32>;
  life_time: f32;
};

struct Particles {
  particles : [[stride(64)]] array<Particle>;
};

[[group(0), binding(0)]] var<uniform> uRenderer: Renderer;
[[group(0), binding(1)]] var<storage, read_write> uParticles : Particles;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[builtin(instance_index)]] instance_id: u32,
) -> VertexOutput {
  let particle_pos = vec4<f32>(uParticles.particles[instance_id].position.xyz, 1.);
  var out: VertexOutput;
  out.position = uRenderer.proj_view * vec4<f32>(position + particle_pos.xyz, 1.0);
  out.color = uParticles.particles[instance_id].color;
  return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}
