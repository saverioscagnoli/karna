// Vertex shader input (unit quad vertices)
struct VertexInput {
    @location(0) position: vec2<f32>,
}

// Instance data (per-glyph data)
struct GlyphInstance {
    @location(1) position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) uv_min: vec2<f32>,
    @location(4) uv_max: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) rotation: f32,
}

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// Camera view-projection matrix
@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

// Texture atlas
@group(1) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(1)
var atlas_sampler: sampler;

@vertex
fn vs_main(
    vertex: VertexInput,
    glyph: GlyphInstance,
) -> VertexOutput {
    var out: VertexOutput;

    // Scale the vertex
    let scaled_vertex = vertex.position * glyph.size;

    // Apply rotation
    let cos_rot = cos(glyph.rotation);
    let sin_rot = sin(glyph.rotation);
    let rotated_vertex = vec2<f32>(
        scaled_vertex.x * cos_rot - scaled_vertex.y * sin_rot,
        scaled_vertex.x * sin_rot + scaled_vertex.y * cos_rot
    );

    // Translate to final position
    let world_pos = glyph.position + rotated_vertex;
    out.clip_position = view_projection * vec4<f32>(world_pos, 0.0, 1.0);

    // Interpolate UV coordinates between min and max based on vertex position
    out.uv = mix(glyph.uv_min, glyph.uv_max, vertex.position);
    out.color = glyph.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture atlas (glyphs are stored as white with alpha)
    let sampled = textureSample(atlas_texture, atlas_sampler, in.uv);

    // Use the alpha channel from the texture and apply the glyph color
    return vec4<f32>(in.color.rgb, in.color.a * sampled.a);
}
