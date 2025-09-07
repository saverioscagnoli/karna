// primitive.wgsl - Shader for 2D primitives

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

// Uniform buffer for view/projection matrices
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_size: vec2<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Convert screen coordinates to clip space
    // Assuming screen coordinates with origin at top-left
    let normalized = vec2<f32>(
        (in.position.x / camera.view_size.x) * 2.0 - 1.0,
        1.0 - (in.position.y / camera.view_size.y) * 2.0
    );
    
    out.clip_position = vec4<f32>(normalized, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.uv;
    
    return out;
}

@fragment 
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}