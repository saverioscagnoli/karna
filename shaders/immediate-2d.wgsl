// immediate-2d-shapes — unified shader for all immediate-mode 2D drawing.
//
// `mode` packs two things:
//   low 8 bits: shape mode        (in.mode & MODE_MASK)
//   bit 8:      AA disabled flag  (in.mode & FLAG_NO_AA)
//
// modes:
//   0 TEXTURED  atlas quads: rects/images/raster glyphs.
//               params.x > 0 enables SDF edge clipping with
//               params = (half_w, half_h, corner_radius, -);
//               glyphs pass zero params (alpha from the glyph texture).
//   1 CIRCLE    params = (radius, stroke, -, -)
//   2 RRECT     params = (half_w, half_h, corner_radius, stroke)
//   3 CAPSULE   params = (bx, by, half_thickness, stroke); a = local origin
//   4 MSDF      params = (px_range, -, -, -)
//
// uv_rect = the atlas entry's uv bounds (min.xy, max.zw). The sample
// coordinate is clamped to it, so padded quads and bilinear filtering can
// never bleed neighboring atlas entries — no gutters required.
//
// stroke convention (SDF modes): stroke <= 0 fills; stroke > 0 draws an
// outline of that total width centered on the shape edge.

const MODE_MASK: u32 = 0xFFu;
const FLAG_NO_AA: u32 = 0x100u;

const MODE_TEXTURED: u32 = 0u;
const MODE_CIRCLE: u32 = 1u;
const MODE_RRECT: u32 = 2u;
const MODE_CAPSULE: u32 = 3u;
const MODE_MSDF: u32 = 4u;

struct VertexInput {
    @location(0) position: vec3<f32>,  // transformed quad corner (world)
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,        // atlas uv (white pixel for SDF modes)
    @location(3) local: vec2<f32>,     // untransformed shape-local position
    @location(4) params: vec4<f32>,    // shape-specific, see table above
    @location(5) uv_rect: vec4<f32>,   // atlas entry bounds: (min.xy, max.zw)
    @location(6) mode: u32,            // shape mode | flags
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) local: vec2<f32>,
    @location(3) params: vec4<f32>,
    @location(4) @interpolate(flat) uv_rect: vec4<f32>,
    @location(5) @interpolate(flat) mode: u32,
}

@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

@group(1) @binding(0)
var texture_atlas: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view_projection * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;
    out.uv = vertex.uv;
    out.local = vertex.local;
    out.params = vertex.params;
    out.uv_rect = vertex.uv_rect;
    out.mode = vertex.mode;
    return out;
}

// Vertex colors arrive sRGB-encoded; the render target is sRGB so the
// shader must output LINEAR values. Texture samples are already linear
// (sRGB texture formats decode on sample).
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Segment from the local-space origin to b.
fn sd_segment(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let h = clamp(dot(p, b) / max(dot(b, b), 1e-6), 0.0, 1.0);
    return length(p - b * h);
}

fn median3(v: vec3<f32>) -> f32 {
    return max(min(v.r, v.g), min(max(v.r, v.g), v.b));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let mode = in.mode & MODE_MASK;
    let no_aa = (in.mode & FLAG_NO_AA) != 0u;

    // IMPORTANT: everything involving textureSample or fwidth is computed
    // unconditionally, before any branching — naga's uniformity analysis
    // rejects these ops inside branches on interpolated values, even
    // flat-interpolated ones.

    let atlas_size = vec2<f32>(textureDimensions(texture_atlas));

    // Clamp the sample coordinate to the atlas entry, inset by half a
    // texel so bilinear filtering at the boundary can't mix neighbors.
    let half_texel = vec2<f32>(0.5, 0.5) / atlas_size;
    let uv = clamp(in.uv, in.uv_rect.xy + half_texel, in.uv_rect.zw - half_texel);
    let tex = textureSample(texture_atlas, texture_sampler, uv);

    // Evaluate all geometric SDFs; unused ones are simply never selected.
    let d_circle = length(in.local) - in.params.x;
    let d_rrect = sd_rounded_box(in.local, in.params.xy, in.params.z);
    let d_capsule = sd_segment(in.local, in.params.xy) - in.params.z;

    var d = d_circle;
    var stroke = in.params.y;
    if mode == MODE_RRECT {
        d = d_rrect;
        stroke = in.params.w;
    } else if mode == MODE_CAPSULE {
        d = d_capsule;
        stroke = in.params.w;
    } else if mode == MODE_TEXTURED {
        d = d_rrect;
        stroke = 0.0;
    }

    // Outline: distance to the edge band instead of the interior.
    if stroke > 0.0 {
        d = abs(d) - stroke * 0.5;
    }

    // Screen-space AA width; collapsing it to ~zero gives a hard SDF
    // cutoff, i.e. the anti_alias-off look. (No derivatives past here.)
    var aa = max(fwidth(d), 1e-4);
    aa = select(aa, 1e-5, no_aa);

    // MSDF screen-pixel range (msdfgen's recommended formula).
    // Uses the raw (unclamped) uv for the derivative.
    let unit_range = vec2<f32>(in.params.x, in.params.x) / atlas_size;
    let screen_tex = vec2<f32>(1.0, 1.0) / max(fwidth(in.uv), vec2<f32>(1e-6, 1e-6));
    let screen_px_range = max(0.5 * dot(unit_range, screen_tex), 1.0);

    let rgb = srgb_to_linear(in.color.rgb);

    if mode == MODE_TEXTURED {
        var a = in.color.a;
        if in.params.x > 0.0 {
            a *= 1.0 - smoothstep(-aa, aa, d);
        }
        return vec4<f32>(rgb, a) * tex;
    }

    if mode == MODE_MSDF {
        let sd_px = screen_px_range * (median3(tex.rgb) - 0.5);
        var alpha = clamp(sd_px + 0.5, 0.0, 1.0);
        alpha = select(alpha, step(0.0, sd_px), no_aa);
        return vec4<f32>(rgb, in.color.a * alpha);
    }

    // Geometric SDF modes.
    let alpha = 1.0 - smoothstep(-aa, aa, d);
    return vec4<f32>(rgb, in.color.a * alpha);
}
