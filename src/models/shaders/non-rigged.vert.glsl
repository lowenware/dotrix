#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 color;
layout (location = 2) in vec2 texture;


layout (location = 0) out vec4 o_color;
void main() {
    o_color = vec4(color, 1.0);
    gl_Position = vec4(pos, 1.0);
}