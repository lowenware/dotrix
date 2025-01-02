#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 world_position;
layout(location = 1) flat in vec3 world_normal;
layout(location = 2) in float vertex_moisture;
layout(location = 0) out vec4 o_frag_color;

float inverse_lerp(float value, float min_value, float max_value) {
    float result = (value - min_value) / (max_value - min_value);
    return clamp(result, 0.0, 1.0);
}

void main() {
    vec3 ambient_light = vec3(0.5, 0.5, 0.5);
    vec3 light_color = vec3(1.0, 1.0, 1.0);
    vec3 light_position = vec3(0.0, 10000.0, 0.0);

    vec3 normal = normalize(world_normal);
    vec3 light_direction = normalize(light_position - world_position);
    //vec3 light_direction = (normalize(light_position));

    vec3 diffuse = max(dot(normal, light_direction), 0.0) * light_color;
    float min_height = 0.0;
    float max_height = 100.0;
    const int max_layers = 5;
    vec3 colors[max_layers] = vec3[](
            vec3(0.059, 0.533, 0.737), // water
            vec3(0.729, 0.714, 0.667), // sand
            vec3(0.11, 0.286, 0.118), // grass
            vec3(0.275, 0.263, 0.224), // rock
            vec3(0.902, 0.937, 0.996) // snow
        );
    float limits[max_layers] = float[](
            0.0,
            0.01,
            0.05,
            0.4,
            0.8
        );
    float blends[max_layers] = float[](
            0.0,
            0.0,
            0.4,
            0.2,
            0.1
        );
    float moisture_intensity[max_layers] = float[](
            0.0,
            0.0,
            0.00,
            0.00,
            0.008
        );
    vec3 color = colors[0];
    float height = inverse_lerp(world_position.y, min_height, max_height);
    for (int i = 0; i < max_layers; i++) {
        if (height < limits[i]) {
            break;
        }
        color = colors[i];
    }
    for (int i = 0; i < max_layers; i++) {
        float moisture_blend = vertex_moisture * moisture_intensity[i];
        float intensity = inverse_lerp(height - moisture_blend - limits[i], -blends[i] / 2.0 - 0.000001, blends[i] / 2.0);
        color = color * (1.0 - intensity) + colors[i] * intensity;
    }
    o_frag_color = vec4((ambient_light + diffuse), 1.0) * vec4(color, 1.0);
    // o_frag_color = vec4(color, 1.0);
}
