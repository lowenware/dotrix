[[location(0)]]
var<in> in_position: vec3<f32>;
[[location(1)]]
var<in> in_color_vs: vec3<f32>;
[[location(0)]]
var<out> out_color_vs: vec4<f32>;
[[builtin(position)]]
var<out> out_position: vec4<f32>;

[[block]]
struct ProjView {
    proj_view: mat4x4<f32>;
};
[[group(0), binding(0)]]
var r_proj_view: ProjView;

[[block]]
struct Transform {
    transform: mat4x4<f32>;
};
[[group(0), binding(1)]]
var r_transform: Transform;

[[stage(vertex)]]
fn vs_main() {
    out_position = r_proj_view.proj_view * r_transform.transform * vec4<f32>(in_position, 1.0);
    out_color_vs = vec4<f32>(in_color_vs, 1.0);
}

[[location(0)]]
var<in> in_color_fs: vec4<f32>;
[[location(0)]]
var<out> out_color_fs: vec4<f32>;

[[stage(fragment)]]
fn fs_main() {
    out_color_fs = in_color_fs;
}
