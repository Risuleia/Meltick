Texture2D InputTexture : register(t0);
SamplerState InputSampler : register(s0);

cbuffer GlassParams : register(b0)
{
    float  blur_strength;
    float  refraction;
    float  glow_power;
    float  shadow_power;
    float  radius;
    float2 size;
    float  _pad0;
    float2 center;
    float2 resolution;
};

float roundrect_sdf(float2 p)
{
    float2 q = abs(p) - (size * 0.5 - radius);
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

// one tap at a chosen mip level: the mip IS the blur
float3 sampleMip(float2 px, float mip)
{
    return InputTexture.SampleLevel(InputSampler, clamp(px / resolution, 0.0, 1.0), mip).rgb;
}

float4 main(float4 pos : SV_POSITION) : SV_TARGET
{
    float2 p = pos.xy - center;
    float dist = roundrect_sdf(p);

    // drop shadow outside the card
    float shadow = exp(-max(dist, 0.0) / 28.0) * 0.35 * shadow_power * step(0.0, dist);
    if (dist > 0.5)
    {
        if (shadow < 0.001) discard;
        return float4(0.0, 0.0, 0.0, shadow);
    }

    // edge normal from SDF gradient
    float dirLen = length(p);

    float2 dir = dirLen > 1e-5
        ? p / dirLen
        : float2(0.0, 0.0);

    // refraction
    float glassSize = min(size.x, size.y);
    float inv = -dist / glassSize;
    float distFromCenter = 1.0 - clamp(inv / 0.3, 0.0, 1.0);
    float distortion = 1.0 - sqrt(max(1.0 - distFromCenter * distFromCenter, 0.0));
    float2 coord = pos.xy - distortion * refraction * dir * glassSize * 0.5;

    // blur radius (px) -> mip level
    float blurR = blur_strength * (1.0 - distFromCenter * 0.5);
    float mip = log2(1.0 + blurR * 1.3);

    // chromatic aberration: 3 taps, one per channel, same mip
    float edge = smoothstep(0.0, 0.02, inv);
    float2 shift = dir * edge * 3.0;

    float3 glass = float3(
        sampleMip(coord - shift, mip).r,
        sampleMip(coord,         mip).g,
        sampleMip(coord + shift, mip).b);

    // rim highlights
    float rim = smoothstep(-8.0, -0.6, dist);
    float2 l1 = normalize(float2(-0.75, -0.65));
    float2 l2 = normalize(float2(0.9, 0.8));
    glass += rim * pow(saturate(dot(dir, l1)), 1.8) * 0.6  * glow_power;
    glass += rim * pow(saturate(dot(dir, l2)), 1.8) * 0.35 * glow_power;

    // adaptive tint, faded out over black so the card isn't gray
    float3 bg = sampleMip(pos.xy, 0.0);
    float lum = dot(bg, float3(0.299, 0.587, 0.114));
    float response = smoothstep(0.25, 0.9, lum);
    float3 tintValue = lerp(float3(1.08, 1.08, 1.10), float3(0.42, 0.48, 0.58), response);
    glass *= lerp(1.0, tintValue, saturate(lum * 4.0));
    glass *= lerp(1.0, 0.90, saturate(lum * 4.0));

    float coverage = saturate(0.5 - dist);
    return float4(lerp(bg, glass, coverage), 1.0);
}