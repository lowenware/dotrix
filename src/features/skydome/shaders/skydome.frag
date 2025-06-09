#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 world_position;
layout(location = 1) in vec3 world_normal;
layout(location = 0) out vec4 o_frag_color;

float inverse_lerp(float value, float min_value, float max_value) {
    float result = (value - min_value) / (max_value - min_value);
    return clamp(result, 0.0, 1.0);
}

layout(binding = 0) uniform DtxGlobals {
    mat4 proj;
    mat4 view;
    mat4 transform;
    vec4 horizon_color;
    vec4 zenith_color;
    vec4 extras;
    vec4 _padding;
} dtx_globals;

void main() {
    float size = dtx_globals.extras.x;
    float offset_y = dtx_globals.extras.y;
    float height_factor = (world_position.y - offset_y) / size;
    o_frag_color = dtx_globals.horizon_color * (1 - height_factor)
            + dtx_globals.zenith_color * height_factor;
}
