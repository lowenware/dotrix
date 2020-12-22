#version 450

const int MAX_LIGHTS = 10;

struct Light {
    vec4 position;
    vec4 color;
};

layout(location = 0) in vec3 v_Position;
layout(location = 1) in vec3 v_Normal;
layout(location = 2) in vec2 v_TexCoord;
layout(location = 0) out vec4 o_Target;
layout(set = 0, binding = 3) uniform texture2D t_Color;
layout(set = 0, binding = 4) uniform sampler s_Color;

layout(set = 0, binding = 5) uniform Lights {
    uvec4 lights_length;
    Light lights[MAX_LIGHTS];
};

void main() {
    vec4 result_color = texture(sampler2D(t_Color, s_Color), v_TexCoord);

    vec3 normal = normalize(v_Normal);
    float ambient = 0.1;
 
    vec3 light_color = vec3(ambient);
    for (int i = 0; i < int(lights_length.x) && i < MAX_LIGHTS; i++) {
        Light light = lights[i];

        vec3 light_direction = normalize(light.position.xyz - v_Position);
        float diffuse = max(0.0, dot(normal, light_direction));

        light_color += diffuse * light.color.xyz;
    }
    result_color.xyz *= light_color;

    float mag = length(v_TexCoord-vec2(0.5));
    o_Target = result_color;
    // o_Target = vec4(mix(result_color.xyz, vec3(0.0), mag*mag), 1.0);
}
