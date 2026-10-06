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

float fbm(float2 p)
{
    float v = 0.0;
    float a = 0.5;

    [unroll]
    for (int i = 0; i < 5; i++)
    {
        v += a * vnoise(p);
        p *= 2.02;
        a *= 0.5;
    }

    return v;
}

float3 palette(float t)
{
    return 0.5 + 0.5 *
        cos(6.28318 * (t + float3(0.0, 0.33, 0.67)));
}

float4 main(float4 pos : SV_POSITION) : SV_TARGET
{
    float2 uv = pos.xy / resolution;

    float aspect =
        resolution.x / resolution.y;

    float2 p =
        float2(
            uv.x * aspect,
            uv.y
        );

    float2 warp = float2(
        fbm(p * 1.5 + time * 0.03),
        fbm(p * 1.5 - time * 0.025 + 7.0)
    );

    float n = fbm(
        p * 2.0 +
        warp * 1.5 +
        time * 0.02
    );

    float3 col =
        palette(
            n * 0.8 +
            uv.y * 0.25 +
            time * 0.02
        );

    col *=
        0.3 +
        0.7 * smoothstep(
            0.2,
            0.8,
            n
        );

    float card0Dist =
        cardSDF(pos.xy, card0);

    float card1Dist =
        cardSDF(pos.xy, card1);

    float cardMask =
        min(card0Dist, card1Dist);

    float digit = 0.0;

    if (cardMask <= 0.0)
    {
        float d0 =
            renderTwoDigits(
                pos.xy,
                card0,
                roll0_a,
                roll0_b
            );

        float d1 =
            renderTwoDigits(
                pos.xy,
                card1,
                roll1_a,
                roll1_b
            );

        digit = max(d0, d1);

        float ampm =
            renderAmPm(
                pos.xy,
                card0
            );

        digit = max(
            digit,
            ampm
        );

        float3 digitColor =
            float3(1.0, 1.0, 1.0);

        col =
            lerp(
                col,
                digitColor,
                digit * 0.08
            );
    }

    return float4(col, 1.0);
}