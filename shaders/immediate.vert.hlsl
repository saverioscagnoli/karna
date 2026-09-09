cbuffer Camera : register(b0, space1)
{
    float4x4 view_projection;
};

struct Input
{
    float3 position : TEXCOORD0;
    float4 color    : TEXCOORD1;
    float2 uv       : TEXCOORD2;
};

struct Output
{
    float4 color    : TEXCOORD0;
    float2 uv       : TEXCOORD1;
    float4 position : SV_Position;
};

Output main(Input input)
{
    Output output;
    output.position = mul(view_projection, float4(input.position, 1.0));
    output.color    = input.color;
    output.uv       = input.uv;
    return output;
}
