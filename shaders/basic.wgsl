struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceInput {
    @location(2) instance_position: vec3<f32>,
    @location(3) instance_scale: vec3<f32>,
    @location(4) instance_rotation: vec3<f32>,
    @location(5) instance_color: vec4<f32>,
    @location(6) instance_uv_offset: vec2<f32>,
    @location(7) instance_uv_scale: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    var pos = vertex.position * instance.instance_scale;

    let angle = instance.instance_rotation.z;
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let rotated_x = pos.x * cos_a - pos.y * sin_a;
    let rotated_y = pos.x * sin_a + pos.y * cos_a;
    pos = vec3<f32>(rotated_x, rotated_y, pos.z);

    pos = pos + instance.instance_position;

    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);
    out.uv = vertex.uv * instance.instance_uv_scale + instance.instance_uv_offset;
    out.color = instance.instance_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    return tex_color * in.color; // Tint texture with instance color
}
