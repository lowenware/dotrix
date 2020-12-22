#version 450

const int MAX_JOINTS = 32;

layout(location = 0) in vec3 a_Position;
layout(location = 1) in vec3 a_Normal;
layout(location = 2) in vec2 a_TexCoord;
layout(location = 3) in vec4 a_Weights;
layout(location = 4) in uvec4 a_Joints;
layout(location = 0) out vec3 v_Position;
layout(location = 1) out vec3 v_Normal;
layout(location = 2) out vec2 v_TexCoord;

layout(set = 0, binding = 0) uniform Renderer {
    mat4 u_ProjView;
};

layout(set = 0, binding = 1) uniform Model {
    mat4 u_Model;
};

layout(set = 0, binding = 2) uniform JointMatrices {
    mat4 u_JointMatrix[MAX_JOINTS];
};

void main() {
    mat4 skinMatrix =
      a_Weights.x * u_JointMatrix[int(a_Joints.x)] +
      a_Weights.y * u_JointMatrix[int(a_Joints.y)] +
      a_Weights.z * u_JointMatrix[int(a_Joints.z)] +
      a_Weights.w * u_JointMatrix[int(a_Joints.w)];

    v_TexCoord = a_TexCoord;
    // TODO: multiply by model transform matrix
    v_Normal = a_Normal;
    v_Position = a_Position;
    gl_Position = u_ProjView * u_Model * skinMatrix * vec4(v_Position, 1.0);
}
