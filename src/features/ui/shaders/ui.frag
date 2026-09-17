#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(binding = 2) uniform sampler2DArray font_atlas;

layout(location = 0) in vec4 v_color;
layout(location = 1) flat in float v_kind;
layout(location = 2) flat in float v_corner_radius;
layout(location = 3) flat in float v_font_layer;
layout(location = 4) in vec2 v_local_px;
layout(location = 5) in vec2 v_half_size_px;
layout(location = 6) in vec2 v_uv;

layout(location = 0) out vec4 out_color;

float rounded_box_sdf(vec2 p, vec2 half_size, float radius) {
    vec2 q = abs(p) - half_size + radius;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

void main() {
    if (v_kind >= 0.5) {
        float alpha = texture(font_atlas, vec3(v_uv, v_font_layer)).r * v_color.a;
        out_color = vec4(v_color.rgb, alpha);
    } else {
        vec2 p = v_local_px - v_half_size_px;
        float radius = min(v_corner_radius, min(v_half_size_px.x, v_half_size_px.y));
        float dist = rounded_box_sdf(p, v_half_size_px, radius);
        float edge = fwidth(dist);
        float alpha = (1.0 - smoothstep(-edge, edge, dist)) * v_color.a;
        out_color = vec4(v_color.rgb, alpha);
    }
}
