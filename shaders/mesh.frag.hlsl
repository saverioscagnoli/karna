Texture2DArray<float4> atlas : register(t0, space2);
SamplerState atlas_sampler : register(s0, space2);

cbuffer Light : register(b0, space3)
{
    float4 light_direction;
    float4 light_color;
    float4 ambient;
};

struct Input
{
    [[vk::location(0)]] float4 color : TEXCOORD0;
    [[vk::location(1)]] float3 normal : TEXCOORD1;
    [[vk::location(2)]] float2 uv : TEXCOORD2;
    [[vk::location(3)]] nointerpolation float4 uv_rect : TEXCOORD3;
    [[vk::location(4)]] nointerpolation float2 params : TEXCOORD4;
};

float4 main(Input input) : SV_Target0
{
    float2 uv = input.uv_rect.xy + saturate(input.uv) * input.uv_rect.zw;
    float4 base = atlas.Sample(atlas_sampler, float3(uv, input.params.x)) * input.color;

    if (input.params.y > 0.5)
    {
        float3 n = normalize(input.normal);
        float diffuse = max(dot(n, -normalize(light_direction.xyz)), 0.0);
        base.rgb *= ambient.rgb + light_color.rgb * diffuse;
    }

    return base;
}
