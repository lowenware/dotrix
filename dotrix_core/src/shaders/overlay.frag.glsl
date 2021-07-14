#version 450

//out(location = 0) in vec2 v_TexCoord;
//layout(location = 0) out vec4 o_Target;
//layout(set = 0, binding = 1) uniform texture2D t_Color;
//layout(set = 0, binding = 2) uniform sampler s_Color;

//void main() {
//    o_Target = texture(sampler2D(t_Color, s_Color), v_TexCoord);
//}

layout(location = 0) in vec2 v_tex_coord;
layout(location = 1) in vec4 v_color;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform texture2D t_texture;
layout(set = 0, binding = 2) uniform sampler s_texture;

void main() {
    f_color = v_color * texture(sampler2D(t_texture, s_texture), v_tex_coord);
}
