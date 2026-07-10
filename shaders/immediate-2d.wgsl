//// Vertex shader input (per-vertex data only, no instances)
//struct VertexInput {
//    @location(0) position: vec3<f32>,
//    @location(1) color: vec4<f32>,
//    @location(2) uv_coords: vec2<f32>,
//}
//
//// Vertex shader output / Fragment shader input
//struct VertexOutput {
//    @builtin(position) clip_position: vec4<f32>,
//    @location(0) color: vec4<f32>,
//    @location(1) uv_coords: vec2<f32>,
//}
//
//// Camera view-projection matrix
//@group(0) @binding(0)
//var<uniform> view_projection: mat4x4<f32>;
//
//// Texture atlas
//@group(1) @binding(0)
//var texture_atlas: texture_2d<f32>;
//@group(1) @binding(1)
//var texture_sampler: sampler;
//
//@vertex
//fn vs_main(vertex: VertexInput) -> VertexOutput {
//    var out: VertexOutput;
//
//    // Apply camera view-projection directly (no model transform)
//    out.clip_position = view_projection * vec4<f32>(vertex.position, 1.0);
//    out.color = vertex.color;
//    out.uv_coords = vertex.uv_coords;
//
//    return out;
//}
//
//@fragment
//fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//    let tex_color = textureSample(texture_atlas, texture_sampler, in.uv_coords);
//    return in.color * tex_color;
//}

struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>, // unused for now
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
