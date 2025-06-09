#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;

layout(binding = 0) uniform DtxGlobals {
    mat4 proj;
    mat4 view;
    mat4 transform;
    vec4 horizon_color;
    vec4 zenith_color;
    vec4 extras;
    vec4 _padding;
} dtx_globals;

layout(location = 0) out vec3 o_world_position;
layout(location = 1) flat out vec3 o_world_normal;
void main() {
    mat4 proj_view = dtx_globals.proj * dtx_globals.view;
    o_world_position = vec3(dtx_globals.transform * vec4(pos, 1.0));
    o_world_normal = vec3(dtx_globals.transform * vec4(normalize(normal), 1.0));
    o_world_normal = normalize(normal);

    gl_Position = proj_view * vec4(o_world_position, 1.0);
}
