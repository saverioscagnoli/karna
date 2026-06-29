// Vertex shader input (per-vertex data only, no instances)
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// Camera view-projection matrix
@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Apply camera view-projection directly (no model transform)
    out.clip_position = view_projection * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Multiply vertex color by texture
    // For text: vertex color defines text color, texture provides alpha mask
    // For rects: white texture pixel, so vertex color passes through
    return in.color;
}
