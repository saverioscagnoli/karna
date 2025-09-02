@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) instance_translation: vec2<f32>,
    @location(3) instance_scale: vec2<f32>,
    @location(4) instance_color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.instance_color;
    let scaled_pos = model.pos.xy * model.instance_scale;
    let final_pos = scaled_pos + model.instance_translation;
    let pos = vec4<f32>(final_pos, model.pos.z, 1.0);
    out.clip_position = projection * pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}