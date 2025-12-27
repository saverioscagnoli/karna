
// Vertex shader input (unit quad vertices)
struct VertexInput {
    @location(0) position: vec3<f32>,
}

// Instance data (per-glyph data)
struct GlyphInstance {
    @location(1) position: vec3<f32>,    // Changed to vec3 to match Rust
    @location(2) size: vec2<f32>,
    @location(3) uv_offset: vec2<f32>,   // Changed from uv_min
    @location(4) uv_scale: vec2<f32>,    // Changed from uv_max
    @location(5) color: vec4<f32>,
    @location(6) rotation: vec3<f32>,    // Changed to vec3 to match Rust
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

    // Scale the vertex (using x and y only)
    let scaled_vertex = vec2<f32>(vertex.position.x, vertex.position.y) * glyph.size;

    // Apply rotation (using z component for 2D rotation)
    let cos_rot = cos(glyph.rotation.z);
    let sin_rot = sin(glyph.rotation.z);

    let rotated_vertex = vec2<f32>(
        scaled_vertex.x * cos_rot - scaled_vertex.y * sin_rot,
        scaled_vertex.x * sin_rot + scaled_vertex.y * cos_rot
    );

    // Translate to final position
    let world_pos = glyph.position.xy + rotated_vertex;
    out.clip_position = view_projection * vec4<f32>(world_pos, glyph.position.z, 1.0);

    // Calculate UV coordinates: offset + (vertex_pos * scale)
    // vertex.position.xy is in [0,1] range for our unit quad
    out.uv = glyph.uv_offset + (vertex.position.xy * glyph.uv_scale);

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
