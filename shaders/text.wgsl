struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) _color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct GlyphInstance {
    @location(3) position: vec3<f32>,
    @location(4) rotation: vec3<f32>,
    @location(5) offset: vec2<f32>,      // ADD THIS
    @location(6) size: vec2<f32>,
    @location(7) scale: vec2<f32>,
    @location(8) uv_offset: vec2<f32>,
    @location(9) uv_scale: vec2<f32>,
    @location(10) color: vec4<f32>,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

@group(1) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(1)
var atlas_sampler: sampler;

// Helper function to create 3D rotation matrix
fn rotation_matrix(rot: vec3<f32>) -> mat3x3<f32> {
    let cos_x = cos(rot.x);
    let sin_x = sin(rot.x);
    let cos_y = cos(rot.y);
    let sin_y = sin(rot.y);
    let cos_z = cos(rot.z);
    let sin_z = sin(rot.z);

    // Rotation around X axis
    let rx = mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, cos_x, -sin_x),
        vec3<f32>(0.0, sin_x, cos_x)
    );

    // Rotation around Y axis
    let ry = mat3x3<f32>(
        vec3<f32>(cos_y, 0.0, sin_y),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-sin_y, 0.0, cos_y)
    );

    // Rotation around Z axis
    let rz = mat3x3<f32>(
        vec3<f32>(cos_z, -sin_z, 0.0),
        vec3<f32>(sin_z, cos_z, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );

    // Combined rotation: Z -> Y -> X
    return rx * ry * rz;
}

@vertex
fn vs_main(vertex: VertexInput, glyph: GlyphInstance) -> VertexOutput {
    var out: VertexOutput;

    let scaled_size = glyph.size * glyph.scale;

    // Quad vertex position (relative to glyph top-left)
    let quad_pos = vec3<f32>(
        (vertex.position.x + 0.5) * scaled_size.x,
        (vertex.position.y + 0.5) * scaled_size.y,
        0.0
    );

    // Total offset from pivot: glyph offset + quad position
    let total_offset = vec3<f32>(glyph.offset.x, glyph.offset.y, 0.0) + quad_pos;

    // Rotate the total offset around the pivot
    let rot_matrix = rotation_matrix(glyph.rotation);
    let rotated_offset = rot_matrix * total_offset;

    // Add to pivot position
    let world_pos = glyph.position + rotated_offset;
    out.clip_position = view_projection * vec4<f32>(world_pos, 1.0);
    out.uv = glyph.uv_offset + (vertex.uv * glyph.uv_scale);
    out.color = glyph.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas_texture, atlas_sampler, in.uv);
    return vec4<f32>(in.color.rgb, in.color.a * sampled.a);
}
