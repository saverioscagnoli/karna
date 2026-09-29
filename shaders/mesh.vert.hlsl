struct MaterialData
{
    float4 color;
    float4 uv_rect;
    float2 params;
    float2 _pad;
};

StructuredBuffer<MaterialData> materials : register(t0, space0);

cbuffer Camera : register(b0, space1)
{
    float4x4 view_projection;
};

struct Input
{
    [[vk::location(0)]] float3 position : TEXCOORD0;
    [[vk::location(1)]] float3 normal : TEXCOORD1;
    [[vk::location(2)]] float2 uv : TEXCOORD2;
    [[vk::location(3)]] float4 color : TEXCOORD3;
    [[vk::location(4)]] float4 model0 : TEXCOORD4;
    [[vk::location(5)]] float4 model1 : TEXCOORD5;
    [[vk::location(6)]] float4 model2 : TEXCOORD6;
    [[vk::location(7)]] float4 model3 : TEXCOORD7;
    [[vk::location(8)]] float3 normal0 : TEXCOORD8;
    [[vk::location(9)]] float3 normal1 : TEXCOORD9;
    [[vk::location(10)]] float3 normal2 : TEXCOORD10;
    [[vk::location(11)]] float4 tint : TEXCOORD11;
    [[vk::location(12)]] uint material : TEXCOORD12;
};

struct Output
{
    [[vk::location(0)]] float4 color : TEXCOORD0;
    [[vk::location(1)]] float3 normal : TEXCOORD1;
    [[vk::location(2)]] float2 uv : TEXCOORD2;
    [[vk::location(3)]] nointerpolation float4 uv_rect : TEXCOORD3;
    [[vk::location(4)]] nointerpolation float2 params : TEXCOORD4;
    float4 position : SV_Position;
};

Output main(Input input)
{
    float4 world = input.model0 * input.position.x
                 + input.model1 * input.position.y
                 + input.model2 * input.position.z
                 + input.model3;

    Output output;
    output.position = mul(view_projection, world);
    output.normal = input.normal0 * input.normal.x
                  + input.normal1 * input.normal.y
                  + input.normal2 * input.normal.z;
    MaterialData material = materials[input.material];

    output.color = input.color * input.tint * material.color;
    output.uv = input.uv;
    output.uv_rect = material.uv_rect;
    output.params = material.params;
    return output;
}
