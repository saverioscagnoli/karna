struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv_coords: vec2<f32>,
}
// Per-instance model matrix, delivered as four column vectors because
// vertex attributes can't be mat4x4 directly. Locations 3-6 continue
// from the per-vertex attributes above and must match the
// `vertex_attr_array!` in the instance VertexBufferLayout.

struct InstanceInput {
    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,
    @location(7) color: vec4<f32>,
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv_coords: vec2<f32>,
}
// Group 0: camera (bound once per layer by Renderer::_present)
@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;
// Group 1: texture atlas (bound once per layer by Renderer::_present)
@group(1) @binding(0)
var texture_atlas: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;
// Group 2: material uniforms (bound once per material batch)
struct MaterialUniforms {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    uv_rect: vec4<f32>, // xy = offset, zw = size, in atlas space
    metallic: f32,
    roughness: f32,
    _pad: vec2<f32>,
}
@group(2) @binding(0)
var<uniform> material: MaterialUniforms;

@vertex
fn vs_main(vertex: VertexInput, inst: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(inst.model_0, inst.model_1, inst.model_2, inst.model_3);
    var out: VertexOutput;
    let world_position = model * vec4<f32>(vertex.position, 1.0);
    out.clip_position = view_projection * world_position;
    out.color = vertex.color;
    out.uv_coords = material.uv_rect.xy + vertex.uv_coords * material.uv_rect.zw;
    return out;
}
// User-facing colors (vertex color, base_color, emissive) arrive
// sRGB-encoded; the render target is sRGB so all math and output happen
// in LINEAR space. Each sRGB input is decoded separately — products of
// encoded values are not equivalent to products of linear ones. Texture
// samples are already linear (sRGB texture formats decode on sample).
// Rendering in linear is what makes upcoming lighting math correct.
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture_atlas, texture_sampler, in.uv_coords);
    let vertex_rgb = srgb_to_linear(in.color.rgb);
    let base_rgb = srgb_to_linear(material.base_color.rgb);
    let emissive_rgb = srgb_to_linear(material.emissive.rgb);
    var color = vec4<f32>(vertex_rgb, in.color.a) * tex_color
        * vec4<f32>(base_rgb, material.base_color.a);
    color = vec4<f32>(color.rgb + emissive_rgb, color.a);
    return color;
}
