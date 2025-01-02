#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
// layout(location = 2) in vec4 color;
layout(location = 2) in float moisture;

layout(binding = 0) uniform DtxGlobals {
    mat4 proj;
    mat4 view;
} dtx_globals;

struct DtxInstance {
    mat4 transform;
};

layout(std430, binding = 1) buffer DtxInstanceLayout
{
    DtxInstance dtx_instance[];
};

layout(location = 0) out vec3 o_world_position;
layout(location = 1) flat out vec3 o_world_normal;
// layout(location = 2) out vec4 o_color;
layout(location = 2) out float o_moisture;
void main() {
    mat4 model_transform = dtx_instance[gl_InstanceIndex].transform;

    mat4 proj_view = dtx_globals.proj * dtx_globals.view;
    o_world_position = pos; //vec3(model_transform * vec4(pos, 1.0));
    // o_world_normal = vec3(model_transform * vec4(normalize(normal), 1.0));
    o_world_normal = normalize(normal);
    o_moisture = moisture;

    //vec4(
    //        float((color >> 24) & 0xFF) / 255.0,
    //        float((color >> 16) & 0xFF) / 255.0,
    //        float((color >> 8) & 0xFF) / 255.0,
    //        float(color & 0xFF) / 255.0
    //    );

    // o_color = vec4(0.0, 1.0, 0.0, 1.0);

    gl_Position = proj_view * vec4(o_world_position, 1.0);
}
