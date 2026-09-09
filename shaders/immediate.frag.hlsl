Texture2D<float4> atlas_page  : register(t0, space2);
SamplerState      atlas_smp   : register(s0, space2);

struct Input
{
    float4 color : TEXCOORD0;
    float2 uv    : TEXCOORD1;
};

float4 main(Input input) : SV_Target0
{
    return input.color * atlas_page.Sample(atlas_smp, input.uv);
}
