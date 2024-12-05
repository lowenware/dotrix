#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 world_position;
layout(location = 1) in vec3 world_normal;
layout(location = 2) in vec4 vertex_color;
layout(location = 3) in vec3 vertex_texture;
layout(location = 0) out vec4 o_frag_color;

layout(binding = 4) uniform sampler2DArray dtx_material_sampler;

void main() {
    vec3 ambient_light = vec3(0.5, 0.5, 0.5);
    vec4 color = vec4(0.0, 0.0, 0.0, 0.0);
    vec3 light_color = vec3(1.0, 1.0, 1.0);
    vec3 light_position = vec3(10.0, 10.0, 10.0);

    vec3 normal = normalize(world_normal);
    vec3 light_direction = normalize(light_position - world_position);

    vec3 diffuse = max(dot(normal, light_direction), 0.0) * light_color;
    vec4 texture_color = texture(dtx_material_sampler, vertex_texture);

    o_frag_color = vec4((ambient_light + diffuse), 1.0) * vertex_color * texture_color;
}
