struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct InstanceInput {
    @location(2) instance_position: vec3<f32>,
    @location(3) instance_scale: vec3<f32>,
    @location(4) instance_rotation: vec3<f32>,
    @location(5) instance_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Apply scale
    var pos = vertex.position * instance.instance_scale;

    // Apply rotation (Z-axis only for 2D)
    let angle = instance.instance_rotation.z;
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let rotated_x = pos.x * cos_a - pos.y * sin_a;
    let rotated_y = pos.x * sin_a + pos.y * cos_a;
    pos = vec3<f32>(rotated_x, rotated_y, pos.z);

    // Apply translation
    pos = pos + instance.instance_position;

    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);
    out.color = instance.instance_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
