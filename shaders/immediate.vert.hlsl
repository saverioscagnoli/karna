cbuffer Camera : register(b0, space1)
{
    float4x4 view_projection;
};

struct Input
{
    float3 position : TEXCOORD0;
    float4 color    : TEXCOORD1;
    // .xy = uv within the page, .z = atlas page (array layer)
    float3 uv       : TEXCOORD2;
};

struct Output
{
    float4 color    : TEXCOORD0;
    float3 uv       : TEXCOORD1;
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
