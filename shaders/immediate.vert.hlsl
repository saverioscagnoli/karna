cbuffer Camera : register(b0, space1)
{
    float4x4 mvp;
};

struct Input
{
    [[vk::location(0)]] float3 position : TEXCOORD0;
    [[vk::location(1)]] float4 color : TEXCOORD1;
    [[vk::location(2)]] float2 uv : TEXCOORD2;
    [[vk::location(3)]] float page : TEXCOORD3;
};

struct Output
{
    [[vk::location(0)]] float4 color : TEXCOORD0;
    [[vk::location(1)]] float3 uv : TEXCOORD1;
    float4 position : SV_Position;
};

Output main(Input input)
{
    Output output;
    output.position = mul(mvp, float4(input.position, 1.0));
    output.color = input.color;
    output.uv = float3(input.uv, input.page);
    return output;
}
