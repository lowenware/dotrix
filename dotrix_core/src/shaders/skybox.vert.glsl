#version 450

layout(location = 0) in vec3 a_Position;
layout(location = 0) out vec3 v_Position;

layout(set = 0, binding = 0) uniform Renderer {
    mat4 u_ProjView;
    mat4 u_ProjView1;
};

void main() {
    v_Position = a_Position;
    gl_Position = u_ProjView * vec4(v_Position, 1.0);
}
