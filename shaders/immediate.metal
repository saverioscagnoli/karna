#include <metal_stdlib>
using namespace metal;

struct Camera { float4x4 view_projection; };

struct VertexIn {
    float3 position [[attribute(0)]];
    float4 color    [[attribute(1)]];
    // .xy = uv within the page, .z = atlas page (array slice)
    float3 uv       [[attribute(2)]];
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
    float3 uv;
};

vertex VertexOut main0(VertexIn in [[stage_in]],
                       constant Camera& camera [[buffer(0)]])
{
    VertexOut out;
    out.position = camera.view_projection * float4(in.position, 1.0);
    out.color    = in.color;
    out.uv       = in.uv;
    return out;
}

fragment float4 main0(VertexOut in [[stage_in]],
                      texture2d_array<float> atlas_page [[texture(0)]],
                      sampler atlas_smp [[sampler(0)]])
{
    return in.color * atlas_page.sample(atlas_smp, in.uv.xy, uint(in.uv.z + 0.5));
}
