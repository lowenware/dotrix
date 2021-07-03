#version 450

layout(location = 0) in vec3 v_Position;
layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 1) uniform textureCube t_CubeMap;
layout(set = 0, binding = 2) uniform sampler s_CubeMap;

void main() {
    vec3 uv = vec3(-v_Position.x, v_Position.yz);
    o_Target = texture(samplerCube(t_CubeMap, s_CubeMap), uv);
}
