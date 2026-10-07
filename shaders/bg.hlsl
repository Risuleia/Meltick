#include "digit.hlsli"

float hash(float2 p)
{
    p = frac(p * float2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return frac(p.x * p.y);
}

float vnoise(float2 p)
{
    float2 i = floor(p);
    float2 f = frac(p);

    f = f * f * (3.0 - 2.0 * f);

    return lerp(
        lerp(hash(i), hash(i + float2(1, 0)), f.x),
        lerp(hash(i + float2(0, 1)), hash(i + float2(1, 1)), f.x),
        f.y
    );
}

float fbm(float2 p, int oct)
{
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < oct; i++)
    {
        v += a * vnoise(p);
        p *= 2.02;
        a *= 0.5;
    }
    return v;
}

float4 main(float4 pos : SV_POSITION) : SV_TARGET
{
    float2 uv = pos.xy / resolution;
    float aspect = resolution.x / resolution.y;
    float2 p = float2(uv.x * aspect, uv.y);

        float t = time * 0.35;

    float2 warp = float2(
        fbm(p * 1.3 + float2(t * 0.10, t * 0.05), 3),
        fbm(p * 1.3 + float2(5.2 - t * 0.08, 1.3 + t * 0.07), 3)
    );
    float2 q = p * 1.8 + warp * 2.2;

    float n1 = fbm(q + t * 0.06, 4);
    float n2 = fbm(q * 1.7 - t * 0.05 + 11.0, 3);

    float d1 = saturate(1.0 - abs(n1 * 2.0 - 1.0));
    float d2 = saturate(1.0 - abs(n2 * 2.0 - 1.0));

    // sharp core + soft glow, so the filaments have body and color
    float core = pow(d1, 7.0) + 0.7 * pow(d2, 9.0);
    float glow = pow(d1, 2.5) * 0.12 + pow(d2, 3.0) * 0.08;

    // higher frequency mask => several regions across the whole screen
    float mask = smoothstep(0.30, 0.60, fbm(p * 2.2 + warp * 0.8 + t * 0.04, 3));

    float intensity = (core * 1.0 + glow) * mask;

    // saturated colors that shift across the screen
    float hue = n1 * 1.2 + warp.y * 1.5 + p.x * 0.35 + t * 0.05;
    float3 hueCol = 0.5 + 0.5 * cos(6.28318 * (hue + float3(0.0, 0.33, 0.67)));
    hueCol = pow(hueCol, 1.6); // deepen saturation
    float3 col = intensity * hueCol * 3.2; // gain: brightness

    // light vignette only
    float2 v = uv - 0.5;
    col *= saturate(1.0 - dot(v, v) * 0.5);

    // cards and digits
    float card0Dist = cardSDF(pos.xy, card0);
    float card1Dist = cardSDF(pos.xy, card1);
    float cardMask = min(card0Dist, card1Dist);

    if (cardMask <= 0.0)
    {
        float d0 = renderTwoDigits(pos.xy, card0, roll0_a, roll0_b);
        float d1 = renderTwoDigits(pos.xy, card1, roll1_a, roll1_b);
        float digit = max(d0, d1);
        digit = max(digit, renderAmPm(pos.xy, card0));
        col = lerp(col, float3(1.0, 1.0, 1.0), digit * 0.08);
    }

    return float4(col, 1.0);
}