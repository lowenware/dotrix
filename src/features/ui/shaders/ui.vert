#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec2 in_pos;

layout(binding = 0) uniform UiGlobals {
    vec2 resolution;
    vec2 _padding;
} ui_globals;

struct UiInstanceData {
    vec4 rect;
    vec4 color;
    vec4 params;
    vec4 uv;
};

layout(std430, binding = 1) readonly buffer UiInstances {
    UiInstanceData instances[];
};

layout(location = 0) out vec4 v_color;
layout(location = 1) flat out float v_kind;
layout(location = 2) flat out float v_corner_radius;
layout(location = 3) flat out float v_font_layer;
layout(location = 4) out vec2 v_local_px;
layout(location = 5) out vec2 v_half_size_px;
layout(location = 6) out vec2 v_uv;

void main() {
    UiInstanceData inst = instances[gl_InstanceIndex];
    // Layout uses top-left origin with +y downward; Vulkan NDC maps y=-1 to the
    // top of the framebuffer, so no extra Y flip is needed here.
    vec2 screen_pos = inst.rect.xy + in_pos * inst.rect.zw;
    vec2 ndc = (screen_pos / ui_globals.resolution) * 2.0 - 1.0;
    gl_Position = vec4(ndc, 0.0, 1.0);

    v_color = inst.color;
    v_kind = inst.params.x;
    v_corner_radius = inst.params.y;
    v_font_layer = inst.params.z;
    v_local_px = in_pos * inst.rect.zw;
    v_half_size_px = inst.rect.zw * 0.5;
    v_uv = mix(inst.uv.xy, inst.uv.zw, in_pos);
}
