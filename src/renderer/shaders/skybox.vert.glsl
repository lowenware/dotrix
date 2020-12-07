#version 450

layout(location = 0) in vec3 a_Position;
layout(location = 0) out vec3 v_Position;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_Proj;
    mat4 u_View;
};

void main() {
    v_Position = a_Position;
    gl_Position = u_Proj * u_View * vec4(v_Position, 1.0);
}
