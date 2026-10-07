Texture2D T : register(t0);
SamplerState S : register(s0);

cbuffer P : register(b0)
{
    float2 resolution;
};

float4 main(float4 pos : SV_POSITION) : SV_TARGET
{
    return T.SampleLevel(S, pos.xy / resolution, 0);
}