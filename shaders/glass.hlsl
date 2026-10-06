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

float3 sampleBg(float2 px)
{
    return InputTexture.SampleLevel(InputSampler, clamp(px / resolution, 0.0, 1.0), 0).rgb;
}

float3 blur(float2 px, float r)
{
    float3 c = 0; float tot = 0;
    [unroll] for (int x = -2; x <= 2; x++)
    [unroll] for (int y = -2; y <= 2; y++)
    {
        float w = exp(-0.25 * (x * x + y * y));
        c += sampleBg(px + float2(x, y) * r) * w;
        tot += w;
    }
    return c / tot;
}

float4 main(float4 pos : SV_POSITION) : SV_TARGET
{
    float2 p =
        pos.xy - center;

    float dist =
        roundrect_sdf(p);

    float3 bg =
        sampleBg(pos.xy);

    // Drop shadow outside the card
    float shadow =
        exp(-max(dist, 0.0) / 28.0) *
        0.35 *
        shadow_power;

    shadow *=
        step(0.0, dist);

    // Outside card
    if (dist > 0.5)
    {
        if (shadow < 0.001)
            discard;

        return float4(
            0.0,
            0.0,
            0.0,
            shadow
        );
    }

    // SDF gradient = edge normal direction
    float e = 1.0;

    float2 grad =
        float2(
            roundrect_sdf(
                p + float2(e, 0)
            ) -
            roundrect_sdf(
                p - float2(e, 0)
            ),

            roundrect_sdf(
                p + float2(0, e)
            ) -
            roundrect_sdf(
                p - float2(0, e)
            )
        );

    float2 dir =
        normalize(
            grad + 1e-6
        );

    // Refraction / distortion
    float glassSize =
        min(size.x, size.y);

    float inv =
        -dist / glassSize;

    float distFromCenter =
        1.0 -
        clamp(
            inv / 0.3,
            0.0,
            1.0
        );

    float distortion =
        1.0 -
        sqrt(
            max(
                1.0 -
                distFromCenter *
                distFromCenter,
                0.0
            )
        );

    float2 offsetPx =
        distortion *
        refraction *
        dir *
        glassSize *
        0.5;

    float2 coord =
        pos.xy - offsetPx;

    float blurR =
        blur_strength *
        (1.0 - distFromCenter * 0.5);

    float edge =
        smoothstep(
            0.0,
            0.02,
            inv
        );

    float2 shift =
        dir *
        edge *
        3.0;

    float3 glass =
        float3(
            blur(coord - shift, blurR).r,
            blur(coord,         blurR).g,
            blur(coord + shift, blurR).b
        );

    // Rim highlights
    float rim =
        smoothstep(
            -8.0,
            -0.6,
            dist
        );

    float2 l1 =
        normalize(
            float2(-0.75, -0.65)
        );

    float2 l2 =
        normalize(
            float2(0.9, 0.8)
        );

    float primary =
        rim *
        pow(
            saturate(
                dot(dir, l1)
            ),
            1.8
        ) *
        0.6 *
        glow_power;

    float secondary =
        rim *
        pow(
            saturate(
                dot(dir, l2)
            ),
            1.8
        ) *
        0.35 *
        glow_power;

    glass +=
        primary +
        secondary;

    // Adaptive tint
    float lum =
        dot(
            bg,
            float3(
                0.299,
                0.587,
                0.114
            )
        );

    float response =
        smoothstep(
            0.25,
            0.9,
            lum
        );

    glass *=
        lerp(
            float3(
                1.08,
                1.08,
                1.10
            ),
            float3(
                0.42,
                0.48,
                0.58
            ),
            response
        );

    glass *= 0.90;

    // Glass coverage
    float coverage =
        saturate(
            0.5 - dist
        );

    float3 color =
        lerp(
            bg,
            glass,
            coverage
        );

    return float4(
        color,
        1.0
    );
}