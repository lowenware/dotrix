#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 texture;

layout (binding = 0) uniform DtxGlobals {
    mat4 proj;
    mat4 view;
} dtx_globals;

struct DtxInstance {
    mat4 transform;
    uint material_index;
    uint _padding[3]; 
};

layout(std430, binding = 1) buffer DtxInstanceLayout
{
    DtxInstance dtx_instance[];
};

struct DtxMaterial {
    vec4 color;
    vec4 options;
    uvec4 maps_1;
    uvec4 maps_2;
};

layout(std430, binding = 2) buffer DtxMaterialLayout
{
    DtxMaterial dtx_material[];
};

layout (location = 0) out vec3 o_world_position;
layout (location = 1) out vec3 o_world_normal;
layout (location = 2) out vec4 o_color;
void main() {
    mat4 model_transform = dtx_instance[gl_InstanceIndex].transform;
    uint material_index = dtx_instance[gl_InstanceIndex].material_index;
    vec4 material_color = dtx_material[material_index].color;
    
    mat4 proj_view = dtx_globals.proj * dtx_globals.view;
    o_world_position = vec3(model_transform * vec4(pos, 1.0));
    o_world_normal = vec3(model_transform * vec4(normal, 1.0));
    o_color = vec4(material_color);

    gl_Position = proj_view * vec4(o_world_position, 1.0);
}
