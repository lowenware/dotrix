#version 450

struct DirLight {
    vec4 direction;
    vec4 color;
};

struct PointLight {
    vec4 position;
    vec4 color;

    // attenuation
    float a_constant;
    float a_linear;
    float a_quadratic;
    float unused;
};

struct SimpleLight {
    vec4 position;
    vec4 color;
};

struct SpotLight {
    vec4 position;
    vec4 direction;
    vec4 color;

    float cut_off;
    float outer_cut_off;
    vec2 unused;
};

const int MAX_LIGHTS = 10;

layout(location = 0) in vec3 v_Position; // FragPos
layout(location = 1) in vec3 v_Normal;
layout(location = 2) in vec2 v_TexCoord;
layout(location = 0) out vec4 o_Target;
layout(set = 0, binding = 3) uniform texture2D t_Color;
layout(set = 0, binding = 4) uniform sampler s_Color;

layout(set = 0, binding = 5) uniform Lights {
    vec4 ambient;
    uvec4 lights_count;
    DirLight dir_lights[MAX_LIGHTS];
    PointLight point_lights[MAX_LIGHTS];
    SimpleLight simple_lights[MAX_LIGHTS];
    SpotLight spot_lights[MAX_LIGHTS];
};

vec3 CalculateDirLight(DirLight light, vec3 light_color, vec3 normal) {
    vec3 light_direction = normalize(-light.direction.xyz);
    float diffuse = max(0.0, dot(normal, light_direction));
    light_color += diffuse * light.color.xyz;

    return light_color;
}

vec3 CalculatePointLight(PointLight light, vec3 light_color, vec3 normal) {
    vec3 light_direction = normalize(light.position.xyz - v_Position);
    float diffuse = max(0.0, dot(normal, light_direction));

    float distance = length(light.position.xyz - v_Position);
    float attenuation = 1.0 / (light.a_constant + light.a_linear * distance + light.a_quadratic * (distance * distance));

    light_color += (diffuse * light.color.xyz) * attenuation;
    return light_color;
}

vec3 CalculateSimpleLight(SimpleLight light, vec3 light_color, vec3 normal) {
    vec3 light_direction = normalize(light.position.xyz - v_Position);
    float diffuse = max(0.0, dot(normal, light_direction));

    light_color += diffuse * light.color.xyz;
    return light_color;
}

vec3 CalculateSpotLight(SpotLight light, vec3 light_color, vec3 normal) {
    vec3 light_direction = normalize(light.position.xyz - v_Position);
    float theta = dot(light_direction, normalize(-light.direction.xyz));

    float epsilon = light.cut_off - light.outer_cut_off;
    float intensity = clamp((theta - light.outer_cut_off) / epsilon, 0.0, 1.0);

    float diffuse = max(0.0, dot(normal, light_direction));
    light_color += (diffuse * light.color.xyz) * intensity;

    return light_color;
}

void main() {
    vec4 result_color = texture(sampler2D(t_Color, s_Color), v_TexCoord);

    vec3 normal = normalize(v_Normal);
    vec3 light_color = ambient.xyz;

    for (int i = 0; i < int(lights_count.x) && i < MAX_LIGHTS; i++) {
        light_color = CalculateDirLight(dir_lights[i], light_color, normal);
    }

    for (int i = 0; i < int(lights_count.y) && i < MAX_LIGHTS; i++) {
        light_color = CalculatePointLight(point_lights[i], light_color, normal);
    }

    for (int i = 0; i < int(lights_count.z) && i < MAX_LIGHTS; i++) {
        light_color = CalculateSimpleLight(simple_lights[i], light_color, normal);
    }

    for (int i = 0; i < int(lights_count.w) && i < MAX_LIGHTS; i++) {
        light_color = CalculateSpotLight(spot_lights[i], light_color, normal);
    }


    result_color.xyz *= light_color;

    float mag = length(v_TexCoord-vec2(0.5));
    o_Target = result_color;
    // o_Target = vec4(mix(result_color.xyz, vec3(0.0), mag*mag), 1.0);
}