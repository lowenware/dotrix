

struct Renderer {
  proj_view: mat4x4<f32>;
};

struct Particle {
  position : vec4<f32>;
  velocity : vec4<f32>;
};

struct Particles {
  particles : [[stride(32)]] array<Particle, 2000>;
};

[[group(0), binding(0)]] var<uniform> uRenderer: Renderer;
[[group(0), binding(1)]] var<uniform> uParticles : Particles;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[builtin(instance_index)]] instance_id: u32,
) -> [[builtin(position)]] vec4<f32> {
  let particle_pos = uParticles.particles[instance_id].position;
  return uRenderer.proj_view * vec4<f32>(position + particle_pos.xyz, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.2, 0.2, 0.2, 1.0);
}
