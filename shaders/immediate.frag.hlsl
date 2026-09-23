Texture2DArray<float4> atlas : register(t0, space2);
SamplerState atlas_sampler : register(s0, space2);

struct Input
{
    [[vk::location(0)]] float4 color : TEXCOORD0;
    [[vk::location(1)]] float3 uv : TEXCOORD1;
};

float4 main(Input input) : SV_Target0
{
    return atlas.Sample(atlas_sampler, input.uv) * input.color;
}
