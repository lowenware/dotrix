struct VertexOutput {
    @location(1) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Camera {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
}

struct Transform {
    model: mat4x4<f32>,
}

struct Instance {
    idx: vec4<u32>, // x - transform, y - material, z/w - reserved
}

struct Material {
    /// Albedo color RGBA
    color: vec4<f32>,
    /// x: ambient_occlusion, y:metallic, z:roughness, w:reserve
    options: vec4<f32>,
    /// x: ambient_occlusion, y:metallic, z:normal, w:roughness
    maps_1: vec4<u32>,
    /// x: color map, yzw: reserved
    maps_2: vec4<u32>,
}


@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0)
@binding(1)
var<storage> instances: array<Instance>;

@group(0)
@binding(2)
var<storage> transform: array<Transform>;

@group(0)
@binding(3)
var<storage> materials: array<Material>;


@vertex
fn vs_main_solid(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texUV: vec2<f32>,
    @builtin(instance_index) instance_id: u32,
) -> VertexOutput {
    var result: VertexOutput;
    var idx: vec4<u32> = instances[instance_id].idx;
    var model = transform[idx.x].model;
    var material = materials[idx.y]; 

    result.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    result.color = material.color;
    result.position = camera.proj * camera.view * model * vec4<f32>(position, 1.0);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
